using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using FatFullVersion.Entities;
using FatFullVersion.Entities.EntitiesEnum;
using FatFullVersion.Entities.ValueObject;
using FatFullVersion.IServices;
using Prism.Commands;
using Prism.Mvvm;

namespace FatFullVersion.ViewModels
{
    /// <summary>
    /// 配置编辑视图模型
    /// </summary>
    public class ConfigEditViewModel : BindableBase
    {
        private readonly IRepository _repository;
        
        /// <summary>
        /// 构造函数
        /// </summary>
        /// <param name="repository">仓储服务</param>
        public ConfigEditViewModel(IRepository repository)
        {
            _repository = repository;
            
            // 初始化命令
            InitializeCommand = new DelegateCommand(async () => await InitializeAsync());
            SaveTestPlcConfigCommand = new DelegateCommand(async () => await SaveTestPlcConfigAsync());
            SaveTargetPlcConfigCommand = new DelegateCommand(async () => await SaveTargetPlcConfigAsync());
            SaveComparisonTablesCommand = new DelegateCommand(async () => await SaveComparisonTablesAsync());
            AddComparisonTableCommand = new DelegateCommand(AddComparisonTable);
            DeleteComparisonTableCommand = new DelegateCommand<ComparisonTable>(DeleteComparisonTable);
            
            // 初始化通道类型列表
            ChannelTypes = new ObservableCollection<ChannelTypeItem>
            {
                new ChannelTypeItem { Value = TestPlcChannelType.AI, DisplayName = "模拟量输入有源" },
                new ChannelTypeItem { Value = TestPlcChannelType.AO, DisplayName = "模拟量输出有源" },
                new ChannelTypeItem { Value = TestPlcChannelType.DI, DisplayName = "数字量输入有源" },
                new ChannelTypeItem { Value = TestPlcChannelType.DO, DisplayName = "数字量输出有源" },
                new ChannelTypeItem { Value = TestPlcChannelType.AINone, DisplayName = "模拟量输入无源" },
                new ChannelTypeItem { Value = TestPlcChannelType.AONone, DisplayName = "模拟量输出无源" },
                new ChannelTypeItem { Value = TestPlcChannelType.DINone, DisplayName = "数字量输入无源" },
                new ChannelTypeItem { Value = TestPlcChannelType.DONone, DisplayName = "数字量输出无源" }
            };
            
            // 初始化
            InitializeCommand.Execute();
        }

        #region 属性

        /// <summary>
        /// 测试PLC配置
        /// </summary>
        private PlcConnectionConfig _testPlcConfig;
        public PlcConnectionConfig TestPlcConfig
        {
            get => _testPlcConfig;
            set => SetProperty(ref _testPlcConfig, value);
        }

        /// <summary>
        /// 被测PLC配置
        /// </summary>
        private PlcConnectionConfig _targetPlcConfig;
        public PlcConnectionConfig TargetPlcConfig
        {
            get => _targetPlcConfig;
            set => SetProperty(ref _targetPlcConfig, value);
        }

        /// <summary>
        /// 通道比较表集合
        /// </summary>
        private ObservableCollection<ComparisonTable> _comparisonTables;
        public ObservableCollection<ComparisonTable> ComparisonTables
        {
            get => _comparisonTables;
            set => SetProperty(ref _comparisonTables, value);
        }

        /// <summary>
        /// 可用的通道类型列表
        /// </summary>
        private ObservableCollection<ChannelTypeItem> _channelTypes;
        public ObservableCollection<ChannelTypeItem> ChannelTypes
        {
            get => _channelTypes;
            set => SetProperty(ref _channelTypes, value);
        }

        /// <summary>
        /// 当前选中的通道比较表项
        /// </summary>
        private ComparisonTable _selectedComparisonTable;
        public ComparisonTable SelectedComparisonTable
        {
            get => _selectedComparisonTable;
            set => SetProperty(ref _selectedComparisonTable, value);
        }

        /// <summary>
        /// 是否正在加载数据
        /// </summary>
        private bool _isLoading;
        public bool IsLoading
        {
            get => _isLoading;
            set => SetProperty(ref _isLoading, value);
        }

        #endregion

        #region 命令

        /// <summary>
        /// 初始化命令
        /// </summary>
        public DelegateCommand InitializeCommand { get; }

        /// <summary>
        /// 保存测试PLC配置命令
        /// </summary>
        public DelegateCommand SaveTestPlcConfigCommand { get; }

        /// <summary>
        /// 保存被测PLC配置命令
        /// </summary>
        public DelegateCommand SaveTargetPlcConfigCommand { get; }

        /// <summary>
        /// 保存通道比较表命令
        /// </summary>
        public DelegateCommand SaveComparisonTablesCommand { get; }

