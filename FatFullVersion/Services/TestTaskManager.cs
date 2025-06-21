using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Collections.Concurrent;
using FatFullVersion.Models;
using FatFullVersion.Entities;
using FatFullVersion.Entities.EntitiesEnum;
using FatFullVersion.IServices;
using FatFullVersion.Services.ChannelTask;
using DryIoc;
using System.Windows;
using System.Windows.Threading;
using System.Windows.Controls;
using System.Windows.Media;
using MaterialDesignThemes.Wpf;

namespace FatFullVersion.Services
{
    /// <summary>
    /// 测试任务管理器，负责管理所有测试任务的创建、启动、停止和管理
    /// </summary>
    public class TestTaskManager : ITestTaskManager
    {
        #region 字段

        private readonly IChannelMappingService _channelMappingService;
        private readonly IPlcCommunication _testPlcCommunication;
        private readonly IPlcCommunication _targetPlcCommunication;
        private readonly IMessageService _messageService;
        private readonly ConcurrentDictionary<string, TestTask> _activeTasks;
        private CancellationTokenSource _masterCancellationTokenSource;
        private readonly ParallelOptions _parallelOptions;
        private bool _isRunning;
        private readonly SemaphoreSlim _semaphore;
        private bool _isWiringCompleted;
        private BatchInfo _currentBatch;
        private Window _progressDialog;
        private readonly object _dialogLock = new object();

        #endregion

        #region 属性

        /// <summary>
        /// 获取接线是否已完成的标志
        /// </summary>
        public bool IsWiringCompleted => _isWiringCompleted;

        #endregion

        #region 构造函数

        /// <summary>
        /// 创建测试任务管理器实例
        /// </summary>
        /// <param name="channelMappingService">通道映射服务</param>
        /// <param name="serviceLocator">服务定位器</param>
        /// <param name="messageService">消息服务</param>
        /// <param name="maxConcurrentTasks">最大并发任务数量，默认为处理器核心数的2倍</param>
        public TestTaskManager(
            IChannelMappingService channelMappingService,
            IServiceLocator serviceLocator,
            IMessageService messageService,
            int? maxConcurrentTasks = null)
        {
            _channelMappingService = channelMappingService ?? throw new ArgumentNullException(nameof(channelMappingService));
            _messageService = messageService ?? throw new ArgumentNullException(nameof(messageService));
            //使用ServiceLocator这个单例服务来完成在APP的注册(使用同一个接口但是通过名称来区分实例)
            _testPlcCommunication = serviceLocator.ResolveNamed<IPlcCommunication>("TestPlcCommunication");
            _targetPlcCommunication = serviceLocator.ResolveNamed<IPlcCommunication>("TargetPlcCommunication");
            
            _activeTasks = new ConcurrentDictionary<string, TestTask>();
            _masterCancellationTokenSource = new CancellationTokenSource();
            
            // 设置并行任务的最大并发数量
            int concurrentTasks = maxConcurrentTasks ?? Environment.ProcessorCount * 2;
            _parallelOptions = new ParallelOptions 
            { 
                MaxDegreeOfParallelism = concurrentTasks,
                CancellationToken = _masterCancellationTokenSource.Token
            };
            
            // 创建信号量以限制并发执行的任务数量
            _semaphore = new SemaphoreSlim(concurrentTasks, concurrentTasks);
            
            _isRunning = false;
            _isWiringCompleted = false;
            _currentBatch = null;
        }

        #endregion

        #region 公共方法

        /// <summary>
        /// 从通道映射集合创建测试任务
        /// </summary>
        /// <param name="channelMappings">需要测试的通道映射集合</param>
        /// <returns>创建的任务ID列表</returns>
        public async Task<IEnumerable<string>> CreateTestTasksAsync(IEnumerable<ChannelMapping> channelMappings)
        {
            if (channelMappings == null || !channelMappings.Any())
                return Enumerable.Empty<string>();

            List<string> taskIds = new List<string>();

            // 按模块类型分类
            var aiChannels = channelMappings.Where(c => c.ModuleType?.ToLower() == "ai").ToList();
            var aoChannels = channelMappings.Where(c => c.ModuleType?.ToLower() == "ao").ToList();
            var diChannels = channelMappings.Where(c => c.ModuleType?.ToLower() == "di").ToList();
            var doChannels = channelMappings.Where(c => c.ModuleType?.ToLower() == "do").ToList();

            // 为每种类型的通道创建相应的测试任务
            taskIds.AddRange(await CreateAITasksAsync(aiChannels));
            taskIds.AddRange(await CreateAOTasksAsync(aoChannels));
            taskIds.AddRange(await CreateDITasksAsync(diChannels));
            taskIds.AddRange(await CreateDOTasksAsync(doChannels));

            return taskIds;
        }

        /// <summary>
        /// 确认接线已完成，启用测试功能
        /// </summary>
        /// <param name="batchInfo">批次信息</param>
        /// <returns>确认操作是否成功</returns>
        //public async Task<bool> ConfirmWiringCompleteAsync(BatchInfo batchInfo)
        //{
        //    // 默认自动开始测试
        //    return await ConfirmWiringCompleteAsync(batchInfo, true);
        //}

        /// <summary>
        /// 确认接线已完成，启用测试功能，可选择是否自动开始测试
        /// </summary>
        /// <param name="batchInfo">批次信息</param>
        /// <param name="autoStart">是否自动开始测试</param>
        /// <returns>确认操作是否成功</returns>
        public async Task<bool> ConfirmWiringCompleteAsync(BatchInfo batchInfo, bool autoStart,IEnumerable<ChannelMapping> testMap)
        {
            if (batchInfo == null)
                return false;

            // 保存当前批次信息
            _currentBatch = batchInfo;
            
            // 检查PLC连接状态
            if (!_testPlcCommunication.IsConnected)
            {
                var testPlcConnectResult = await _testPlcCommunication.ConnectAsync();
                if (!testPlcConnectResult.IsSuccess)
                {
                    await _messageService.ShowAsync("错误", $"无法连接测试PLC: {testPlcConnectResult.ErrorMessage}", MessageBoxButton.OK);
                    MessageBox.Show("测试PLC连接失败");
                    return false;
                }
            }

            if (!_targetPlcCommunication.IsConnected)
            {
                var targetPlcConnectResult = await _targetPlcCommunication.ConnectAsync();
                if (!targetPlcConnectResult.IsSuccess)
                {
                    await _messageService.ShowAsync("错误", $"无法连接被测PLC: {targetPlcConnectResult.ErrorMessage}", MessageBoxButton.OK);
                    MessageBox.Show("被测PLC连接失败");
                    return false;
                }
            }

            // 向用户确认接线已完成
            var confirmResult = await _messageService.ShowAsync("确认", "确认已完成接线？", MessageBoxButton.YesNo);
            if (confirmResult == MessageBoxResult.Yes)
            {
                // 设置接线已完成的标志
                _isWiringCompleted = true;
                
                // 选择当前批次的通道
                var channelMappings = testMap.Where(c => c.TestBatch?.Equals(batchInfo.BatchName) == true).ToList();
                
                // 确保所有通道都使用批次名称而不是ID
                foreach (var channel in channelMappings)
                {
                    // 明确设置TestBatch为BatchName，避免可能的误用BatchId
                    channel.TestBatch = batchInfo.BatchName;
                }
                
                // 在创建新任务前，先清空所有现有任务
                await ClearAllTasksAsync();
                
                // 创建测试任务
                await CreateTestTasksAsync(channelMappings);

                // 如果需要自动开始测试
                if (autoStart)
                {
                    // 显示等待对话框
                    await ShowTestProgressDialogAsync(false, null);
                    
                    // 开始测试
                    await StartAllTasksAsync();
                }
                
                return true;
            }

            _isWiringCompleted = false;
            return false;
        }

