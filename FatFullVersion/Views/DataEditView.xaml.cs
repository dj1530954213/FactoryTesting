using System;
using System.Linq;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using FatFullVersion.IServices;
using FatFullVersion.Optimizations;
using FatFullVersion.Services.Interfaces;
using FatFullVersion.ViewModels;
using Prism.Events;

namespace FatFullVersion.Views
{
    /// <summary>
    /// DataEditView.xaml 的交互逻辑
    /// </summary>
    public partial class DataEditView : UserControl
    {
        private readonly IPointDataService _pointDataService;
        private readonly IChannelMappingService _channelMappingService;
        private readonly IEventAggregator _eventAggregator;
        private readonly IServiceLocator _serviceLocator;

        private readonly ITestTaskManager _testTaskManager;
        private readonly IPlcCommunication _testPlc;
        private readonly IPlcCommunication _targetPlc;
        private readonly IMessageService _messageService;
        private readonly ITestResultExportService _testResultExportService;
        private readonly ITestRecordService _testRecordService;
        private readonly IChannelRangerSettingService _channelRangeSettingService;

        /// <summary>
        /// 构造函数
        /// </summary>
        /// <param name="pointDataService">点位数据服务</param>
        /// <param name="channelMappingService">通道映射服务</param>
        /// <param name="eventAggregator">事件聚合器</param>
        /// <param name="testTaskManager">测试任务管理器</param>
        public DataEditView(
            IPointDataService pointDataService,
            IChannelMappingService channelMappingService,
            ITestTaskManager testTaskManager,
            IEventAggregator eventAggregator,
            //IPlcCommunication testPlc,
            //IPlcCommunication targetPlc,
            IServiceLocator serviceLocator,
            IMessageService messageService, 
            ITestResultExportService testResultExportService,
            ITestRecordService testRecordService,
            IChannelRangerSettingService channelRangeSettingService)
        {
            _pointDataService = pointDataService;
            _channelMappingService = channelMappingService;
            _eventAggregator = eventAggregator;
            _serviceLocator = serviceLocator;
            _testTaskManager = testTaskManager;
            //_testPlc = testPlc ?? throw new ArgumentNullException(nameof(testPlc));
            //_targetPlc = targetPlc ?? throw new ArgumentNullException(nameof(targetPlc));
            //调用依赖注入的时候需要显示的指定名称
            _testPlc = serviceLocator.ResolveNamed<IPlcCommunication>("TestPlcCommunication");
            _targetPlc = serviceLocator.ResolveNamed<IPlcCommunication>("TargetPlcCommunication");
            _messageService = messageService ?? throw new ArgumentNullException(nameof(messageService));
            _testResultExportService = testResultExportService;
            _testRecordService = testRecordService ?? throw new ArgumentNullException(nameof(testRecordService));
            _channelRangeSettingService = channelRangeSettingService ?? throw new ArgumentNullException(nameof(channelRangeSettingService));

            try
            {
                // 初始化组件（由XAML设计器生成的方法）
                InitializeComponent();
                
                // 确保ViewModel已正确初始化
                if (this.DataContext == null)
                {
                    this.DataContext = new DataEditViewModel(
                        _pointDataService,
                        _channelMappingService,
                        _testTaskManager,
                        _eventAggregator,
                        _testPlc,
                        _targetPlc,
                        _testResultExportService,
                        _testRecordService,
                        _channelRangeSettingService
                        );
                }
                
                // 注册加载完成事件用于延迟初始化
                this.Loaded += DataEditView_Loaded;
                
                // 注册卸载事件处理程序以清理内存
                this.Unloaded += DataEditView_Unloaded;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"初始化DataEditView时出错: {ex.Message}");
                // 防止异常向上传播
            }
        }
        
        private void DataEditView_Loaded(object sender, RoutedEventArgs e)
        {
            try
            {
                // 使用低优先级操作以确保界面已完全加载
                Application.Current.Dispatcher.BeginInvoke(
                    System.Windows.Threading.DispatcherPriority.Background,
                    new Action(() => {
                        OptimizeDataGrids();
                    })
                );
            }
            catch (Exception ex)
            {
                Console.WriteLine($"加载DataEditView时出错: {ex.Message}");
            }
        }
        
        private void DataEditView_Unloaded(object sender, RoutedEventArgs e)
        {
            try
            {
                // 清理内存
                MemoryOptimizations.CleanupMemory();
                
                // 移除事件处理程序
                this.Loaded -= DataEditView_Loaded;
                this.Unloaded -= DataEditView_Unloaded;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"卸载DataEditView时出错: {ex.Message}");
            }
        }
        
        private void OptimizeDataGrids()
        {
            try
            {
                // 使用FindName方法获取ChannelTabControl
                var channelTabControl = this.FindName("ChannelTabControl") as TabControl;
                
                // 安全地获取所有DataGrid控件
                if (channelTabControl != null)
                {
                    foreach (TabItem tabItem in channelTabControl.Items)
                    {
                        if (tabItem.Content is DataGrid grid)
                        {
                            MemoryOptimizations.OptimizeDataGrid(grid);
                        }
                    }
                }
                
                // 获取最后一个DataGrid（测试结果表格）
                var grids = this.FindVisualChildren<DataGrid>();
                var resultGrid = grids.LastOrDefault();
                if (resultGrid != null)
                {
                    MemoryOptimizations.OptimizeDataGrid(resultGrid);
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"优化DataGrid时出错: {ex.Message}");
            }
        }
        
        // 辅助方法：查找指定类型的所有子控件
        private System.Collections.Generic.IEnumerable<T> FindVisualChildren<T>() where T : DependencyObject
        {
            var count = VisualTreeHelper.GetChildrenCount(this);
            for (int i = 0; i < count; i++)
            {
                var child = VisualTreeHelper.GetChild(this, i);
                if (child != null)
                {
                    if (child is T t)
                    {
                        yield return t;
                    }
                    
                    foreach (var childOfChild in FindVisualChildrenInternal<T>(child))
                    {
                        yield return childOfChild;
                    }
                }
            }
        }
        
        private System.Collections.Generic.IEnumerable<T> FindVisualChildrenInternal<T>(DependencyObject parent) where T : DependencyObject
        {
            var count = VisualTreeHelper.GetChildrenCount(parent);
            for (int i = 0; i < count; i++)
            {
                var child = VisualTreeHelper.GetChild(parent, i);
                if (child != null)
                {
                    if (child is T t)
                    {
                        yield return t;
                    }
                    
                    foreach (var childOfChild in FindVisualChildrenInternal<T>(child))
                    {
                        yield return childOfChild;
                    }
                }
            }
        }

        //#pragma warning disable CS0162 // 检测到无法访问的代码
        //private void InitializeComponent()
        //{
        //    // 这是一个临时的空实现，以解决编译错误
        //    // 在正确配置的WPF项目中，这个方法应该由XAML编译器自动生成
        //}
        //#pragma warning restore CS0162 // 检测到无法访问的代码
    }
} 