        /// <summary>
        /// 添加通道比较表项命令
        /// </summary>
        public DelegateCommand AddComparisonTableCommand { get; }

        /// <summary>
        /// 删除通道比较表项命令
        /// </summary>
        public DelegateCommand<ComparisonTable> DeleteComparisonTableCommand { get; }

        #endregion

        #region 方法

        /// <summary>
        /// 初始化
        /// </summary>
        private async Task InitializeAsync()
        {
            try
            {
                IsLoading = true;

                // 初始化数据库
                await _repository.InitializeDatabaseAsync();

                // 加载PLC配置
                TestPlcConfig = await _repository.GetTestPlcConnectionConfigAsync();
                TargetPlcConfig = await _repository.GetTargetPlcConnectionConfigAsync();

                // 加载通道比较表
                var tables = await _repository.GetComparisonTablesAsync();
                ComparisonTables = new ObservableCollection<ComparisonTable>(tables);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"初始化失败: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
            }
            finally
            {
                IsLoading = false;
            }
        }

        /// <summary>
        /// 保存测试PLC配置
        /// </summary>
        private async Task SaveTestPlcConfigAsync()
        {
            if (TestPlcConfig == null)
                return;

            try
            {
                IsLoading = true;
                
                // 确保标记为测试PLC
                TestPlcConfig.IsTestPlc = true;
                
                // 保存配置
                bool result = await _repository.SavePlcConnectionConfigAsync(TestPlcConfig);
                await Task.Delay(1000);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"保存测试PLC配置失败: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
            }
            finally
            {
                IsLoading = false;
            }
        }

        /// <summary>
        /// 保存被测PLC配置
        /// </summary>
        private async Task SaveTargetPlcConfigAsync()
        {
            if (TargetPlcConfig == null)
                return;

            try
            {
                IsLoading = true;
                
                // 确保标记为非测试PLC
                TargetPlcConfig.IsTestPlc = false;
                
                // 保存配置
                bool result = await _repository.SavePlcConnectionConfigAsync(TargetPlcConfig);
                await Task.Delay(1000);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"保存被测PLC配置失败: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
            }
            finally
            {
                IsLoading = false;
            }
        }

        /// <summary>
        /// 保存通道比较表
        /// </summary>
        private async Task SaveComparisonTablesAsync()
        {
            if (ComparisonTables == null)
                return;

            try
            {
                IsLoading = true;
                
                // 保存所有通道比较表项
                bool result = await _repository.SaveAllComparisonTablesAsync(ComparisonTables.ToList());
                await Task.Delay(1000);
                //if (result)
                //{
                //    MessageBox.Show("通道比较表保存成功！", "成功", MessageBoxButton.OK, MessageBoxImage.Information);
                    
                //    // 重新加载数据
                //    var tables = await _repository.GetComparisonTablesAsync();
                //    ComparisonTables = new ObservableCollection<ComparisonTable>(tables);
                //}
                //else
                //{
                //    MessageBox.Show("通道比较表保存失败！", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                //}
            }
            catch (Exception ex)
            {
                MessageBox.Show($"保存通道比较表失败: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
            }
            finally
            {
                IsLoading = false;
            }
        }

        /// <summary>
        /// 添加通道比较表项
        /// </summary>
        private void AddComparisonTable()
        {
            // 创建新的通道比较表项
            var newItem = new ComparisonTable
            {
                ChannelAddress = $"New_{DateTime.Now.Ticks % 1000}",
                CommunicationAddress = "00000",
                ChannelType = TestPlcChannelType.DI
            };
            
            // 添加到集合
            ComparisonTables.Add(newItem);
            
            // 选中新添加的项
            SelectedComparisonTable = newItem;
        }

        /// <summary>
        /// 删除通道比较表项
        /// </summary>
        /// <param name="item">要删除的项</param>
        private void DeleteComparisonTable(ComparisonTable item)
        {
            if (item == null)
                return;

            // 确认删除
            var result = MessageBox.Show($"确定要删除通道 {item.ChannelAddress} 吗？", "确认删除", 
                MessageBoxButton.YesNo, MessageBoxImage.Question);
            
            if (result == MessageBoxResult.Yes)
            {
                // 从集合中移除
                ComparisonTables.Remove(item);
            }
        }

        #endregion
    }

    /// <summary>
    /// 通道类型项
    /// </summary>
    public class ChannelTypeItem
    {
        /// <summary>
        /// 通道类型值
        /// </summary>
        public TestPlcChannelType Value { get; set; }

        /// <summary>
        /// 显示名称
        /// </summary>
        public string DisplayName { get; set; }
    }
}