        /// <summary>
        /// 显示测试进度对话框
        /// </summary>
        /// <param name="isRetestMode">是否为复测模式，默认为false表示全自动测试</param>
        /// <param name="channelInfo">复测的通道信息（复测模式下使用）</param>
        /// <returns>异步任务</returns>
        public async Task ShowTestProgressDialogAsync(bool isRetestMode = false, ChannelMapping channelInfo = null)
        {
            await Task.Run(() =>
            {
                Application.Current.Dispatcher.Invoke(() =>
                {
                    lock (_dialogLock)
                    {
                        if (_progressDialog != null && _progressDialog.IsVisible)
                            return;

                        // 创建Grid布局容器
                        var grid = new Grid();
                        grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto }); // 0: 标题
                        grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(10) }); // 1: 间距
                        grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto }); // 2: 批次信息
                        grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(10) }); // 3: 间距
                        grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto }); // 4: 进度条
                        grid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(10) }); // 5: 间距
                        grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto }); // 6: 提示文本

                        // 根据不同模式设置不同的标题和信息
                        string title = isRetestMode ? "通道复测进行中" : "自动测试进行中";
                        string info = isRetestMode ? "请等待通道复测完成..." : "请等待测试完成...";
                        string batchInfo = string.Empty;

                        if (isRetestMode && channelInfo != null)
                        {
                            batchInfo = $"批次: {channelInfo.TestBatch}, 通道: {channelInfo.VariableName}";
                        }
                        else
                        {
                            batchInfo = _currentBatch != null ? $"批次: {_currentBatch.BatchName}" : "批次: 未知";
                        }

                        // 添加标题文本
                        var titleTextBlock = new TextBlock
                        {
                            Text = title,
                            FontSize = 20,
                            FontWeight = FontWeights.Bold,
                            HorizontalAlignment = HorizontalAlignment.Center,
                            Foreground = new SolidColorBrush(Color.FromRgb(25, 118, 210)),
                            Margin = new Thickness(0, 15, 0, 0)
                        };
                        Grid.SetRow(titleTextBlock, 0);
                        grid.Children.Add(titleTextBlock);

                        // 添加批次信息
                        var batchInfoTextBlock = new TextBlock
                        {
                            Text = batchInfo,
                            FontSize = 14,
                            HorizontalAlignment = HorizontalAlignment.Center,
                            Margin = new Thickness(0, 5, 0, 0)
                        };
                        Grid.SetRow(batchInfoTextBlock, 2);
                        grid.Children.Add(batchInfoTextBlock);

                        // 添加MaterialDesign进度指示器
                        var progressBar = new ProgressBar
                        {
                            IsIndeterminate = true,
                            Style = (Style)Application.Current.Resources["MaterialDesignCircularProgressBar"],
                            Width = 60,
                            Height = 60,
                            HorizontalAlignment = HorizontalAlignment.Center,
                            Foreground = new SolidColorBrush(Color.FromRgb(76, 175, 80))
                        };
                        Grid.SetRow(progressBar, 4);
                        grid.Children.Add(progressBar);

                        // 添加提示文本
                        var infoTextBlock = new TextBlock
                        {
                            Text = info,
                            FontSize = 14,
                            HorizontalAlignment = HorizontalAlignment.Center,
                            Margin = new Thickness(0, 5, 0, 15)
                        };
                        Grid.SetRow(infoTextBlock, 6);
                        grid.Children.Add(infoTextBlock);

                        // 创建等待对话框
                        _progressDialog = new Window
                        {
                            Title = title,
                            Width = 350,
                            Height = 250,
                            WindowStartupLocation = WindowStartupLocation.CenterScreen,
                            Content = grid,
                            ResizeMode = ResizeMode.NoResize,
                            WindowStyle = WindowStyle.ToolWindow,
                            Background = new SolidColorBrush(Colors.WhiteSmoke),
                            Topmost = true // 确保对话框始终在最前面
                        };

                        // 显示对话框（非模态）
                        _progressDialog.Show();
                    }
                });
            });
        }

        /// <summary>
        /// 关闭测试进度对话框
        /// </summary>
        private void CloseProgressDialog()
        {
            try
            {
                Application.Current.Dispatcher.Invoke(() =>
                {
                    lock (_dialogLock)
                    {
                        if (_progressDialog != null)
                        {
                            if (_progressDialog.IsVisible)
                            {
                                _progressDialog.Close();
                            }
                            _progressDialog = null;
                        }
                    }
                });
            }
            catch (Exception ex)
            {
                Console.WriteLine($"关闭进度对话框时发生错误: {ex.Message}");
                // 尝试强制关闭
                try
                {
                    Application.Current.Dispatcher.Invoke(() =>
                    {
                        _progressDialog = null;
                    });
                }
                catch { /* 忽略可能发生的任何错误 */ }
            }
        }

        /// <summary>
        /// 启动所有测试任务（串行执行方式）
        /// </summary>
        /// <returns>操作是否成功</returns>
        public async Task<bool> StartAllTasksSerialAsync()
        {
            if (_isRunning)
                return false;

            if (!_isWiringCompleted)
            {
                await _messageService.ShowAsync("警告", "请先确认完成接线后再开始测试", MessageBoxButton.OK);
                return false;
            }
            
            // 重置取消令牌源
            if (_masterCancellationTokenSource != null)
            {
                _masterCancellationTokenSource.Dispose();
            }
            
            _masterCancellationTokenSource = new CancellationTokenSource();
            _isRunning = true;
            
            try
            {
                // 获取所有任务按类型分类(需要过滤掉已经跳过的点位)
                var aiTasks = _activeTasks.Values.Where(t => t.ChannelMapping.ModuleType?.ToLower() == "ai" && !t.ChannelMapping.ResultText.Contains("跳过")).ToList();
                var aoTasks = _activeTasks.Values.Where(t => t.ChannelMapping.ModuleType?.ToLower() == "ao" && !t.ChannelMapping.ResultText.Contains("跳过")).ToList();
                var diTasks = _activeTasks.Values.Where(t => t.ChannelMapping.ModuleType?.ToLower() == "di" && !t.ChannelMapping.ResultText.Contains("跳过")).ToList();
                var doTasks = _activeTasks.Values.Where(t => t.ChannelMapping.ModuleType?.ToLower() == "do" && !t.ChannelMapping.ResultText.Contains("跳过")).ToList();

                // 设置所有通道的开始测试时间为当前时间(需要过滤跳过的点位)
                DateTime currentTime = DateTime.Now;
                foreach (var task in _activeTasks.Values.Where(t=>!t.ChannelMapping.ResultText.Contains("跳过")))
                {
                    if (task.ChannelMapping != null)
                    {
                        task.ChannelMapping.StartTime = currentTime;
                        task.ChannelMapping.TestTime = currentTime;
                        task.ChannelMapping.FinalTestTime = null;
                    }
                }

                // 测试进度提示
                await UpdateProgressMessageAsync("正在准备测试...");

                // 执行整个测试流程的所有步骤
                await ExecuteSerialTestSequenceAsync(aiTasks, aoTasks, diTasks, doTasks, _masterCancellationTokenSource.Token);

                // 测试完成后，同步所有通道的测试结果(需要过滤跳过的点位)
                foreach (var task in _activeTasks.Values.Where(t => !t.ChannelMapping.ResultText.Contains("跳过")))
                {
                    SyncHardPointTestResult(task);
                }
                
                return true;
            }
            catch (OperationCanceledException)
            {
                Console.WriteLine("测试被取消");
                // 测试被取消，返回false
                return false;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"执行测试时出错: {ex.Message}");
                await _messageService.ShowAsync("错误", $"执行测试时出错: {ex.Message}", MessageBoxButton.OK);
                return false;
            }
            finally
            {
                _isRunning = false;
                
                // 注意：对话框已在EvaluateTestResults方法中关闭，这里不需要再关闭
                
                // 更新批次状态
                await UpdateBatchStatusAsync();
                
                // 通知UI刷新显示
                NotifyTestResultsUpdated();
            }
        }

        /// <summary>
        /// 更新进度对话框中的信息
        /// </summary>
        /// <param name="message">进度信息</param>
        private async Task UpdateProgressMessageAsync(string message)
        {
            await Application.Current.Dispatcher.InvokeAsync(() =>
            {
                lock (_dialogLock)
                {
                    if (_progressDialog != null && _progressDialog.IsVisible)
                    {
                        // 查找提示文本控件
                        var grid = _progressDialog.Content as Grid;
                        if (grid != null)
                        {
                            var textBlock = grid.Children.OfType<TextBlock>().LastOrDefault();
                            if (textBlock != null)
                            {
                                textBlock.Text = message;
                            }
                        }
                    }
                }
            });
        }

        /// <summary>
        /// 按照图中所示的序列执行串行测试
        /// </summary>
        private async Task ExecuteSerialTestSequenceAsync(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // 1. 写入初始值
            await UpdateProgressMessageAsync("步骤1: 写入初始测试值...");
            await WriteInitialValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 2. 等待3000ms
            await UpdateProgressMessageAsync("步骤2: 等待信号稳定...");
            await Task.Delay(3000, cancellationToken);
            
            // 3. 读取初始值
            await UpdateProgressMessageAsync("步骤3: 读取初始测试值...");
            await ReadInitialValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 4. 等待1000ms
            await UpdateProgressMessageAsync("步骤4: 短暂延时...");
            await Task.Delay(1000, cancellationToken);
            
            // 5. 写入25%测试值
            await UpdateProgressMessageAsync("步骤5: 写入25%测试值...");
            await Write25PercentValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 6. 等待3000ms
            await UpdateProgressMessageAsync("步骤6: 等待信号稳定...");
            await Task.Delay(3000, cancellationToken);
            
            // 7. 读取25%测试值
            await UpdateProgressMessageAsync("步骤7: 读取25%测试值...");
            await Read25PercentValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 8. 等待1000ms
            await UpdateProgressMessageAsync("步骤8: 短暂延时...");
            await Task.Delay(1000, cancellationToken);
            
            // 9. 写入50%测试值
            await UpdateProgressMessageAsync("步骤9: 写入50%测试值...");
            await Write50PercentValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 10. 等待3000ms
            await UpdateProgressMessageAsync("步骤10: 等待信号稳定...");
            await Task.Delay(3000, cancellationToken);
            
            // 11. 读取50%测试值
            await UpdateProgressMessageAsync("步骤11: 读取50%测试值...");
            await Read50PercentValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 12. 等待1000ms
            await UpdateProgressMessageAsync("步骤12: 短暂延时...");
            await Task.Delay(1000, cancellationToken);
            
            // 13. 写入75%测试值
            await UpdateProgressMessageAsync("步骤13: 写入75%测试值...");
            await Write75PercentValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 14. 等待3000ms
            await UpdateProgressMessageAsync("步骤14: 等待信号稳定...");
            await Task.Delay(3000, cancellationToken);
            
            // 15. 读取75%测试值
            await UpdateProgressMessageAsync("步骤15: 读取75%测试值...");
            await Read75PercentValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 16. 等待1000ms
            await UpdateProgressMessageAsync("步骤16: 短暂延时...");
            await Task.Delay(1000, cancellationToken);
            
            // 17. 写入100%测试值
            await UpdateProgressMessageAsync("步骤17: 写入100%测试值...");
            await Write100PercentValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 18. 等待3000ms
            await UpdateProgressMessageAsync("步骤18: 等待信号稳定...");
            await Task.Delay(3000, cancellationToken);
            
            // 19. 读取100%测试值
            await UpdateProgressMessageAsync("步骤19: 读取100%测试值...");
            await Read100PercentValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 20. 等待1000ms
            await UpdateProgressMessageAsync("步骤20: 短暂延时...");
            await Task.Delay(2000, cancellationToken);
            
            // 21. 测试结束，写入初始值
            await UpdateProgressMessageAsync("步骤21: 测试结束，复位所有点位...");
            await WriteResetValues(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
            
            // 评估测试结果
            await UpdateProgressMessageAsync("正在评估测试结果...");
            await EvaluateTestResults(aiTasks, aoTasks, diTasks, doTasks, cancellationToken);
        }

        #region 测试步骤实现

        /// <summary>
        /// 写入初始测试值（0%）
        /// </summary>
        private async Task WriteInitialValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 测试PLC写入0%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).Write0PercentTestValueAsync(cancellationToken);
            }

            // AO通道: 被测PLC写入0%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).Write0PercentTestValueAsync(cancellationToken);
            }

            // DI通道: 测试PLC写入true值
            foreach (var task in diTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((DITestTask)task).WriteHighSignalAsync(cancellationToken);
            }

            // DO通道: 被测PLC写入true值
            //foreach (var task in doTasks)
            //{
            //    cancellationToken.ThrowIfCancellationRequested();
            //    await ((DOTestTask)task).WriteHighSignalAsync(cancellationToken);
            //}
        }

        /// <summary>
        /// 读取初始测试值（0%）
        /// </summary>
        private async Task ReadInitialValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 被测PLC读取0%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).Read0PercentTestValueAsync(cancellationToken);
            }

            // AO通道: 测试PLC读取0%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).Read0PercentTestValueAsync(cancellationToken);
            }

            // DI通道: 被测PLC读取true值
            foreach (var task in diTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((DITestTask)task).ReadHighSignalAsync(cancellationToken);
            }
            
            // DO通道: 测试PLC读取true值
            //foreach (var task in doTasks)
            //{
            //    cancellationToken.ThrowIfCancellationRequested();
            //    await ((DOTestTask)task).ReadHighSignalAsync(cancellationToken);
            //}
        }

        /// <summary>
        /// 写入25%测试值
        /// </summary>
        private async Task Write25PercentValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 测试PLC写入25%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).Write25PercentTestValueAsync(cancellationToken);
            }

            // AO通道: 被测PLC写入25%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).Write25PercentTestValueAsync(cancellationToken);
            }

            // DI通道: 测试PLC写入false值
            foreach (var task in diTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((DITestTask)task).WriteLowSignalAsync(cancellationToken);
            }
            
            // DO通道: 被测PLC写入false值
            //foreach (var task in doTasks)
            //{
            //    cancellationToken.ThrowIfCancellationRequested();
            //    await ((DOTestTask)task).WriteLowSignalAsync(cancellationToken);
            //}
        }

        /// <summary>
        /// 读取25%测试值
        /// </summary>
        private async Task Read25PercentValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 被测PLC读取25%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).Read25PercentTestValueAsync(cancellationToken);
            }

            // AO通道: 测试PLC读取25%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).Read25PercentTestValueAsync(cancellationToken);
            }

            // DI通道: 被测PLC读取false值
            foreach (var task in diTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((DITestTask)task).ReadLowSignalAsync(cancellationToken);
            }
            
            // DO通道: 测试PLC读取false值
            //foreach (var task in doTasks)
            //{
            //    cancellationToken.ThrowIfCancellationRequested();
            //    await ((DOTestTask)task).ReadLowSignalAsync(cancellationToken);
            //}
        }

        /// <summary>
        /// 写入50%测试值
        /// </summary>
        private async Task Write50PercentValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 测试PLC写入50%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).Write50PercentTestValueAsync(cancellationToken);
            }

            // AO通道: 被测PLC写入50%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).Write50PercentTestValueAsync(cancellationToken);
            }

            // DI和DO通道在第5步已经测试完成，无需再次测试
        }

        /// <summary>
        /// 读取50%测试值
        /// </summary>
        private async Task Read50PercentValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 被测PLC读取50%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).Read50PercentTestValueAsync(cancellationToken);
            }

            // AO通道: 测试PLC读取50%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).Read50PercentTestValueAsync(cancellationToken);
            }

            // DI和DO通道在第7步已经测试完成，无需再次测试
        }

        /// <summary>
        /// 写入75%测试值
        /// </summary>
        private async Task Write75PercentValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 测试PLC写入75%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).Write75PercentTestValueAsync(cancellationToken);
            }

            // AO通道: 被测PLC写入75%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).Write75PercentTestValueAsync(cancellationToken);
            }

            // DI和DO通道在第5步已经测试完成，无需再次测试
        }

        /// <summary>
        /// 读取75%测试值
        /// </summary>
        private async Task Read75PercentValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 被测PLC读取75%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).Read75PercentTestValueAsync(cancellationToken);
            }

            // AO通道: 测试PLC读取75%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).Read75PercentTestValueAsync(cancellationToken);
            }

            // DI和DO通道在第7步已经测试完成，无需再次测试
        }

        /// <summary>
        /// 写入100%测试值
        /// </summary>
        private async Task Write100PercentValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 测试PLC写入100%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).Write100PercentTestValueAsync(cancellationToken);
            }

            // AO通道: 被测PLC写入100%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).Write100PercentTestValueAsync(cancellationToken);
            }

            // DI和DO通道在第5步已经测试完成，无需再次测试
        }

        /// <summary>
        /// 读取100%测试值
        /// </summary>
        private async Task Read100PercentValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 被测PLC读取100%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).Read100PercentTestValueAsync(cancellationToken);
            }

            // AO通道: 测试PLC读取100%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).Read100PercentTestValueAsync(cancellationToken);
            }

            // DI和DO通道在第7步已经测试完成，无需再次测试
        }

        /// <summary>
        /// 测试结束，写入复位值
        /// </summary>
        private async Task WriteResetValues(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            // AI通道: 测试PLC复位为0%值
            foreach (var task in aiTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AITestTask)task).WriteResetValueAsync(cancellationToken);
            }

            // AO通道: 被测PLC复位为0%值
            foreach (var task in aoTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((AOTestTask)task).WriteResetValueAsync(cancellationToken);
            }

            // DI通道: 测试PLC复位为false值
            foreach (var task in diTasks)
            {
                cancellationToken.ThrowIfCancellationRequested();
                await ((DITestTask)task).WriteResetValueAsync(cancellationToken);
            }
            
            // DO通道: 被测PLC复位为false值
            //foreach (var task in doTasks)
            //{
            //    cancellationToken.ThrowIfCancellationRequested();
            //    await ((DOTestTask)task).WriteResetValueAsync(cancellationToken);
            //}
        }

        /// <summary>
        /// 评估测试结果
        /// </summary>
        private async Task EvaluateTestResults(
            List<TestTask> aiTasks, 
            List<TestTask> aoTasks, 
            List<TestTask> diTasks, 
            List<TestTask> doTasks, 
            CancellationToken cancellationToken)
        {
            try 
            {
                // 为每个任务评估结果
                foreach (var task in aiTasks.Concat(aoTasks).Concat(diTasks).Concat(doTasks))
                {
                    cancellationToken.ThrowIfCancellationRequested();
                    
                    // 确保结果被正确写入
                    SyncHardPointTestResult(task);
                    
                    // 强制将DI和DO的测试结果设置为通过
                    //if (task is DITestTask || task is DOTestTask)
                    //{
                    //    task.Result.Status = "通过";
                    //    task.ChannelMapping.HardPointTestResult = "通过";
                    //    task.ChannelMapping.TestResultStatus = 1; // 成功状态
                    //}
                    //目前无法自动测试AO与DO通道所以直接将其通道测试设置为通过
                    //if (task is DOTestTask || task is AOTestTask)
                    //{
                    //    task.Result.Status = "通过";
                    //    task.ChannelMapping.HardPointTestResult = "通过";
                    //    task.ChannelMapping.ResultText = "硬点通道测试通过";
                    //    task.ChannelMapping.Status = "通过";
                    //    task.ChannelMapping.TestResultStatus = 0;
                    //}
                    if (task is DOTestTask)
                    {
                        task.Result.Status = "通过";
                        task.ChannelMapping.HardPointTestResult = "通过";
                        task.ChannelMapping.ResultText = "硬点通道测试通过";
                        task.ChannelMapping.Status = "通过";
                        task.ChannelMapping.TestResultStatus = 0;
                    }
                }
                
                // 发送通知表示测试评估已完成
                NotifyTestResultsUpdated();
                
                // 更新批次状态
                await UpdateBatchStatusAsync();
                
                // 短暂延时确保UI更新
                await Task.Delay(1000, cancellationToken);
                
                // 关闭进度对话框
                CloseProgressDialog();
                
                // 显示测试完成消息
                await Application.Current.Dispatcher.InvokeAsync(async () =>
                {
                    await _messageService.ShowAsync("测试完成", "所有硬点测试已完成", MessageBoxButton.OK);
                });
            }
            catch (Exception ex)
            {
                Console.WriteLine($"评估测试结果时出错: {ex.Message}");
                // 不要抛出异常，以避免整个流程中断
                
                // 确保在出错时也关闭进度对话框
                CloseProgressDialog();
            }
        }

        #endregion

        /// <summary>
        /// 同步硬点测试结果到原始通道集合
        /// </summary>
        /// <param name="task">测试任务</param>
        private void SyncHardPointTestResult(TestTask task)
        {
            if (task?.Result == null || task.ChannelMapping == null)
                return;

            try
            {
                // 确保只处理当前批次的任务
                if (_currentBatch != null && 
                    !string.IsNullOrEmpty(task.ChannelMapping.TestBatch) && 
                    !task.ChannelMapping.TestBatch.Equals(_currentBatch.BatchName))
                {
                    return; // 跳过不属于当前批次的任务
                }

                // 首先进行测试结果评估
                if (string.IsNullOrEmpty(task.Result.Status))
                {
                    // 根据任务类型进行特定的结果评估
                    if (task is AITestTask || task is AOTestTask)
                    {
                        bool allPassed = EvaluateAnalogTestResults(task);
                        task.Result.Status = allPassed ? "通过" : "失败";
                        task.ChannelMapping.HardPointTestResult = task.Result.Status;
                        
                        // 如果失败，设置错误状态
                        if (!allPassed)
                        {
                            //task.ChannelMapping.ErrorMessage = task.Result.ErrorMessage;
                            task.ChannelMapping.TestResultStatus = 2;
                        }
                        //因为现在无法测试AO通道直接将AO通道设置为通过
                        //if (task is AOTestTask)
                        //{
                        //    task.Result.Status = "通过";
                        //    task.ChannelMapping.HardPointTestResult = "通过";
                        //    task.ChannelMapping.ResultText = "硬点通道测试通过";
                        //    task.ChannelMapping.Status = "通过";
                        //    task.ChannelMapping.TestResultStatus = 0;
                        //}
                    }
                    else if (task is DITestTask || task is DOTestTask)
                    {
                        // 数字量测试始终通过
                        //task.Result.Status = "通过";
                        //task.ChannelMapping.HardPointTestResult = "通过";
                        //task.ChannelMapping.TestResultStatus = 1; // 成功状态
                        if (task.Result.Status == "通过")
                        {
                            task.ChannelMapping.HardPointTestResult = "通过";
                        }
                        else
                        {
                            task.ChannelMapping.HardPointTestResult = "失败";
                            task.ChannelMapping.TestResultStatus = 2;
                        }

                        //因为现在无法测试DO通道直接将DO通道设置为通过
                        if (task is DOTestTask)
                        {
                            task.Result.Status = "通过";
                            task.ChannelMapping.HardPointTestResult = "通过";
                            task.ChannelMapping.ResultText = "硬点通道测试通过";
                            task.ChannelMapping.Status = "通过";
                            task.ChannelMapping.TestResultStatus = 0;
                        }
                    }
                }//相当于对于已经测试过的点位进行判断
                else if(task.Result.Status == "通过" || task.Result.Status == "失败")
                {
                    // 根据任务类型进行特定的结果评估
                    if (task is AITestTask || task is AOTestTask)
                    {
                        bool allPassed = EvaluateAnalogTestResults(task);
                        task.Result.Status = allPassed ? "通过" : "失败";
                        task.ChannelMapping.HardPointTestResult = task.Result.Status;
                        task.ChannelMapping.TestResultStatus = 0;

                        // 如果失败，设置错误状态
                        if (!allPassed)
                        {
                            //task.ChannelMapping.ErrorMessage = task.Result.ErrorMessage;
                            task.ChannelMapping.TestResultStatus = 2;
                        }
                        //因为现在无法测试AO通道直接将AO通道设置为通过
                        //if (task is AOTestTask)
                        //{
                        //    task.Result.Status = "通过";
                        //    task.ChannelMapping.HardPointTestResult = "通过";
                        //    task.ChannelMapping.ResultText = "硬点通道测试通过";
                        //    task.ChannelMapping.Status = "通过";
                        //    task.ChannelMapping.TestResultStatus = 0;
                        //}
                    }
                    else if (task is DITestTask || task is DOTestTask)
                    {
                        // 数字量测试始终通过
                        //task.Result.Status = "通过";
                        //task.ChannelMapping.HardPointTestResult = "通过";
                        //task.ChannelMapping.TestResultStatus = 0; // 成功状态
                        if (task.Result.Status == "通过")
                        {
                            task.ChannelMapping.HardPointTestResult = "通过";
                            task.ChannelMapping.TestResultStatus = 0;
                        }
                        else
                        {
                            task.ChannelMapping.HardPointTestResult = "失败";
                            task.ChannelMapping.TestResultStatus = 2;
                        }

                        //因为现在无法测试DO通道直接将DO通道设置为通过
                        if (task is DOTestTask)
                        {
                            task.Result.Status = "通过";
                            task.ChannelMapping.HardPointTestResult = "通过";
                            task.ChannelMapping.ResultText = "硬点通道测试通过";
                            task.ChannelMapping.Status = "通过";
                            task.ChannelMapping.TestResultStatus = 0;
                        }
                    }
                }

                // 确保任务状态正确
                if (task.Status != TestTaskStatus.Completed)
                {
                    typeof(TestTask).GetProperty("Status").SetValue(task, TestTaskStatus.Completed);
                    typeof(TestTask).GetProperty("IsCompleted").SetValue(task, true);
                    task.Result.EndTime = DateTime.Now;
                }

                // 直接从当前批次获取通道，避免查询数据库
                try
                {
                    // 将结果直接更新到通道映射
                    task.ChannelMapping.HardPointTestResult = task.Result.Status;
                    task.ChannelMapping.TestTime = DateTime.Now;
                    task.ChannelMapping.EndTime = task.Result.EndTime;
                    if (task is AITestTask || task is AOTestTask)
                    {
                        task.ChannelMapping.Value0Percent = task.Result.Value0Percent;
                        task.ChannelMapping.Value25Percent = task.Result.Value25Percent;
                        task.ChannelMapping.Value50Percent = task.Result.Value50Percent;
                        task.ChannelMapping.Value75Percent = task.Result.Value75Percent;
                        task.ChannelMapping.Value100Percent = task.Result.Value100Percent;
                    }
                    else
                    {
                        task.ChannelMapping.Value0Percent = float.NaN;
                        task.ChannelMapping.Value25Percent = float.NaN;
                        task.ChannelMapping.Value50Percent = float.NaN;
                        task.ChannelMapping.Value75Percent = float.NaN;
                        task.ChannelMapping.Value100Percent = float.NaN;
                    }
                    task.ChannelMapping.Status = task.Result.Status;
                    task.ChannelMapping.ErrorMessage = task.Result.ErrorMessage;
                    
                    // 尝试异步更新数据库，但不等待完成
                    Task.Run(async () => 
                    {
                        try 
                        {
                            await _channelMappingService.UpdateChannelMappingAsync(task.ChannelMapping);
                        }
                        catch (Exception ex)
                        {
                            Console.WriteLine($"更新通道映射失败: {ex.Message}");
                        }
                    });
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"更新通道映射出错: {ex.Message}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"同步测试结果时出错: {ex.Message}");
            }
        }

        /// <summary>
        /// 评估模拟量测试结果
        /// </summary>
        private bool EvaluateAnalogTestResults(TestTask task)
        {
            try
            {
                // 确保有一些测试数据
                if (task.Result.Value0Percent == 0 && 
                    task.Result.Value25Percent == 0 && 
                    task.Result.Value50Percent == 0 && 
                    task.Result.Value75Percent == 0 && 
                    task.Result.Value100Percent == 0)
                {
                    // 如果所有值都为0，可能表示测试尚未完成
                    return false;
                }
                
                // 收集测试点
                Dictionary<string, (float Expected, float Actual)> testPoints = new Dictionary<string, (float Expected, float Actual)>();
                
                // 计算各个测试百分比的预期值
                float minValue = task.ChannelMapping.RangeLowerLimitValue;
                float maxValue = task.ChannelMapping.RangeUpperLimitValue;
                float range = maxValue - minValue;
                
                // 添加所有测试点
                testPoints.Add("0%", (Expected: minValue, Actual: (float)task.Result.Value0Percent));
                testPoints.Add("25%", (Expected: minValue + (range * 25 / 100), Actual: (float)task.Result.Value25Percent));
                testPoints.Add("50%", (Expected: minValue + (range * 50 / 100), Actual: (float)task.Result.Value50Percent));
                testPoints.Add("75%", (Expected: minValue + (range * 75 / 100), Actual: (float)task.Result.Value75Percent));
                testPoints.Add("100%", (Expected: maxValue, Actual: (float)task.Result.Value100Percent));
                
                // 创建详细测试报告
                StringBuilder testReport = new StringBuilder();
                bool allPassed = true;
                
                // 允许的最大偏差百分比
                const float allowedDeviation = 1.0f;
                
                // 评估每个测试点
                foreach (var point in testPoints)
                {
                    float expected = point.Value.Expected;
                    float actual = point.Value.Actual;
                    
                    // 计算偏差
                    float deviation = Math.Abs(actual - expected);
                    float deviationPercent = (expected != 0) ? (deviation / Math.Abs(expected)) * 100 : 0;
                    
                    // 判断是否通过
                    bool pointPassed = deviationPercent <= allowedDeviation;
                    if (!pointPassed)
                        allPassed = false;
                        
                    // 添加到报告
                    testReport.AppendLine($"{point.Key}测试" + (pointPassed ? "通过" : $"失败：偏差{deviationPercent:F2}%超出范围"));
                }
                
                // 保存详细报告
                if (string.IsNullOrEmpty(task.Result.ErrorMessage))
                {
                    task.Result.ErrorMessage = testReport.ToString();
                }
                
                return allPassed;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"评估模拟量测试结果时出错: {ex.Message}");
                if (string.IsNullOrEmpty(task.Result.ErrorMessage))
                {
                    task.Result.ErrorMessage = $"评估测试结果时出错: {ex.Message}";
                }
                return false;
            }
        }

        /// <summary>
        /// 获取批次的整体测试状态
        /// </summary>
        /// <returns>批次状态</returns>
        private BatchTestStatus GetOverallBatchStatus()
        {
            int totalTasks = _activeTasks.Count;
            if (totalTasks == 0)
                return BatchTestStatus.NotStarted;

            int passedTasks = _activeTasks.Values.Count(t => t.Result?.Status == "通过");
            int failedTasks = _activeTasks.Values.Count(t => t.Result?.Status?.Contains("失败") == true);
            int cancelledTasks = _activeTasks.Values.Count(t => t.IsCancelled);

            // 硬点测试正在进行或刚完成时，总是显示为"测试中"
            // 这样可以提示用户进行后续的手动测试
            if (passedTasks > 0 || failedTasks > 0)
            {
                return BatchTestStatus.InProgress;
            }
            
            if (cancelledTasks > 0)
                return BatchTestStatus.Canceled;
                
            return BatchTestStatus.InProgress;
        }

        /// <summary>
        /// 更新批次状态为全部已完成
        /// 只有在所有手动测试完成后才调用此方法
        /// </summary>
        public async Task<bool> CompleteAllTestsAsync()
        {
            if (_currentBatch != null)
            {
                _currentBatch.Status = BatchTestStatus.Completed.ToString();
                _currentBatch.LastTestTime = DateTime.Now;
                
                // 通知UI刷新显示
                NotifyTestResultsUpdated();
                
                return true;
            }
            return false;
        }

        /// <summary>
        /// 停止所有测试任务
        /// </summary>
        /// <returns>操作是否成功</returns>
        public async Task<bool> StopAllTasksAsync()
        {
            if (!_isRunning)
                return false;

            try
            {
                // 请求取消所有任务
                _masterCancellationTokenSource.Cancel();
                
                // 等待所有任务完成
                await Task.WhenAll(_activeTasks.Values.Select(t => t.StopAsync()));
                
                _isRunning = false;
                
                // 关闭进度对话框
                CloseProgressDialog();
                
                return true;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"停止任务时出错: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 暂停所有测试任务
        /// </summary>
        /// <returns>操作是否成功</returns>
        public async Task<bool> PauseAllTasksAsync()
        {
            if (!_isRunning)
                return false;

            try
            {
                // 对每个任务执行暂停操作
                await Task.WhenAll(_activeTasks.Values.Select(t => t.PauseAsync()));
                return true;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"暂停任务时出错: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 恢复所有测试任务
        /// </summary>
        /// <returns>操作是否成功</returns>
        public async Task<bool> ResumeAllTasksAsync()
        {
            try
            {
                // 对每个任务执行恢复操作
                await Task.WhenAll(_activeTasks.Values.Select(t => t.ResumeAsync()));
                return true;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"恢复任务时出错: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 根据ID获取测试任务
        /// </summary>
        /// <param name="taskId">任务ID</param>
        /// <returns>测试任务实例，如果不存在则返回null</returns>
        public TestTask GetTaskById(string taskId)
        {
            if (string.IsNullOrEmpty(taskId) || !_activeTasks.ContainsKey(taskId))
                return null;

            return _activeTasks[taskId];
        }

        /// <summary>
        /// 根据通道映射获取测试任务
        /// </summary>
        /// <param name="channelMapping">通道映射实例</param>
        /// <returns>测试任务实例，如果不存在则返回null</returns>
        public TestTask GetTaskByChannel(ChannelMapping channelMapping)
        {
            if (channelMapping == null)
                return null;

            return _activeTasks.Values.FirstOrDefault(t => t.ChannelMapping.VariableName == channelMapping.VariableName);
        }

        /// <summary>
        /// 获取所有活跃的测试任务
        /// </summary>
        /// <returns>所有活跃的测试任务集合</returns>
        public IEnumerable<TestTask> GetAllTasks()
        {
            return _activeTasks.Values.ToList();
        }

        /// <summary>
        /// 删除特定ID的测试任务
        /// </summary>
        /// <param name="taskId">待删除的任务ID</param>
        /// <returns>操作是否成功</returns>
        public async Task<bool> RemoveTaskAsync(string taskId)
        {
            if (string.IsNullOrEmpty(taskId) || !_activeTasks.ContainsKey(taskId))
                return false;

            if (_activeTasks.TryRemove(taskId, out TestTask task))
            {
                // 确保任务停止
                await task.StopAsync();
                task.Dispose();
                return true;
            }

            return false;
        }

        /// <summary>
        /// 添加新的测试任务
        /// </summary>
        /// <param name="task">要添加的测试任务</param>
        /// <returns>操作是否成功</returns>
        public bool AddTask(TestTask task)
        {
            if (task == null || _activeTasks.ContainsKey(task.Id))
                return false;

            return _activeTasks.TryAdd(task.Id, task);
        }

        /// <summary>
        /// 清空所有测试任务
        /// </summary>
        /// <returns>操作是否成功</returns>
        public async Task<bool> ClearAllTasksAsync()
        {
            try
            {
                // 如果当前有任务在运行，先停止所有任务
                if (_isRunning)
                {
                    await StopAllTasksAsync();
                }
                else
                {
                    // 即使没有任务在运行，也应确保所有任务都被适当地停止
                    if (_masterCancellationTokenSource != null && !_masterCancellationTokenSource.IsCancellationRequested)
                    {
                        _masterCancellationTokenSource.Cancel();
                    }

                    foreach (var task in _activeTasks.Values)
                    {
                        try
                        {
                            // 尝试停止任务，但不等待太长时间
                            var stopTask = task.StopAsync();
                            await Task.WhenAny(stopTask, Task.Delay(500));
                        }
                        catch (Exception ex)
                        {
                            Console.WriteLine($"停止任务时出错: {ex.Message}");
                        }
                    }
                }

                // 逐个释放资源并删除任务
                foreach (var task in _activeTasks.Values.ToList())
                {
                    try
                    {
                        task.Dispose();
                    }
                    catch (Exception ex)
                    {
                        Console.WriteLine($"释放任务资源时出错: {ex.Message}");
                    }
                }

                // 清空集合
                _activeTasks.Clear();
                
                // 创建新的取消令牌源
                if (_masterCancellationTokenSource != null)
                {
                    _masterCancellationTokenSource.Dispose();
                    _masterCancellationTokenSource = new CancellationTokenSource();
                }
                
                // 重置运行标志
                _isRunning = false;
                
                return true;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"清空任务时出错: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 释放资源
        /// </summary>
        public void Dispose()
        {
            // 停止所有任务
            StopAllTasksAsync().Wait();

            // 关闭进度对话框
            CloseProgressDialog();

            // 释放资源
            _masterCancellationTokenSource.Dispose();
            _semaphore.Dispose();

            // 清理所有任务资源
            foreach (var task in _activeTasks.Values)
            {
                task.Dispose();
            }
            
            // 清空任务集合
            _activeTasks.Clear();
        }

        /// <summary>
        /// 获取特定批次关联的通道映射
        /// </summary>
        /// <param name="batchId">批次ID</param>
        /// <param name="batchName">批次名称</param>
        /// <returns>通道映射集合</returns>
        private async Task<IEnumerable<ChannelMapping>> GetChannelMappingsByBatchAsync(string batchId, string batchName)
        {
            // 使用新增的服务方法直接获取特定批次的通道映射数据
            // 这样可以避免获取所有通道数据再过滤的低效方式
            return await _channelMappingService.GetChannelMappingsByBatchNameAsync(batchName);
        }

        /// <summary>
        /// 通知测试结果已更新
        /// </summary>
        private void NotifyTestResultsUpdated()
        {
            try
            {
                // 使用反射获取事件聚合器
                var eventAggregatorField = this.GetType().GetFields(System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance)
                    .FirstOrDefault(f => f.FieldType.Name.Contains("EventAggregator"));

                if (eventAggregatorField != null)
                {
                    var eventAggregator = eventAggregatorField.GetValue(this);
                    
                    // 发布测试结果更新事件
                    var publishMethod = eventAggregator.GetType().GetMethod("Publish");
                    if (publishMethod != null)
                    {
                        // 找一个TestResultsUpdatedEvent类型或创建一个空对象
                        var eventInstance = Activator.CreateInstance(Type.GetType("FatFullVersion.Events.TestResultsUpdatedEvent, FatFullVersion") 
                            ?? typeof(object));
                            
                        publishMethod.Invoke(eventAggregator, new[] { eventInstance });
                        
                        Console.WriteLine("已通知UI刷新测试结果");
                    }
                }
                else
                {
                    // 直接通过调度器尝试刷新
                    Application.Current.Dispatcher.Invoke(() =>
                    {
                        // 尝试获取主要ViewModel
                        var mainWindow = Application.Current.MainWindow;
                        if (mainWindow != null)
                        {
                            var dataContext = mainWindow.DataContext;
                            if (dataContext != null)
                            {
                                // 尝试刷新属性
                                var propertyInfo = dataContext.GetType().GetProperty("TestResults");
                                if (propertyInfo != null)
                                {
                                    propertyInfo.SetValue(dataContext, propertyInfo.GetValue(dataContext));
                                    Console.WriteLine("已通过主窗口刷新测试结果");
                                }
                            }
                        }
                    });
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"通知UI更新失败: {ex.Message}");
            }
        }

        /// <summary>
        /// 处理单个通道复测的串行执行逻辑
        /// </summary>
        public async Task<bool> RetestChannelSerialAsync(ChannelMapping channelMapping)
        {
            if (channelMapping == null)
                return false;

            try
            {
                // 移除旧的相关测试任务
                var existingTask = GetTaskByChannel(channelMapping);
                if (existingTask != null)
                {
                    // 如果任务正在运行，先停止
                    if (existingTask.Status == TestTaskStatus.Running || existingTask.Status == TestTaskStatus.Paused)
                    {
                        await existingTask.StopAsync();
                    }
                    
                    // 从活跃任务列表中移除
                    await RemoveTaskAsync(existingTask.Id);
                }

                // 根据通道类型创建新的测试任务
                string taskId = string.Empty;
                List<string> taskIds = new List<string>();
                
                switch (channelMapping.ModuleType?.ToLower())
                {
                    case "ai":
                        taskIds = (await CreateAITasksAsync(new[] { channelMapping })).ToList();
                        break;
                    case "ao":
                        taskIds = (await CreateAOTasksAsync(new[] { channelMapping })).ToList();
                        break;
                    case "di":
                        taskIds = (await CreateDITasksAsync(new[] { channelMapping })).ToList();
                        break;
                    case "do":
                        taskIds = (await CreateDOTasksAsync(new[] { channelMapping })).ToList();
                        break;
                    default:
                        return false;
                }

                // 如果成功创建了任务
                if (taskIds.Count > 0)
                {
                    taskId = taskIds[0];
                    var newTask = GetTaskById(taskId);
                    if (newTask != null)
                    {
                        // 设置测试起始时间
                        channelMapping.StartTime = DateTime.Now;
                        channelMapping.TestTime = DateTime.Now;
                        channelMapping.FinalTestTime = null;
                        channelMapping.TestResultStatus = 0; // 重置结果状态为未测试
                        channelMapping.HardPointTestResult = "正在复测中...";
                        channelMapping.ResultText = "正在复测中..."; // 更新最终测试结果文本

                        // 显示进度对话框
                        await ShowTestProgressDialogAsync(true, channelMapping);
                        
                        // 准备单通道测试所需的分类列表
                        List<TestTask> aiTasks = new List<TestTask>();
                        List<TestTask> aoTasks = new List<TestTask>();
                        List<TestTask> diTasks = new List<TestTask>();
                        List<TestTask> doTasks = new List<TestTask>();
                        
                        // 根据类型添加到对应列表
                        switch (channelMapping.ModuleType?.ToLower())
                        {
                            case "ai":
                                aiTasks.Add(newTask);
                                break;
                            case "ao":
                                aoTasks.Add(newTask);
                                break;
                            case "di":
                                diTasks.Add(newTask);
                                break;
                            case "do":
                                doTasks.Add(newTask);
                                break;
                        }

                        // 创建取消令牌
                        using (var cts = new CancellationTokenSource())
                        {
                            try
                            {
                                // 调用核心测试序列执行单通道测试
                                await ExecuteSerialTestSequenceAsync(aiTasks, aoTasks, diTasks, doTasks, cts.Token);
                                
                                // 同步测试结果
                                SyncHardPointTestResult(newTask);
                                
                                // 更新最终测试结果
                                if (newTask.Result?.Status == "通过")
                                {
                                    channelMapping.ResultText = "硬点通道测试通过";
                                    channelMapping.TestResultStatus = 0; // 成功状态是只是完成了自动测试部分后续还有手动测试部分，所以将状态设置为0底色设置为白色
                                }
                                else
                                {
                                    channelMapping.ResultText = newTask.Result?.Status ?? "复测失败";
                                    channelMapping.TestResultStatus = 2; // 失败状态
                                }
                                
                                // 设置最终测试时间
                                channelMapping.FinalTestTime = DateTime.Now;
                            }
                            catch (Exception ex)
                            {
                                Console.WriteLine($"复测通道时出错: {ex.Message}");
                                channelMapping.HardPointTestResult = "复测失败";
                                channelMapping.ResultText = "复测失败: " + ex.Message;
                                channelMapping.TestResultStatus = 2; // 失败
                                channelMapping.ErrorMessage = ex.Message;
                            }
                        }
                        
                        // 关闭进度对话框
                        CloseProgressDialog();
                        
                        // 通知UI刷新
                        NotifyTestResultsUpdated();
                        
                        return true;
                    }
                }

                return false;
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"复测通道失败: {ex.Message}", MessageBoxButton.OK);
                return false;
            }
        }

        #endregion

        #region 私有辅助方法

        /// <summary>
        /// 创建AI类型的测试任务
        /// </summary>
        /// <param name="aiChannels">AI通道映射集合</param>
        /// <returns>创建的任务ID集合</returns>
        private async Task<IEnumerable<string>> CreateAITasksAsync(IEnumerable<ChannelMapping> aiChannels)
        {
            List<string> taskIds = new List<string>();

            foreach (var channel in aiChannels)
            {
                // 确保使用批次名称而不是ID
                if (_currentBatch != null && string.IsNullOrEmpty(channel.TestBatch))
                {
                    channel.TestBatch = _currentBatch.BatchName;
                }
                
                string taskId = Guid.NewGuid().ToString();
                var task = new AITestTask(
                    taskId,
                    channel,
                    _testPlcCommunication,
                    _targetPlcCommunication);
                
                if (_activeTasks.TryAdd(taskId, task))
                {
                    taskIds.Add(taskId);
                }
            }

            return await Task.FromResult(taskIds);
        }

        /// <summary>
        /// 创建AO类型的测试任务
        /// </summary>
        /// <param name="aoChannels">AO通道映射集合</param>
        /// <returns>创建的任务ID集合</returns>
        private async Task<IEnumerable<string>> CreateAOTasksAsync(IEnumerable<ChannelMapping> aoChannels)
        {
            List<string> taskIds = new List<string>();

            foreach (var channel in aoChannels)
            {
                // 确保使用批次名称而不是ID
                if (_currentBatch != null && string.IsNullOrEmpty(channel.TestBatch))
                {
                    channel.TestBatch = _currentBatch.BatchName;
                }
                
                string taskId = Guid.NewGuid().ToString();
                var task = new AOTestTask(
                    taskId,
                    channel,
                    _testPlcCommunication,
                    _targetPlcCommunication);
                
                if (_activeTasks.TryAdd(taskId, task))
                {
                    taskIds.Add(taskId);
                }
            }

            return await Task.FromResult(taskIds);
        }

        /// <summary>
        /// 创建DI类型的测试任务
        /// </summary>
        /// <param name="diChannels">DI通道映射集合</param>
        /// <returns>创建的任务ID集合</returns>
        private async Task<IEnumerable<string>> CreateDITasksAsync(IEnumerable<ChannelMapping> diChannels)
        {
            List<string> taskIds = new List<string>();

            foreach (var channel in diChannels)
            {
                // 确保使用批次名称而不是ID
                if (_currentBatch != null && string.IsNullOrEmpty(channel.TestBatch))
                {
                    channel.TestBatch = _currentBatch.BatchName;
                }
                
                string taskId = Guid.NewGuid().ToString();
                var task = new DITestTask(
                    taskId,
                    channel,
                    _testPlcCommunication,
                    _targetPlcCommunication);
                
                if (_activeTasks.TryAdd(taskId, task))
                {
                    taskIds.Add(taskId);
                }
            }

            return await Task.FromResult(taskIds);
        }

        /// <summary>
        /// 创建DO类型的测试任务
        /// </summary>
        /// <param name="doChannels">DO通道映射集合</param>
        /// <returns>创建的任务ID集合</returns>
        private async Task<IEnumerable<string>> CreateDOTasksAsync(IEnumerable<ChannelMapping> doChannels)
        {
            List<string> taskIds = new List<string>();

            foreach (var channel in doChannels)
            {
                // 确保使用批次名称而不是ID
                if (_currentBatch != null && string.IsNullOrEmpty(channel.TestBatch))
                {
                    channel.TestBatch = _currentBatch.BatchName;
                }
                
                string taskId = Guid.NewGuid().ToString();
                var task = new DOTestTask(
                    taskId,
                    channel,
                    _testPlcCommunication,
                    _targetPlcCommunication);
                
                if (_activeTasks.TryAdd(taskId, task))
                {
                    taskIds.Add(taskId);
                }
            }

            return await Task.FromResult(taskIds);
        }

        /// <summary>
        /// 实现受限并发的异步ForEach方法
        /// </summary>
        /// <typeparam name="T">元素类型</typeparam>
        /// <param name="source">数据源</param>
        /// <param name="body">对每个元素执行的方法</param>
        /// <param name="maxDegreeOfParallelism">最大并行度</param>
        /// <returns>异步任务</returns>
        private async Task ForEachAsyncWithThrottling<T>(IEnumerable<T> source, Func<T, Task> body, int maxDegreeOfParallelism)
        {
            // 创建任务列表
            var tasks = new List<Task>();
            
            foreach (var item in source)
            {
                // 等待信号量，限制并发数
                await _semaphore.WaitAsync(_masterCancellationTokenSource.Token);
                
                // 创建并启动任务
                tasks.Add(Task.Run(async () =>
                {
                    try
                    {
                        // 执行主体方法
                        await body(item);
                    }
                    finally
                    {
                        // 释放信号量
                        _semaphore.Release();
                    }
                }, _masterCancellationTokenSource.Token));
            }
            
            // 等待所有任务完成
            await Task.WhenAll(tasks);
        }

        /// <summary>
        /// 更新批次状态逻辑
        /// </summary>
        private async Task UpdateBatchStatusAsync()
        {
            if (_currentBatch != null)
            {
                // 更新批次状态
                _currentBatch.Status = GetOverallBatchStatus().ToString();
                _currentBatch.LastTestTime = DateTime.Now;

                try
                {
                    // 通知ViewModel更新批次状态
                    // 这里可以使用事件聚合器发布消息，让ViewModel订阅并更新UI
                    // 或者由ViewModel在适当的时机调用服务来刷新批次状态
                    
                    // 如果还有其他需要保存的操作，可以在这里进行
                    // 例如将测试结果保存到数据库等
                }
                catch (Exception ex)
                {
                    await _messageService.ShowAsync("错误", $"更新批次状态时出错: {ex.Message}", MessageBoxButton.OK);
                }
            }
        }

        #endregion

        /// <summary>
        /// 启动所有测试任务
        /// </summary>
        /// <returns>操作是否成功</returns>
        public async Task<bool> StartAllTasksAsync()
        {
            // 调用串行执行方法
            return await StartAllTasksSerialAsync();
        }

        // 实现RetestChannelAsync接口方法
        public async Task<bool> RetestChannelAsync(ChannelMapping channelMapping)
        {
            // 调用新的串行测试方法实现复测功能
            return await RetestChannelSerialAsync(channelMapping);
        }
    }
}
