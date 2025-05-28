using Prism.Mvvm;
using System.Collections.ObjectModel;
using Prism.Commands;
using System;
using System.Windows.Media;
using System.Linq;
using System.Windows.Data;
using System.Globalization;
using System.Windows;
using FatFullVersion.Models;
using FatFullVersion.IServices;
using System.Collections.Generic;
using System.Threading.Tasks;
using System.Windows.Threading;
using FatFullVersion.Entities;
using FatFullVersion.Shared.Converters;
using FatFullVersion.Views;
using FatFullVersion.Services.Interfaces;
using Prism.Events;
using FatFullVersion.Events; // <-- 添加 using
using FatFullVersion.Shared;

namespace FatFullVersion.ViewModels
{
    public class DataEditViewModel : BindableBase
    {
        #region 属性和字段

        private readonly IPointDataService _pointDataService;
        private readonly IChannelMappingService _channelMappingService;
        private readonly ITestTaskManager _testTaskManager;
        private readonly IEventAggregator _eventAggregator;
        private readonly IPlcCommunication _testPlc;
        private readonly IPlcCommunication _targetPlc;
        private readonly IMessageService _messageService;
        private readonly ITestResultExportService _testResultExportService;
        private readonly ITestRecordService _testRecordService;
        private readonly IChannelStateManager _channelStateManager;
        private readonly IManualTestIoService _manualTestIoService;

        private string _message;

        public string Message
        {
            get { return _message; }
            set { SetProperty(ref _message, value); }
        }

        /// <summary>
        /// 测试PLC通信实例，用于UI绑定
        /// </summary>
        public IPlcCommunication TestPlc => _testPlc;

        /// <summary>
        /// 被测PLC通信实例，用于UI绑定
        /// </summary>
        public IPlcCommunication TargetPlc => _targetPlc;

        // 当前状态消息
        private string _statusMessage;

        public string StatusMessage
        {
            get { return _statusMessage; }
            set { SetProperty(ref _statusMessage, value); }
        }

        // 加载状态
        private bool _isLoading;

        public bool IsLoading
        {
            get { return _isLoading; }
            set { SetProperty(ref _isLoading, value); }
        }

        // 当前选择的通道类型（用于统一DataGrid）
        private string _selectedChannelType;

        public string SelectedChannelType
        {
            get { return _selectedChannelType; }
            set
            {
                if (SetProperty(ref _selectedChannelType, value))
                {
                    UpdateCurrentChannels();
                }
            }
        }

        /// <summary>
        /// 被测PLC通信实例
        /// </summary>
        protected readonly IPlcCommunication TargetPlcCommunication;

        // 当前显示的通道列表（用于UI展示）
        private ObservableCollection<ChannelMapping> _currentChannels;

        public ObservableCollection<ChannelMapping> CurrentChannels
        {
            get { return _currentChannels; }
            private set { SetProperty(ref _currentChannels, value); }
        }

        // 所有通道的数据源
        private ObservableCollection<ChannelMapping> _allChannels;

        /// <summary>
        /// 所有通道数据的主数据源
        /// </summary>
        public ObservableCollection<ChannelMapping> AllChannels
        {
            get { return _allChannels; }
            set
            {
                if (SetProperty(ref _allChannels, value))
                {
                    // 当AllChannels更新时，同步更新当前显示的通道
                    UpdateCurrentChannels();
                }
            }
        }

        /// <summary>
        /// 原始通道数据（用于批次选择）
        /// </summary>
        private ObservableCollection<ChannelMapping> _originalAllChannels;

        private ObservableCollection<ChannelMapping> OriginalAllChannels
        {
            get { return _originalAllChannels; }
            set { SetProperty(ref _originalAllChannels, value); }
        }

        /// <summary>
        /// 获取所有AI类型的通道
        /// </summary>
        public IEnumerable<ChannelMapping> GetAIChannels() => AllChannels?.Where(c => c.ModuleType?.ToUpper() == "AI");

        /// <summary>
        /// 获取所有AO类型的通道
        /// </summary>
        public IEnumerable<ChannelMapping> GetAOChannels() => AllChannels?.Where(c => c.ModuleType?.ToUpper() == "AO");

        /// <summary>
        /// 获取所有DI类型的通道
        /// </summary>
        public IEnumerable<ChannelMapping> GetDIChannels() => AllChannels?.Where(c => c.ModuleType?.ToUpper() == "DI");

        /// <summary>
        /// 获取所有DO类型的通道
        /// </summary>
        public IEnumerable<ChannelMapping> GetDOChannels() => AllChannels?.Where(c => c.ModuleType?.ToUpper() == "DO");

        /// <summary>
        /// 获取所有AI无源类型的通道
        /// </summary>
        public IEnumerable<ChannelMapping> GetAINoneChannels() => AllChannels?.Where(c => c.ModuleType?.ToUpper() == "AINONE");

        /// <summary>
        /// 获取所有AO无源类型的通道
        /// </summary>
        public IEnumerable<ChannelMapping> GetAONoneChannels() => AllChannels?.Where(c => c.ModuleType?.ToUpper() == "AONONE");

        /// <summary>
        /// 获取所有DI无源类型的通道
        /// </summary>
        public IEnumerable<ChannelMapping> GetDINoneChannels() => AllChannels?.Where(c => c.ModuleType?.ToUpper() == "DINONE");

        /// <summary>
        /// 获取所有DO无源类型的通道
        /// </summary>
        public IEnumerable<ChannelMapping> GetDONoneChannels() => AllChannels?.Where(c => c.ModuleType?.ToUpper() == "DONONE");

        /// <summary>
        /// 获取所有AI无源类型的通道
        /// </summary>
        // 测试结果数据
        //private ObservableCollection<ChannelMapping> _testResults;
        //public ObservableCollection<ChannelMapping> TestResults
        //{
        //    get => _testResults;
        //    set => SetProperty(ref _testResults, value);
        //}

        private ChannelMapping _currentTestResult;

        /// <summary>
        /// 当前测试结果
        /// </summary>
        public ChannelMapping CurrentTestResult
        {
            get { return _currentTestResult; }
            set
            {
                if (_currentTestResult != value)
                {
                    _currentTestResult = value;
                    //System.Diagnostics.Debug.WriteLine(
                    //    $"CurrentTestResult changed: {_currentTestResult?.VariableName}, HardPointTestResult: {_currentTestResult?.HardPointTestResult}");
                    RaisePropertyChanged(nameof(CurrentTestResult));
                }
            }
        }

        // 批次相关数据
        private ObservableCollection<BatchInfo> _batches;

        public ObservableCollection<BatchInfo> Batches
        {
            get { return _batches; }
            set { SetProperty(ref _batches, value); }
        }

        //选择的当前批次信息
        private BatchInfo _selectedBatch;

        public BatchInfo SelectedBatch
        {
            get => _selectedBatch;
            set
            {
                SetProperty(ref _selectedBatch, value);
                OnBatchSelected();
            }
        }

        private bool _isBatchSelectionOpen;

        public bool IsBatchSelectionOpen
        {
            get { return _isBatchSelectionOpen; }
            set { SetProperty(ref _isBatchSelectionOpen, value); }
        }

        private string _selectedResultFilter;

        public string SelectedResultFilter
        {
            get { return _selectedResultFilter; }
            set
            {
                if (SetProperty(ref _selectedResultFilter, value))
                {
                    // 应用结果过滤
                    ApplyResultFilter();
                }
            }
        }

        // 点位统计数据
        private string _totalPointCount;

        public string TotalPointCount
        {
            get { return _totalPointCount; }
            set { SetProperty(ref _totalPointCount, value); }
        }

        private string _testedPointCount;

        public string TestedPointCount
        {
            get { return _testedPointCount; }
            set { SetProperty(ref _testedPointCount, value); }
        }

        private string _waitingPointCount;

        public string WaitingPointCount
        {
            get { return _waitingPointCount; }
            set { SetProperty(ref _waitingPointCount, value); }
        }

        private string _successPointCount;

        public string SuccessPointCount
        {
            get { return _successPointCount; }
            set { SetProperty(ref _successPointCount, value); }
        }

        private string _failurePointCount;

        public string FailurePointCount
        {
            get { return _failurePointCount; }
            set { SetProperty(ref _failurePointCount, value); }
        }

        // 测试队列相关属性
        private ObservableCollection<ChannelMapping> _testQueue;

        /// <summary>
        /// 测试队列，存储待测试的通道列表
        /// </summary>
        public ObservableCollection<ChannelMapping> TestQueue
        {
            get { return _testQueue; }
            set { SetProperty(ref _testQueue, value); }
        }

        private int _testQueuePosition;

        /// <summary>
        /// 当前测试队列位置
        /// </summary>
        public int TestQueuePosition
        {
            get { return _testQueuePosition; }
            set { SetProperty(ref _testQueuePosition, value); }
        }

        private string _testQueueStatus;

        /// <summary>
        /// 测试队列状态描述
        /// </summary>
        public string TestQueueStatus
        {
            get { return _testQueueStatus; }
            set { SetProperty(ref _testQueueStatus, value); }
        }

        private ChannelMapping _currentQueueItem;

        /// <summary>
        /// 当前队列中的测试项
        /// </summary>
        public ChannelMapping CurrentQueueItem
        {
            get { return _currentQueueItem; }
            set { SetProperty(ref _currentQueueItem, value); }
        }

        // 命令
        public DelegateCommand ImportConfigCommand { get; private set; }
        public DelegateCommand RestoreConfigCommand { get; private set; }
        public DelegateCommand SelectBatchCommand { get; private set; }
        public DelegateCommand ExportChannelMapCommand { get; private set; }
        public DelegateCommand SkipModuleCommand { get; private set; }
        public DelegateCommand FinishWiringCommand { get; private set; }
        public DelegateCommand StartTestCommand { get; private set; }
        public DelegateCommand<ChannelMapping> RetestCommand { get; private set; }
        public DelegateCommand ExportTestResultsCommand { get; private set; }
        public DelegateCommand ConfirmBatchSelectionCommand { get; private set; }
        public DelegateCommand CancelBatchSelectionCommand { get; private set; }
        public DelegateCommand AllocateChannelsCommand { get; private set; }
        public DelegateCommand ClearChannelAllocationsCommand { get; private set; }

        // 命令
        public DelegateCommand<ChannelMapping> OpenAIManualTestCommand { get; private set; }
        public DelegateCommand<ChannelMapping> OpenDIManualTestCommand { get; private set; }
        public DelegateCommand<ChannelMapping> OpenDOManualTestCommand { get; private set; }
        public DelegateCommand<ChannelMapping> OpenAOManualTestCommand { get; private set; }
        public DelegateCommand CloseAIManualTestCommand { get; private set; }
        public DelegateCommand CloseDIManualTestCommand { get; private set; }
        public DelegateCommand CloseDOManualTestCommand { get; private set; }
        public DelegateCommand CloseAOManualTestCommand { get; private set; }

        /// <summary>
        /// 发送AI测试值命令
        /// </summary>
        public DelegateCommand<ChannelMapping> SendAITestValueCommand { get; private set; }

        /// <summary>
        /// 确认AI值命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ConfirmAIValueCommand { get; private set; }

        /// <summary>
        /// 发送AI高报命令
        /// </summary>
        public DelegateCommand<ChannelMapping> SendAIHighAlarmCommand { get; private set; }
        /// <summary>
        /// 发送AI高高报命令
        /// </summary>
        public DelegateCommand<ChannelMapping> SendAIHighHighAlarmCommand { get; private set; }

        /// <summary>
        /// 复位AI高报命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ResetAIHighAlarmCommand { get; private set; }

        /// <summary>
        /// 确认AI高报命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ConfirmAIHighAlarmCommand { get; private set; }

        /// <summary>
        /// 发送AI低报命令
        /// </summary>
        public DelegateCommand<ChannelMapping> SendAILowAlarmCommand { get; private set; }

        /// <summary>
        /// 发送AI低低报命令
        /// </summary>
        public DelegateCommand<ChannelMapping> SendAILowLowAlarmCommand { get; private set; }

        /// <summary>
        /// 复位AI低报命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ResetAILowAlarmCommand { get; private set; }

        /// <summary>
        /// 确认AI低报命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ConfirmAILowAlarmCommand { get; private set; }

        /// <summary>
        /// 确认AI报警值设定命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ConfirmAIAlarmValueSetCommand { get; private set; }

        /// <summary>
        /// 发送AI维护功能命令
        /// </summary>
        public DelegateCommand<ChannelMapping> SendAIMaintenanceCommand { get; private set; }

        /// <summary>
        /// 复位AI维护功能命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ResetAIMaintenanceCommand { get; private set; }

        /// <summary>
        /// 确认AI维护功能命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ConfirmAIMaintenanceCommand { get; private set; }

        /// <summary>
        /// 确认AI趋势检查命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ConfirmAITrendCheckCommand { get; private set; }

        /// <summary>
        /// 确认AI报表检查命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ConfirmAIReportCheckCommand { get; private set; }

        /// <summary>
        /// 确认AO趋势检查命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ConfirmAOTrendCheckCommand { get; private set; }

        /// <summary>
        /// 确认AO报表检查命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ConfirmAOReportCheckCommand { get; private set; }

        /// <summary>
        /// 发送DI测试命令
        /// </summary>
        public DelegateCommand<ChannelMapping> SendDITestCommand { get; private set; }

        /// <summary>
        /// 复位DI命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ResetDICommand { get; private set; }

        /// <summary>
        /// 确认DI命令
        /// </summary>
        public DelegateCommand<ChannelMapping> ConfirmDICommand { get; private set; }

        private bool _isDOManualTestOpen;

        public bool IsDOManualTestOpen
        {
            get => _isDOManualTestOpen;
            set => SetProperty(ref _isDOManualTestOpen, value);
        }

        private bool _isAOManualTestOpen;

        public bool IsAOManualTestOpen
        {
            get => _isAOManualTestOpen;
            set => SetProperty(ref _isAOManualTestOpen, value);
        }

        // 监测状态
        private string _diMonitorStatus = "开始监测";

        public string DIMonitorStatus
        {
            get { return _diMonitorStatus; }
            set { SetProperty(ref _diMonitorStatus, value); }
        }

        private string _doMonitorStatus = "开始监测";

        public string DOMonitorStatus
        {
            get { return _doMonitorStatus; }
            set { SetProperty(ref _doMonitorStatus, value); }
        }

        private string _aoMonitorStatus = "开始监测";

        public string AOMonitorStatus
        {
            get { return _aoMonitorStatus; }
            set { SetProperty(ref _aoMonitorStatus, value); }
        }

        // 当前值
        private string _diCurrentValue;

        public string DICurrentValue
        {
            get { return _diCurrentValue; }
            set { SetProperty(ref _diCurrentValue, value); }
        }

        private string _doCurrentValue;

        public string DOCurrentValue
        {
            get { return _doCurrentValue; }
            set { SetProperty(ref _doCurrentValue, value); }
        }

        private string _aoCurrentValue;

        public string AOCurrentValue
        {
            get { return _aoCurrentValue; }
            set { SetProperty(ref _aoCurrentValue, value); }
        }

        // 命令
        public DelegateCommand<ChannelMapping> StartDOMonitorCommand { get; private set; }
        public DelegateCommand<ChannelMapping> ConfirmDOCommand { get; private set; }

        public DelegateCommand<ChannelMapping> StartAOMonitorCommand { get; private set; }
        //public DelegateCommand<ChannelMapping> SaveAO0Command { get; private set; }
        //public DelegateCommand<ChannelMapping> SaveAO25Command { get; private set; }
        //public DelegateCommand<ChannelMapping> SaveAO50Command { get; private set; }
        //public DelegateCommand<ChannelMapping> SaveAO75Command { get; private set; }
        //public DelegateCommand<ChannelMapping> SaveAO100Command { get; private set; }
        public DelegateCommand<ChannelMapping> ConfirmAOCommand { get; private set; }

        private ChannelMapping _currentChannel;

        public ChannelMapping CurrentChannel
        {
            get => _currentChannel;
            set => SetProperty(ref _currentChannel, value);
        }

        /// <summary>
        /// AI手动测试窗口是否打开
        /// </summary>
        private bool _isAIManualTestOpen;

        public bool IsAIManualTestOpen
        {
            get => _isAIManualTestOpen;
            set => SetProperty(ref _isAIManualTestOpen, value);
        }

        /// <summary>
        /// DI手动测试窗口是否打开
        /// </summary>
        private bool _isDIManualTestOpen;

        public bool IsDIManualTestOpen
        {
            get => _isDIManualTestOpen;
            set => SetProperty(ref _isDIManualTestOpen, value);
        }

        /// <summary>
        /// AI设定值
        /// </summary>
        private string _aiSetValue;

        public string AISetValue
        {
            get => _aiSetValue;
            set => SetProperty(ref _aiSetValue, value);
        }

        /// <summary>
        /// AI低报设定值
        /// </summary>
        private string _aILowSetValue;

        public string AILowSetValue
        {
            get => _aILowSetValue;
            set => SetProperty(ref _aILowSetValue, value);
        }

        /// <summary>
        /// AI低低报设定值
        /// </summary>
        private string _aILowLowSetValue;

        public string AILowLowSetValue
        {
            get => _aILowLowSetValue;
            set => SetProperty(ref _aILowLowSetValue, value);
        }

        /// <summary>
        /// AI高报设定值
        /// </summary>
        private string _aIHighSetValue;

        public string AIHighSetValue
        {
            get => _aIHighSetValue;
            set => SetProperty(ref _aIHighSetValue, value);
        }

        /// <summary>
        /// AI高高报设定值
        /// </summary>
        private string _aIHighHighSetValue;

        public string AIHighHighSetValue
        {
            get => _aIHighHighSetValue;
            set => SetProperty(ref _aIHighHighSetValue, value);
        }

        /// <summary>
        /// 当前选中的通道
        /// </summary>
        private ChannelMapping _selectedChannel;

        public ChannelMapping SelectedChannel
        {
            get => _selectedChannel;
            set => SetProperty(ref _selectedChannel, value);
        }

        // 添加字段和属性
        private bool _isWiringCompleteBtnEnabled = true;

        /// <summary>
        /// 表示"完成接线确认"按钮是否可用
        /// </summary>
        public bool IsWiringCompleteBtnEnabled
        {
            get => _isWiringCompleteBtnEnabled;
            set => SetProperty(ref _isWiringCompleteBtnEnabled, value);
        }

        // 添加命令属性
        private DelegateCommand _confirmWiringCompleteCommand;

        /// <summary>
        /// 确认接线完成命令
        /// </summary>
        public DelegateCommand ConfirmWiringCompleteCommand =>
            _confirmWiringCompleteCommand ??=
                new DelegateCommand(FinishWiring, CanExecuteConfirmWiringComplete); // 改为执行 FinishWiring

        // 添加控制通道硬点自动测试按钮的启用属性
        private bool _isStartTestButtonEnabled = false;

        /// <summary>
        /// 表示"通道硬点自动测试"按钮是否可用
        /// </summary>
        public bool IsStartTestButtonEnabled
        {
            get => _isStartTestButtonEnabled;
            set => SetProperty(ref _isStartTestButtonEnabled, value);
        }

        /// <summary>
        /// AI手动测试状态（专用于手动测试窗口，与硬点测试分离）
        /// </summary>
        private string _manualAITestStatus = "未测试";
        public string ManualAITestStatus
        {
            get => _manualAITestStatus;
            set => SetProperty(ref _manualAITestStatus, value);
        }

        /// <summary>
        /// DI手动测试状态（专用于手动测试窗口，与硬点测试分离）
        /// </summary>
        private string _manualDITestStatus = "未测试";
        public string ManualDITestStatus
        {
            get => _manualDITestStatus;
            set => SetProperty(ref _manualDITestStatus, value);
        }

        /// <summary>
        /// DO手动测试状态（专用于手动测试窗口，与硬点测试分离）
        /// </summary>
        private string _manualDOTestStatus = "未测试";
        public string ManualDOTestStatus
        {
            get => _manualDOTestStatus;
            set => SetProperty(ref _manualDOTestStatus, value);
        }

        /// <summary>
        /// AO手动测试状态（专用于手动测试窗口，与硬点测试分离）
        /// </summary>
        private string _manualAOTestStatus = "未测试";
        public string ManualAOTestStatus
        {
            get => _manualAOTestStatus;
            set => SetProperty(ref _manualAOTestStatus, value);
        }

        /// <summary>
        /// AI高报警手动测试状态
        /// </summary>
        private string _manualAIHighAlarmStatus = "未测试";
        public string ManualAIHighAlarmStatus
        {
            get => _manualAIHighAlarmStatus;
            set => SetProperty(ref _manualAIHighAlarmStatus, value);
        }

        /// <summary>
        /// AI低报警手动测试状态
        /// </summary>
        private string _manualAILowAlarmStatus = "未测试";
        public string ManualAILowAlarmStatus
        {
            get => _manualAILowAlarmStatus;
            set => SetProperty(ref _manualAILowAlarmStatus, value);
        }

        /// <summary>
        /// AI维护功能手动测试状态
        /// </summary>
        private string _manualAIMaintenanceStatus = "未测试";
        public string ManualAIMaintenanceStatus
        {
            get => _manualAIMaintenanceStatus;
            set => SetProperty(ref _manualAIMaintenanceStatus, value);
        }

        private bool _isHistoryRecordsOpen;
        /// <summary>
        /// 历史记录查看窗口是否打开
        /// </summary>
        public bool IsHistoryRecordsOpen
        {
            get { return _isHistoryRecordsOpen; }
            set { SetProperty(ref _isHistoryRecordsOpen, value); }
        }

        private ObservableCollection<TestBatchInfo> _testBatches;
        /// <summary>
        /// 历史测试批次列表
        /// </summary>
        public ObservableCollection<TestBatchInfo> TestBatches
        {
            get { return _testBatches; }
            set { SetProperty(ref _testBatches, value); }
        }

        private TestBatchInfo _selectedTestBatch;
        /// <summary>
        /// 当前选择的历史测试批次
        /// </summary>
        public TestBatchInfo SelectedTestBatch
        {
            get { return _selectedTestBatch; }
            set { SetProperty(ref _selectedTestBatch, value); }
        }

        // 历史记录相关命令
        public DelegateCommand RestoreTestRecordsCommand { get; private set; }
        public DelegateCommand DeleteTestBatchCommand { get; private set; }
        public DelegateCommand CloseHistoryRecordsCommand { get; private set; }

        // 添加模块跳过相关属性和命令
        private bool _isSkipModuleOpen;
        /// <summary>
        /// 模块跳过窗口是否打开
        /// </summary>
        public bool IsSkipModuleOpen
        {
            get { return _isSkipModuleOpen; }
            set { SetProperty(ref _isSkipModuleOpen, value); }
        }

        private ObservableCollection<ModuleInfo> _modules;
        /// <summary>
        /// 模块列表
        /// </summary>
        public ObservableCollection<ModuleInfo> Modules
        {
            get { return _modules; }
            set { SetProperty(ref _modules, value); }
        }

        private string _skipReason;
        /// <summary>
        /// 跳过原因
        /// </summary>
        public string SkipReason
        {
            get { return _skipReason; }
            set { SetProperty(ref _skipReason, value); }
        }

        // 初始化模块跳过相关命令
        public DelegateCommand ConfirmSkipModuleCommand { get; private set; }
        public DelegateCommand CancelSkipModuleCommand { get; private set; }

        private string _moduleSearchFilter;
        /// <summary>
        /// 模块搜索过滤条件
        /// </summary>
        public string ModuleSearchFilter
        {
            get { return _moduleSearchFilter; }
            set 
            { 
                if (SetProperty(ref _moduleSearchFilter, value))
                {
                    RaisePropertyChanged(nameof(FilteredModules));
                }
            }
        }

        /// <summary>
        /// 过滤后的模块列表
        /// </summary>
        public ObservableCollection<ModuleInfo> FilteredModules
        {
            get
            {
                if (Modules == null)
                    return new ObservableCollection<ModuleInfo>();

                if (string.IsNullOrWhiteSpace(ModuleSearchFilter))
                    return Modules;

                var filtered = Modules.Where(m => 
                    m.ModuleName.Contains(ModuleSearchFilter, StringComparison.OrdinalIgnoreCase) || 
                    m.ModuleType.Contains(ModuleSearchFilter, StringComparison.OrdinalIgnoreCase));
                
                return new ObservableCollection<ModuleInfo>(filtered);
            }
        }

        private bool _selectAllModules;
        /// <summary>
        /// 是否全选模块
        /// </summary>
        public bool SelectAllModules
        {
            get { return _selectAllModules; }
            set 
            { 
                if (SetProperty(ref _selectAllModules, value))
                {
                    // 更新所有模块的选中状态
                    if (Modules != null)
                    {
                        foreach (var module in Modules)
                        {
                            module.IsSelected = value;
                        }
                    } // Closes if (Modules != null)
                } // Closes if (SetProperty(ref _selectAllModules, value))
            }
        }

        /// <summary>
        /// 全选模块命令
        /// </summary>
        public DelegateCommand SelectAllModulesCommand { get; private set; }
        
        /// <summary>
        /// 取消全选模块命令
        /// </summary>
        public DelegateCommand UnselectAllModulesCommand { get; private set; }

        /// <summary>
        /// 全选所有模块
        /// </summary>
        private void ExecuteSelectAllModules()
        {
            if (Modules != null)
            {
                foreach (var module in Modules)
                {
                    module.IsSelected = true;
                }
                SelectAllModules = true;
            }
        }

        /// <summary>
        /// 取消全选所有模块
        /// </summary>
        private void ExecuteUnselectAllModules()
        {
            if (Modules != null)
            {
                foreach (var module in Modules)
                {
                    module.IsSelected = false;
                }
                SelectAllModules = false;
            }
        }

        public DelegateCommand<ChannelMapping> ShowErrorDetailCommand { get; private set; }

        #endregion

        #region 构造函数和初始化

        /// <summary>
        /// DataEditViewModel构造函数
        /// </summary>
        /// <param name="pointDataService">点位数据服务接口</param>
        /// <param name="channelMappingService">通道映射服务接口</param>
        /// <param name="testTaskManager">测试任务管理器接口</param>
        /// <param name="eventAggregator">事件聚合器</param>
        /// <param name="testPlc">测试PLC通信接口</param>
        /// <param name="targetPlc">目标PLC通信接口</param>
        /// <param name="testResultExportService">测试结果导出服务接口</param>
        /// <param name="testRecordService">测试记录服务接口</param>
        /// <param name="messageService">消息服务接口</param>
        public DataEditViewModel(
            IPointDataService pointDataService,
            IChannelMappingService channelMappingService,
            ITestTaskManager testTaskManager,
            IEventAggregator eventAggregator,
            IPlcCommunication testPlc,
            IPlcCommunication targetPlc,
            IMessageService messageService,
            ITestResultExportService testResultExportService,
            ITestRecordService testRecordService,
            IChannelStateManager channelStateManager,
            IManualTestIoService manualTestIoService
        )
        {
            _pointDataService = pointDataService;
            _channelMappingService = channelMappingService;
            _testTaskManager = testTaskManager;
            _eventAggregator = eventAggregator;
            _testPlc = testPlc ?? throw new ArgumentNullException(nameof(testPlc));
            _targetPlc = targetPlc ?? throw new ArgumentNullException(nameof(targetPlc));
            _testResultExportService = testResultExportService;
            _testRecordService = testRecordService ?? throw new ArgumentNullException(nameof(testRecordService));
            _messageService = messageService ?? throw new ArgumentNullException(nameof(messageService));
            _channelStateManager = channelStateManager;
            _manualTestIoService = manualTestIoService;

            // 初始化数据结构
            Initialize();

            ShowErrorDetailCommand = new DelegateCommand<ChannelMapping>(ExecuteShowErrorDetail);
        }

        /// <summary>
        /// 初始化ViewModel
        /// </summary>
        /// <remarks>
        /// 初始化所有命令、事件订阅和数据集合，设置默认状态
        /// </remarks>
        private void Initialize()
        {
            // 订阅测试结果更新事件
            // 在 DataEditViewModel.cs 的 Initialize() 方法内
            // ... (已有的 _eventAggregator.GetEvent<TestResultsUpdatedEvent>().Subscribe(OnTestResultsUpdated); 之后) ...
            _eventAggregator.GetEvent<ChannelStatesModifiedEvent>().Subscribe(OnChannelStatesModified); // <<<< 新增此行 >>>>

            // 初始化集合
            AllChannels = new ObservableCollection<ChannelMapping>();
            CurrentChannels = new ObservableCollection<ChannelMapping>();
            //TestResults = new ObservableCollection<ChannelMapping>();
            Batches = new ObservableCollection<BatchInfo>();
            TestQueue = new ObservableCollection<ChannelMapping>();
            Modules = new ObservableCollection<ModuleInfo>();

            // 初始化搜索过滤和选中状态
            ModuleSearchFilter = string.Empty;
            SelectAllModules = false;
            SkipReason = string.Empty;

            // 初始化测试队列相关属性
            TestQueuePosition = 0;
            TestQueueStatus = "队列为空";

            // 初始化其他属性
            SelectedChannelType = "AI通道";
            IsBatchSelectionOpen = false;
            SelectedResultFilter = "全部";
            TotalPointCount = "0";
            TestedPointCount = "0";
            WaitingPointCount = "0";
            SuccessPointCount = "0";
            FailurePointCount = "0";

            // 初始化按钮状态
            IsStartTestButtonEnabled = false;

            // 初始化命令
            ImportConfigCommand = new DelegateCommand(ImportConfig);
            SelectBatchCommand = new DelegateCommand(ExecuteSelectBatch);
            ExportChannelMapCommand = new DelegateCommand(ExportChannelMap);
            SkipModuleCommand = new DelegateCommand(SkipModule);
            FinishWiringCommand = new DelegateCommand(FinishWiring);
            StartTestCommand = new DelegateCommand(StartTest);
            RetestCommand = new DelegateCommand<ChannelMapping>(Retest);
            ConfirmBatchSelectionCommand = new DelegateCommand(ConfirmBatchSelection);
            CancelBatchSelectionCommand = new DelegateCommand(CancelBatchSelection);
            AllocateChannelsCommand = new DelegateCommand(ExecuteAllocateChannels);
            ClearChannelAllocationsCommand = new DelegateCommand(ClearChannelAllocationsAsync);
            ExportTestResultsCommand = new DelegateCommand(ExportTestResults, CanExportTestResults);

            // 初始化手动测试相关命令
            OpenAIManualTestCommand = new DelegateCommand<ChannelMapping>(OpenAIManualTest);
            OpenDIManualTestCommand = new DelegateCommand<ChannelMapping>(OpenDIManualTest);
            OpenDOManualTestCommand = new DelegateCommand<ChannelMapping>(OpenDOManualTest);
            OpenAOManualTestCommand = new DelegateCommand<ChannelMapping>(OpenAOManualTest);
            CloseAIManualTestCommand = new DelegateCommand(ExecuteCloseAIManualTest);
            CloseDIManualTestCommand = new DelegateCommand(ExecuteCloseDIManualTest);
            CloseDOManualTestCommand = new DelegateCommand(ExecuteCloseDOManualTest);
            CloseAOManualTestCommand = new DelegateCommand(ExecuteCloseAOManualTest);


            // AI手动测试命令
            SendAITestValueCommand = new DelegateCommand<ChannelMapping>(ExecuteSendAITestValue);
            ConfirmAIValueCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmAIValue);
            SendAIHighAlarmCommand = new DelegateCommand<ChannelMapping>(ExecuteSendAIHighAlarm);
            SendAIHighHighAlarmCommand = new DelegateCommand<ChannelMapping>(ExecuteSendAIHighHighAlarm);
            ResetAIHighAlarmCommand = new DelegateCommand<ChannelMapping>(ExecuteResetAIHighAlarm);
            ConfirmAIHighAlarmCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmAIHighAlarm);
            SendAILowAlarmCommand = new DelegateCommand<ChannelMapping>(ExecuteSendAILowAlarm);
            SendAILowLowAlarmCommand = new DelegateCommand<ChannelMapping>(ExecuteSendAILowLowAlarm);
            ResetAILowAlarmCommand = new DelegateCommand<ChannelMapping>(ExecuteResetAILowAlarm);
            ConfirmAILowAlarmCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmAILowAlarm);
            ConfirmAIAlarmValueSetCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmAIAlarmValueSet);
            SendAIMaintenanceCommand = new DelegateCommand<ChannelMapping>(ExecuteSendAIMaintenance);
            ResetAIMaintenanceCommand = new DelegateCommand<ChannelMapping>(ExecuteResetAIMaintenance);
            ConfirmAIMaintenanceCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmAIMaintenance);

            // 添加新的命令初始化
            ConfirmAITrendCheckCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmAITrendCheck);
            ConfirmAIReportCheckCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmAIReportCheck);
            ConfirmAOTrendCheckCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmAOTrendCheck); // 新增AO趋势检查命令初始化
            ConfirmAOReportCheckCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmAOReportCheck); // 新增AO报表检查命令初始化


            // DI手动测试命令
            SendDITestCommand = new DelegateCommand<ChannelMapping>(ExecuteSendDITest);
            ResetDICommand = new DelegateCommand<ChannelMapping>(ExecuteResetDI);
            ConfirmDICommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmDI);

            // DO手动测试命令
            StartDOMonitorCommand = new DelegateCommand<ChannelMapping>(ExecuteStartDOMonitor);
            ConfirmDOCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmDO);

            // AO手动测试命令
            StartAOMonitorCommand = new DelegateCommand<ChannelMapping>(ExecuteStartAOMonitor);
            //SaveAO0Command = new DelegateCommand<ChannelMapping>(ExecuteSaveAO0);
            //SaveAO25Command = new DelegateCommand<ChannelMapping>(ExecuteSaveAO25);
            //SaveAO50Command = new DelegateCommand<ChannelMapping>(ExecuteSaveAO50);
            //SaveAO75Command = new DelegateCommand<ChannelMapping>(ExecuteSaveAO75);
            //SaveAO100Command = new DelegateCommand<ChannelMapping>(ExecuteSaveAO100);
            ConfirmAOCommand = new DelegateCommand<ChannelMapping>(ExecuteConfirmAO);

            // 尝试从通道映射信息中提取批次信息
            InitializeBatchData();

            // 历史记录相关命令
            RestoreConfigCommand = new DelegateCommand(RestoreConfig);
            RestoreTestRecordsCommand = new DelegateCommand(RestoreTestRecords);
            DeleteTestBatchCommand = new DelegateCommand(DeleteTestBatch);
            CloseHistoryRecordsCommand = new DelegateCommand(CloseHistoryRecords);

            // 初始化模块跳过相关命令
            ConfirmSkipModuleCommand = new DelegateCommand(ConfirmSkipModule);
            CancelSkipModuleCommand = new DelegateCommand(CancelSkipModule);

            // 初始化模块全选/取消全选命令
            SelectAllModulesCommand = new DelegateCommand(ExecuteSelectAllModules);
            UnselectAllModulesCommand = new DelegateCommand(ExecuteUnselectAllModules);
        }

                // <<<< 新增事件处理器 >>>>
        private void OnChannelStatesModified(List<Guid> modifiedChannelIds)
        {
            if (modifiedChannelIds == null || !modifiedChannelIds.Any() || AllChannels == null) return;

            // 确保在UI线程执行UI更新和依赖属性的刷新
            Application.Current.Dispatcher.Invoke(() =>
            {
                bool currentChannelsPotentiallyAffected = false;
                foreach (Guid id in modifiedChannelIds)
                {
                    // 检查是否有当前显示的通道受到了影响
                    if (CurrentChannels.Any(cc => cc.Id == id))
                    {
                        currentChannelsPotentiallyAffected = true;
                        break; 
                    }
                }

                if (currentChannelsPotentiallyAffected)
                {
                    UpdateCurrentChannels();
                    System.Diagnostics.Debug.WriteLine($"UI事件：OnChannelStatesModified - CurrentChannels 已通过 UpdateCurrentChannels() 刷新，因为受影响的通道在其中。");
                }
                else
                {
                    System.Diagnostics.Debug.WriteLine($"UI事件：OnChannelStatesModified - 受影响的通道不在CurrentChannels中，未主动刷新CurrentChannels。");
                }

                // 更新依赖于通道状态的ViewModel聚合属性和命令状态
                UpdatePointStatistics(); 
                RefreshBatchStatus();    
                
                // 重新评估 StartTestButton 的可用性
                if (SelectedBatch != null)
                {
                    IsStartTestButtonEnabled = SelectedBatch.Status == "接线已确认" &&
                                               AllChannels.Any(c => c.TestBatch == SelectedBatch.BatchName && c.HardPointTestResult == "等待测试");
                    System.Diagnostics.Debug.WriteLine($"OnChannelStatesModified: IsStartTestButtonEnabled 更新为 {IsStartTestButtonEnabled}。批次状态: {SelectedBatch.Status}, 等待测试通道数: {AllChannels.Count(c => c.TestBatch == SelectedBatch.BatchName && c.HardPointTestResult == "等待测试")}");
                }
                else
                {
                    IsStartTestButtonEnabled = false;
                    System.Diagnostics.Debug.WriteLine($"OnChannelStatesModified: IsStartTestButtonEnabled 更新为 false (无选中批次)。");
                }
                // 确保命令的CanExecute状态得到通知，以便UI上的按钮能够正确更新其可用性
                // 假设 StartTestCommand 是 DelegateCommand 类型
                (StartTestCommand as DelegateCommand)?.RaiseCanExecuteChanged();


                ExportTestResultsCommand.RaiseCanExecuteChanged(); 
                RetestCommand.RaiseCanExecuteChanged(); 
            });
        }
        #endregion

        #region 数据加载和处理

        /// <summary>
        /// 导入配置
        /// </summary>
        /// <remarks>
        /// 打开文件对话框并导入Excel配置数据，然后处理导入的数据
        /// </remarks>
        private async void ImportConfig()
        {
            // 防止重复点击
            IsLoading = true;
            StatusMessage = "正在导入Excel点表配置文件...";

            try
            {
                // 导入点表配置，并处理导入的数据
                if (_pointDataService != null)
                {
                    // 使用TaskCompletionSource来处理回调形式的异步方法
                    var tcs = new TaskCompletionSource<IEnumerable<ExcelPointData>>();

                    await _pointDataService.ImportPointConfigurationAsync(data =>
                    {
                        try
                        {
                            tcs.SetResult(data);
                        }
                        catch (Exception ex)
                        {
                            tcs.SetException(ex);
                        }
                    });

                    // 等待异步操作完成并获取结果
                    var importedData = await tcs.Task;

                    // 清空原始通道集合引用
                    OriginalAllChannels = null;

                    // 处理导入的数据
                    if (importedData != null)
                    {
                        await ProcessImportedDataAsync(importedData);
                    }
                }
                else
                {
                    await _messageService.ShowAsync("导入失败", "无法获取点表数据服务", MessageBoxButton.OK);
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("导入失败", $"导入配置失败: {ex.Message}", MessageBoxButton.OK);
            }
            finally
            {
                IsLoading = false;
                StatusMessage = string.Empty;
            }
        }

        /// <summary>
        /// 处理导入的数据
        /// </summary>
        /// <param name="importedData">从Excel导入的点位数据集合</param>
        /// <returns>处理完成的任务</returns>
        /// <remarks>
        /// 将Excel数据转换为通道映射对象，应用业务规则，并更新UI
        /// </remarks>
        private async Task ProcessImportedDataAsync(IEnumerable<ExcelPointData> importedData)
        {
            if (importedData == null || !importedData.Any())
            {
                await _messageService.ShowAsync("提示", "没有导入任何数据。", MessageBoxButton.OK);
                return;
            }

            try
            {
                IsLoading = true;
                StatusMessage = "正在处理导入数据并初始化通道状态...";

                // 使用 ChannelMappingService 来创建和初始化 ChannelMapping 对象
                // ChannelMappingService.CreateAndInitializeChannelMappingsAsync 内部会调用 IChannelStateManager.InitializeChannelFromImport
                var initializedChannels = await _channelMappingService.CreateAndInitializeChannelMappingsAsync(importedData);

                if (initializedChannels != null)
                {
                    AllChannels = new ObservableCollection<ChannelMapping>(initializedChannels.OrderBy(c => c.TestId)); 

                    // 导入完成后立即进行通道自动分配，保证 TestBatch 在首次显示时就已填充
                    AllChannels = await _channelMappingService.AllocateChannelsTestAsync(AllChannels);

                    OriginalAllChannels = new ObservableCollection<ChannelMapping>(AllChannels); // 更新原始备份

                    // UI 更新
                    UpdateCurrentChannels(); 
                    RaisePropertyChanged(nameof(AllChannels));
                    UpdatePointStatistics();
                    await RefreshBatchesFromChannelsAsync(); 
                    if (Batches != null && Batches.Any())
                    {
                        SelectedBatch = Batches.FirstOrDefault();
                    }
                    else
                    {
                        SelectedBatch = null;
                    }
                    IsWiringCompleteBtnEnabled = true; 

                    await _messageService.ShowAsync("成功", $"已成功处理和初始化 {AllChannels.Count} 个通道。", MessageBoxButton.OK);
                }
                else
                {
                    AllChannels.Clear(); // 如果服务未返回任何内容，则清空
                    OriginalAllChannels.Clear();
                    UpdateCurrentChannels();
                    RaisePropertyChanged(nameof(AllChannels));
                    UpdatePointStatistics();
                    await RefreshBatchesFromChannelsAsync();
                    SelectedBatch = null;
                    await _messageService.ShowAsync("提示", "数据导入后未生成任何通道信息。", MessageBoxButton.OK);
                }
            }
            catch (Exception ex)
            {
                StatusMessage = string.Empty; // 在显示错误消息前清除状态
                await _messageService.ShowAsync("错误", $"导入数据处理错误: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ProcessImportedDataAsync Error: {ex.Message}");
                // 发生错误时也清空数据，避免不一致状态
                AllChannels?.Clear();
                OriginalAllChannels?.Clear();
                UpdateCurrentChannels();
                RaisePropertyChanged(nameof(AllChannels));
                UpdatePointStatistics();
                await RefreshBatchesFromChannelsAsync();
                SelectedBatch = null;
            }
            finally
            {
                IsLoading = false;
                StatusMessage = string.Empty;
            }
        }

        /// <summary>
        /// 恢复配置
        /// </summary>
        /// <remarks>
        /// 从数据库中恢复之前保存的通道配置数据
        /// </remarks>
        private async void RestoreConfig()
        {
            try
            {
                // 显示历史记录窗口
                await LoadTestBatchesAsync();
                IsHistoryRecordsOpen = true;
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"加载历史测试记录失败: {ex.Message}", MessageBoxButton.OK);
            }
        }

        /// <summary>
        /// 加载测试批次
        /// </summary>
        /// <returns>加载完成的任务</returns>
        /// <remarks>
        /// 从数据库加载所有可用的测试批次信息
        /// </remarks>
        private async Task LoadTestBatchesAsync()
        {
            try
            {
                IsLoading = true;
                StatusMessage = "正在加载历史测试记录...";

                // 获取所有测试批次信息
                var batches = await _testRecordService.GetAllTestBatchesAsync();

                if (batches != null && batches.Any())
                {
                    TestBatches = new ObservableCollection<TestBatchInfo>(batches);
                    SelectedTestBatch = TestBatches.FirstOrDefault();
                }
                else
                {
                    TestBatches = new ObservableCollection<TestBatchInfo>();
                    await _messageService.ShowAsync("提示", "没有找到历史测试记录", MessageBoxButton.OK);
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"加载历史测试记录时出错: {ex.Message}", MessageBoxButton.OK);
            }
            finally
            {
                IsLoading = false;
                StatusMessage = string.Empty;
            }
        }

        /// <summary>
        /// 初始化批次数据
        /// </summary>
        /// <remarks>
        /// 初始化批次选择界面并获取可用的批次列表
        /// </remarks>
        private async void InitializeBatchData()
        {
            // 初始化批次数据
            Batches = new ObservableCollection<BatchInfo>();

            // 尝试从通道映射信息中提取批次信息
            await UpdateBatchInfoAsync();
        }

        /// <summary>
        /// 更新批次信息
        /// </summary>
        /// <returns>更新完成的任务</returns>
        /// <remarks>
        /// 更新当前选中批次的详细信息，包括测试状态和统计数据
        /// </remarks>
        private async Task UpdateBatchInfoAsync()
        {
            try
            {
                IsLoading = true;
                StatusMessage = "正在更新批次信息...";

                // 检查AllChannels是否有效
                if (AllChannels == null || !AllChannels.Any())
                {
                    Message = "没有可用的通道信息";
                    return;
                }

                // 同步确保批次信息和PLC通道信息是最新的
                foreach (var channel in AllChannels)
                {
                    if (channel != null && !string.IsNullOrEmpty(channel.TestBatch))
                    {
                        var result = AllChannels.FirstOrDefault(r =>
                            r != null &&
                            r.VariableName == channel.VariableName &&
                            r.ChannelTag == channel.ChannelTag);

                        if (result != null)
                        {
                            result.TestBatch = channel.TestBatch;
                            result.TestPLCChannelTag = channel.TestPLCChannelTag;
                        }
                    }
                }

                // 使用通道映射服务提取批次信息
                var batches = await _channelMappingService.ExtractBatchInfoAsync(AllChannels);

                // 根据测试结果更新批次状态
                var updatedBatches = await _channelMappingService.UpdateBatchStatusAsync(batches, AllChannels);

                // 保存当前选中的批次名称
                string selectedBatchName = SelectedBatch?.BatchName;

                // 更新批次集合
                if (updatedBatches != null && updatedBatches.Any())
                {
                    Batches = new ObservableCollection<BatchInfo>(updatedBatches);

                    // 如果之前有选中的批次，尝试找回并选中
                    if (!string.IsNullOrEmpty(selectedBatchName))
                    {
                        var updatedSelectedBatch =
                            Batches.FirstOrDefault(b => b != null && b.BatchName == selectedBatchName);
                        if (updatedSelectedBatch != null)
                        {
                            // 直接设置字段而不触发OnBatchSelected
                            _selectedBatch = updatedSelectedBatch;
                            RaisePropertyChanged(nameof(SelectedBatch));

                            // 更新接线确认按钮状态
                            IsWiringCompleteBtnEnabled = updatedSelectedBatch.Status == "未开始" ||
                                                         updatedSelectedBatch.Status == "测试中";
                            //IsWiringCompleteBtnEnabled = true;
                        }
                    }
                    // 如果没有选中批次但有可用批次，选择第一个
                    else if (Batches.Count > 0)
                    {
                        SelectedBatch = Batches[0];
                    }
                }
                else
                {
                    Message = "未找到批次信息";
                }

                // 更新点位统计数据
                UpdatePointStatistics();

                // 新增：根据批次状态和通道状态刷新"开始测试"按钮可用性
                if (SelectedBatch != null)
                {
                    IsStartTestButtonEnabled = SelectedBatch.Status == "接线已确认" &&
                                               AllChannels.Any(c => c.TestBatch == SelectedBatch.BatchName && c.HardPointTestResult == "等待测试");
                    (StartTestCommand as DelegateCommand)?.RaiseCanExecuteChanged();
                }
                else
                {
                    IsStartTestButtonEnabled = false;
                    (StartTestCommand as DelegateCommand)?.RaiseCanExecuteChanged();
                }

                // 通知UI更新
                RaisePropertyChanged(nameof(AllChannels));
            }
            catch (Exception ex)
            {
                Message = $"更新批次信息失败: {ex.Message}";
                System.Diagnostics.Debug.WriteLine($"更新批次信息失败: {ex.Message}");
            }
            finally
            {
                IsLoading = false;
                StatusMessage = string.Empty;
            }
        }

        /// <summary>
        /// 执行批次选择
        /// </summary>
        /// <remarks>
        /// 打开批次选择对话框并加载可用批次
        /// </remarks>
        private async void ExecuteSelectBatch()
        {
            try
            {
                IsLoading = true;
                StatusMessage = "正在获取批次信息...";

                // 使用原始通道数据更新批次信息，确保批次列表完整
                if (OriginalAllChannels != null && OriginalAllChannels.Any())
                {
                    // 使用通道映射服务提取批次信息
                    var batchInfoList = await _channelMappingService.ExtractBatchInfoAsync(OriginalAllChannels);

                    // 检查是否有批次信息
                    if (batchInfoList != null && batchInfoList.Any())
                    {
                        Batches = new ObservableCollection<BatchInfo>(batchInfoList.OrderBy(b=>b.BatchName));

                        // 预先选择第一个批次，提升用户体验
                        if (Batches.Count > 0 && SelectedBatch == null)
                        {
                            //SelectedBatch = Batches[0];
                        }
                    }
                    else
                    {
                        Message = "未找到批次信息，请先分配通道";
                    }
                }
                else
                {
                    // 首次使用当前通道集合
                    await UpdateBatchInfoAsync();
                }
                // 显示批次选择窗口
                IsBatchSelectionOpen = true;
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"获取批次信息失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"获取批次信息失败: {ex.Message}");
            }
            finally
            {
                IsLoading = false;
                StatusMessage = string.Empty;
            }
        }
        /// <summary>
        /// 导出通道映射
        /// </summary>
        private void ExportChannelMap()
        {
            try
            {
                IsLoading = true;
                StatusMessage = "正在导出通道映射表...";

                if (AllChannels == null || !AllChannels.Any())
                {
                    _messageService.ShowAsync("导出失败", "没有可导出的通道映射数据", MessageBoxButton.OK);
                    return;
                }

                // 使用ITestResultExportService导出通道映射表
                _testResultExportService.ExportChannelMapToExcelAsync(AllChannels)
                    .ContinueWith(task =>
                    {
                        Application.Current.Dispatcher.Invoke(() =>
                        {
                            if (task.Result)
                            {
                                StatusMessage = "通道映射表导出成功";
                            }
                            else
                            {
                                StatusMessage = "通道映射表导出失败";
                            }
                            IsLoading = false;
                        });
                    });
            }
            catch (Exception ex)
            {
                _messageService.ShowAsync("导出失败", $"导出通道映射表时发生错误: {ex.Message}", MessageBoxButton.OK);
                StatusMessage = string.Empty;
                IsLoading = false;
            }
        }

        /// <summary>
        /// 跳过选择的模块，不测试并添加备注项
        /// </summary>
        private async void SkipModule()
        {
            // 检查是否有可用通道
            if (AllChannels == null || !AllChannels.Any())
            {
                await _messageService.ShowAsync("提示", "没有可用的通道信息", MessageBoxButton.OK);
                return;
            }

            try
            {
                // 输出调试信息
                System.Diagnostics.Debug.WriteLine($"开始提取模块信息: AllChannels包含 {AllChannels.Count} 个通道");
                int validChannels = AllChannels.Count(c => !string.IsNullOrEmpty(c.ChannelTag));
                System.Diagnostics.Debug.WriteLine($"有效ChannelTag的通道数量: {validChannels}");
                
                // 输出前10个通道的ChannelTag示例
                var sampleChannels = AllChannels.Where(c => !string.IsNullOrEmpty(c.ChannelTag)).Take(10);
                foreach (var channel in sampleChannels)
                {
                    System.Diagnostics.Debug.WriteLine($"通道示例: {channel.ChannelTag}, 模块类型: {channel.ModuleType}");
                }
                
                // 清空搜索过滤和选中状态
                ModuleSearchFilter = string.Empty;
                SelectAllModules = false;
                SkipReason = string.Empty;
                
                // 打开模块跳过窗口
                IsSkipModuleOpen = true;
                
                // 从所有通道中提取模块信息
                RefreshModules();
                
                // 打印模块列表信息
                System.Diagnostics.Debug.WriteLine($"模块提取结果: {(Modules == null ? "Modules为null" : $"找到{Modules.Count}个模块")}");
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"跳过模块操作异常: {ex.Message}");
                System.Diagnostics.Debug.WriteLine($"异常详情: {ex}");
                await _messageService.ShowAsync("错误", $"跳过模块操作失败: {ex.Message}", MessageBoxButton.OK);
            }
        }

        /// <summary>
        /// 刷新模块列表
        /// </summary>
        private void RefreshModules()
        {
            if (AllChannels == null || !AllChannels.Any())
                return;

            try
            {
                // 按模块为单位汇总模块信息，从ChannelTag中提取机架、槽位、模块类型信息
                var moduleGroups = AllChannels
                    .Where(c => !string.IsNullOrEmpty(c.ChannelTag))
                    .Select(c => new
                    {
                        Channel = c,
                        Parts = c.ChannelTag.Split('_')
                    })
                    .Where(x => x.Parts.Length >= 3) // 确保至少包含机架_槽_模块类型
                    .GroupBy(x => new
                    {
                        Rack = x.Parts[0],
                        Slot = x.Parts[1],
                        ModuleType = x.Parts[2]
                    })
                    .Select(g => new ModuleInfo
                    {
                        // 生成模块名称: "机架1_槽2_AI"
                        ModuleName = $"{g.Key.Rack}机架_{g.Key.Slot}槽_{g.Key.ModuleType}",
                        ChannelCount = g.Count(),
                        ModuleType = g.Key.ModuleType,
                        IsSelected = false
                    })
                    .OrderBy(m => m.ModuleName)
                    .ToList();

                Modules = new ObservableCollection<ModuleInfo>(moduleGroups);
                
                // 通知UI更新
                RaisePropertyChanged(nameof(Modules));
                RaisePropertyChanged(nameof(FilteredModules));
                
                // 输出调试信息
                System.Diagnostics.Debug.WriteLine($"已加载 {Modules.Count} 个模块");
                foreach (var module in Modules)
                {
                    System.Diagnostics.Debug.WriteLine($"模块: {module.ModuleName}, 类型: {module.ModuleType}, 通道数: {module.ChannelCount}");
                }
            }
            catch (Exception ex)
            {
                // 在非异步方法中无法使用 await，这里使用"弃元"方式触发对话框但不等待结果
                _ = _messageService.ShowAsync("错误", $"提取模块信息失败: {ex.Message}", MessageBoxButton.OK);
            }
        }

        /// <summary>
        /// 确认跳过选中的模块
        /// </summary>
        private async void ConfirmSkipModule()
        {
            if (Modules == null || !Modules.Any(m => m.IsSelected))
            {
                await _messageService.ShowAsync("提示", "请至少选择一个要跳过的模块。", MessageBoxButton.OK);
                return;
            }

            if (string.IsNullOrWhiteSpace(SkipReason))
            {
                await _messageService.ShowAsync("提示", "请输入跳过原因。", MessageBoxButton.OK);
                return;
            }

            var selectedModules = Modules.Where(m => m.IsSelected).ToList();
            if (!selectedModules.Any()) return;

            string confirmationMessage = $"确定要跳过选中的 {selectedModules.Count} 个模块吗？\n原因: {SkipReason}";
            var result = await _messageService.ShowAsync("确认跳过模块", confirmationMessage, MessageBoxButton.YesNo);
            if (result != MessageBoxResult.Yes)
            {
                return;
            }

            IsLoading = true;
            StatusMessage = "正在处理跳过模块...";
            List<Guid> modifiedChannelIds = new List<Guid>();

            try
            {
                var channelsToSkip = AllChannels
                    .Where(c => selectedModules.Any(m => m.ModuleName == c.ModuleName && m.StationName == c.StationName))
                    .ToList();

                if (channelsToSkip.Any())
                {
                    DateTime skipTime = DateTime.Now;
                    foreach (var channel in channelsToSkip)
                    {
                        _channelStateManager.MarkAsSkipped(channel, SkipReason, skipTime);
                        modifiedChannelIds.Add(channel.Id); 
                    }

                    // 持久化更改 (如果需要)
                    // await _channelMappingService.UpdateChannelMappingsAsync(channelsToSkip); // 示例
                    await _testRecordService.SaveTestRecordsAsync(channelsToSkip); // 保存跳过的记录

                    await _messageService.ShowAsync("操作成功", $"{channelsToSkip.Count} 个通道已成功标记为跳过。", MessageBoxButton.OK);
                }
                else
                {
                    await _messageService.ShowAsync("提示", "未找到与选定模块关联的可跳过通道。", MessageBoxButton.OK);
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"跳过模块时发生错误: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ConfirmSkipModule Error: {ex.Message}");
            }
            finally
            {
                IsLoading = false;
                StatusMessage = string.Empty;
                IsSkipModuleOpen = false;
                SkipReason = string.Empty; // 清空原因
                RefreshModules(); // 取消模块选择

                // UI 更新
                if (modifiedChannelIds.Any())
                {
                    // 触发属性变更，让UI知道数据已更新
                    // RaisePropertyChanged(nameof(AllChannels)); // 如果AllChannels本身被替换
                    // 或者如果AllChannels是ObservableCollection并且ChannelMapping实现了INotifyPropertyChanged，则其内部属性更改会自动通知绑定
                    // 对于集合视图的更新，可以这样做：
                    CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                    CollectionViewSource.GetDefaultView(CurrentChannels)?.Refresh();
                }
                UpdatePointStatistics();
                RefreshBatchStatus();
                ExportTestResultsCommand.RaiseCanExecuteChanged();
                // OnChannelStatesModified(modifiedChannelIds); // 或者通过事件机制更新
            }
        }

        private void CancelSkipModule()
        {
            IsSkipModuleOpen = false;
            SkipReason = string.Empty;
        }

        /// <summary>
        /// 保存单个通道的测试记录
        /// </summary>
        /// <param name="channel">需要保存记录的通道对象</param>
        /// <returns>保存是否成功的布尔值任务</returns>
        /// <remarks>
        /// 将单个通道的测试结果保存到数据库
        /// </remarks>
        private async Task<bool> SaveSingleChannelTestRecordAsync(ChannelMapping channel)
        {
            try
            {
                if (channel == null)
                    return false;

                // 确保通道有测试标识
                if (string.IsNullOrEmpty(channel.TestTag))
                {
                    // 使用当前批次名作为TestTag的一部分，以确保同一测试批次的记录具有相同的TestTag
                    string batchInfo = string.IsNullOrEmpty(channel.TestBatch) ? "UNKNOWN" : channel.TestBatch;
                    channel.TestTag = $"TEST_{batchInfo}_{DateTime.Now:yyyyMMdd}";
                }

                // 记录最终测试时间
                channel.FinalTestTime = DateTime.Now;

                // 调用测试记录服务保存记录
                bool result = await _testRecordService.SaveTestRecordAsync(channel);

                if (result)
                {
                    System.Diagnostics.Debug.WriteLine($"通道 {channel.VariableName} 测试记录已保存");
                }
                else
                {
                    System.Diagnostics.Debug.WriteLine($"通道 {channel.VariableName} 测试记录保存失败");
                }

                return result;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"保存单个通道测试记录时出错: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 检查通道是否完成所有测试并自动保存记录
        /// </summary>
        /// <param name="channel">需要检查的通道对象</param>
        /// <param name="useAsyncQueue">是否使用异步队列（避免并发锁竞争）</param>
        /// <returns>检查和保存操作的任务</returns>
        /// <remarks>
        /// 【关键节点3】手动测试整体通过时存储单条记录
        /// 当通道的所有必需测试（硬点+手动）都完成时，自动保存到数据库
        /// 添加防重复保存逻辑，避免性能问题
        /// 使用异步队列机制避免并发锁竞争
        /// </remarks>
        private async Task CheckAndSaveCompletedChannelAsync(ChannelMapping channel, bool useAsyncQueue = true)
        {
            try
            {
                if (channel == null) return;

                // 检查通道是否已完成所有测试（通过或失败）
                bool isCompleted = channel.OverallStatus == OverallResultStatus.Passed || 
                                   channel.OverallStatus == OverallResultStatus.Failed ||
                                   channel.OverallStatus == OverallResultStatus.Skipped;

                if (isCompleted)
                {
                    // 添加防重复保存检查：如果已经有FinalTestTime且不为今天，说明之前已保存过
                    bool alreadySaved = channel.FinalTestTime.HasValue && 
                                       (DateTime.Now - channel.FinalTestTime.Value).TotalMinutes > 5;
                    
                    if (alreadySaved)
                    {
                        System.Diagnostics.Debug.WriteLine($"通道 {channel.VariableName} 已在 {channel.FinalTestTime} 保存过，跳过重复保存");
                        return;
                    }

                    System.Diagnostics.Debug.WriteLine($"检测到通道 {channel.VariableName} 测试完成，开始自动保存");

                    bool saveSuccess = false;
                    
                    if (useAsyncQueue)
                    {
                        // 使用异步队列避免并发锁竞争
                        saveSuccess = await _testRecordService.SaveTestRecordAsyncQueued(channel);
                    }
                    else
                    {
                        // 直接保存（用于手动测试等非并发场景）
                        saveSuccess = await SaveSingleChannelTestRecordAsync(channel);
                    }
                    
                    if (saveSuccess)
                    {
                        System.Diagnostics.Debug.WriteLine($"通道 {channel.VariableName} 测试记录自动保存成功");
                    }
                    else
                    {
                        System.Diagnostics.Debug.WriteLine($"通道 {channel.VariableName} 测试记录自动保存失败");
                    }
                }
                else
                {
                    System.Diagnostics.Debug.WriteLine($"通道 {channel.VariableName} 测试尚未完成，当前状态: {channel.OverallStatus}");
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"检查和保存通道测试完成状态时出错: {ex.Message}");
                // 错误不应影响用户的正常操作流程
            }
        }

        /// <summary>
        /// 检查硬点测试完成后是否可以直接完成整体测试
        /// </summary>
        /// <param name="channel">刚完成硬点测试的通道</param>
        /// <returns>异步任务</returns>
        /// <remarks>
        /// 某些通道类型（如DI/DO）在硬点测试完成后可能就直接达到整体完成状态
        /// 此时需要立即保存，但要使用异步队列避免并发锁竞争
        /// </remarks>
        public async Task CheckHardPointTestCompletionAsync(ChannelMapping channel)
        {
            try
            {
                if (channel == null) return;

                // 检查是否是可以直接完成的通道类型
                bool canCompleteDirectly = false;
                
                switch (channel.ModuleType?.ToUpper())
                {
                    case "DI":
                    case "DO":
                        // DI/DO通道通常只需要硬点测试+显示值确认就完成
                        canCompleteDirectly = channel.HardPointTestResult == "通过" && 
                                              channel.ShowValueStatus == "通过";
                        break;
                    case "DINONE":
                    case "DONONE":
                        // 无源DI/DO可能硬点测试完成就直接完成
                        canCompleteDirectly = channel.HardPointTestResult == "通过";
                        break;
                    // AI/AO通道需要更多手动测试，通常不会直接完成
                    default:
                        canCompleteDirectly = false;
                        break;
                }

                if (canCompleteDirectly)
                {
                    System.Diagnostics.Debug.WriteLine($"通道 {channel.VariableName} 硬点测试完成后可直接完成，使用异步队列保存");
                    await CheckAndSaveCompletedChannelAsync(channel, useAsyncQueue: true);
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"检查硬点测试完成状态时出错: {ex.Message}");
            }
        }
        #endregion

        #region 6、手动测试 - AI通道
        /// <summary>
        /// 打开AI通道手动测试窗口
        /// </summary>
        /// <param name="channel">需要手动测试的AI通道</param>
        /// <remarks>
        /// 该方法打开AI通道的手动测试窗口，执行以下操作：
        /// 1. 设置当前选中的通道
        /// 2. 初始化手动测试状态
        /// 3. 打开AI手动测试窗口
        /// </remarks>
        private async void OpenAIManualTest(ChannelMapping channel)
        {
            try
            {
                if (channel != null && channel.ModuleType?.ToLower() == "ai")
                {
                    CurrentChannel = channel;
                    CurrentTestResult = channel; // 保持与当前通道同步

                    // 调用 ChannelStateManager 来准备手动测试状态
                    _channelStateManager.BeginManualTest(channel); 

                    // UI 更新逻辑 (RaisePropertyChanged, UpdatePointStatistics 等)
                    // 应在状态管理器调用后，并且如果事件聚合器未处理这些，则在此处显式调用
                    RaisePropertyChanged(nameof(CurrentChannel));
                    RaisePropertyChanged(nameof(AllChannels)); // 或更细粒度的通知
                    CollectionViewSource.GetDefaultView(AllChannels)?.Refresh(); // 确保DataGrid等控件看到变更
                    UpdatePointStatistics();
                    RefreshBatchStatus(); // 如果适用
                    ExportTestResultsCommand.RaiseCanExecuteChanged();

                    // AISetValue 的初始化可以保留，因为它服务于UI输入，不直接是ChannelMapping的状态
                    if (channel.HighLimit.HasValue && channel.LowLimit.HasValue && channel.HighLimit.Value > channel.LowLimit.Value)
                    {
                        AISetValue = (Math.Round((new Random().NextDouble() * (channel.HighLimit.Value - channel.LowLimit.Value) + channel.LowLimit.Value), 3)).ToString();
                    }
                    else
                    {
                        AISetValue = channel.LowLimit.HasValue ? channel.LowLimit.Value.ToString() : "0"; // 默认值
                    }

                    IsAIManualTestOpen = true;

                    // 移除原先在此处直接初始化子测试状态的逻辑，例如：
                    // channel.ShowValueStatus = "未测试";
                    // channel.HighAlarmStatus = "未测试"; 等等，这些已由 BeginManualTest 处理。
                    // 也移除原先在此处直接修改 channel.ResultText 的逻辑。

                    // 启动报警设定值监控服务，每 0.5 秒读取一次并更新绑定值
                    _manualTestIoService.StartAlarmValueMonitoring(CurrentChannel, (sl, sll, sh, shh) =>
                    {
                        Application.Current.Dispatcher.Invoke(() =>
                        {
                            AILowSetValue      = sl  .HasValue ? sl.Value .ToString("F3") : "N/A";
                            AILowLowSetValue   = sll .HasValue ? sll.Value.ToString("F3") : "N/A";
                            AIHighSetValue     = sh  .HasValue ? sh.Value .ToString("F3") : "N/A";
                            AIHighHighSetValue = shh .HasValue ? shh.Value.ToString("F3") : "N/A";
                        });
                    });
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"打开AI手动测试窗口失败: {ex.Message}");
                await _messageService.ShowAsync("错误", $"打开AI手动测试窗口失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 关闭AI通道手动测试窗口
        /// </summary>
        /// <remarks>
        /// 该方法关闭AI通道的手动测试窗口，并清空当前选中的通道和测试结果。
        /// </remarks>
        private async void ExecuteCloseAIManualTest()
        {
            try
            {
                // 设置AI手动测试窗口打开状态为false
                IsAIManualTestOpen = false;

                // 检查当前通道是否通过了测试
                if (CurrentChannel != null && CurrentChannel.ShowValueStatus == "通过")
                {
                    // 如果手动测试已通过，检查是否所有测试都已完成
                    _testTaskManager.CompleteAllTestsAsync();
                }

                // 刷新批次状态
                RefreshBatchStatus();

                // 更新点位统计
                UpdatePointStatistics();

                // 停止报警设定值监控
                _manualTestIoService.StopAll();
            }
            catch (Exception ex)
            {
                _ = _messageService.ShowAsync("错误", $"关闭AI手动测试窗口失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 发送AI测试值
        /// </summary>
        /// <param name="channel">需要测试的AI通道</param>
        /// <remarks>
        /// 该方法发送AI测试值到测试PLC，执行以下操作：
        /// 1. 验证测试值的有效性
        /// 2. 将测试值转换为百分比值
        /// 3. 将测试值写入测试PLC的相应地址
        /// 4. 更新手动测试状态
        /// </remarks>
        private async void ExecuteSendAITestValue(ChannelMapping channel)
        {
            try
            {
                if (channel == null)
                {
                    await _messageService.ShowAsync("错误", "通道信息无效", MessageBoxButton.OK);
                    return;
                }

                // 验证并解析AI设定值
                if (string.IsNullOrWhiteSpace(AISetValue) || !float.TryParse(AISetValue, out float testValue))
                {
                    await _messageService.ShowAsync("错误", "请输入有效的测试值", MessageBoxButton.OK);
                    return;
                }

                // 调用服务发送测试值
                bool success = await _manualTestIoService.SendAITestValueAsync(channel, testValue);
                
                if (!success)
                {
                    await _messageService.ShowAsync("错误", "发送AI测试值失败，请检查通道配置和PLC连接", MessageBoxButton.OK);
                }
                // 注意：成功时不显示消息，避免频繁弹窗干扰操作员
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"发送AI测试值失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ExecuteSendAITestValue Error: {ex.Message}");
            }
        }
        /// <summary>
        /// 确认AI显示值
        /// </summary>
        /// <param name="channel">需要确认的AI通道</param>
        /// <remarks>
        /// 该方法确认AI通道的显示值是否正确，执行以下操作：
        /// 1. 将通道的显示值状态设置为"通过"
        /// 2. 更新手动测试状态
        /// 3. 检查并更新通道的总体测试状态
        /// </remarks>
        private async void ExecuteConfirmAIValue(ChannelMapping channel)
        {
            try
            {
                if (channel != null)
                {
                    _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.ShowValue, true, DateTime.Now);

                    // 检查通道是否完成所有测试，如果是则自动保存
                    await CheckAndSaveCompletedChannelAsync(channel);

                    RaisePropertyChanged(nameof(CurrentChannel)); // CurrentChannel 应该在手动测试UI中被正确设置
                    RaisePropertyChanged(nameof(AllChannels)); // Or CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                    UpdatePointStatistics();
                    RefreshBatchStatus(); // 如果适用
                    ExportTestResultsCommand.RaiseCanExecuteChanged();
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"确认AI显示值失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ExecuteConfirmAIValue Error: {ex.Message}");
            }
        }
        /// <summary>
        /// 发送AI高报测试值
        /// </summary>
        /// <param name="channel">需要测试高报的AI通道</param>
        /// <remarks>
        /// 该方法发送高报测试值到测试PLC，执行以下操作：
        /// 1. 检查通道是否配置了高报值
        /// 2. 将高报值转换为百分比值
        /// 3. 将高报值写入测试PLC的相应地址
        /// 4. 更新高报测试状态
        /// </remarks>
        private async void ExecuteSendAIHighAlarm(ChannelMapping channel)
        {
            try
            {
                if (channel.HighLimit.HasValue)
                {
                    await _testPlc.WriteAnalogValueAsync(channel.TestPLCCommunicationAddress.Substring(1), ChannelRangeConversion.RealValueToPercentage(channel, channel.HighLimit.Value) + 5f);
                }
                else
                {
                    // Optionally log or message that HighLimit is not set
                    System.Diagnostics.Debug.WriteLine($"Cannot send AI High Alarm for {channel.VariableName}: HighLimit is null.");
                    // await _messageService.ShowAsync("错误", $"通道 {channel.VariableName} 未配置高报警限值。", MessageBoxButton.OK);
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"发送AI高报警测试信号失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 发送AI高高报测试值
        /// </summary>
        /// <param name="channel">需要测试高高报的AI通道</param>
        /// <remarks>
        /// 该方法发送高高报测试值到测试PLC，执行以下操作：
        /// 1. 检查通道是否配置了高高报值
        /// 2. 将高高报值转换为百分比值
        /// 3. 将高高报值写入测试PLC的相应地址
        /// 4. 更新高高报测试状态
        /// </remarks>
        private async void ExecuteSendAIHighHighAlarm(ChannelMapping channel)
        {
            try
            {
                if (channel.HighHighLimit.HasValue)
                {
                    await _testPlc.WriteAnalogValueAsync(channel.TestPLCCommunicationAddress.Substring(1), ChannelRangeConversion.RealValueToPercentage(channel, channel.HighHighLimit.Value) + 5f);
                }
                else
                {
                    System.Diagnostics.Debug.WriteLine($"Cannot send AI High High Alarm for {channel.VariableName}: HighHighLimit is null.");
                    // await _messageService.ShowAsync("错误", $"通道 {channel.VariableName} 未配置高高报警限值。", MessageBoxButton.OK);
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"发送AI高报警测试信号失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 复位AI高报测试
        /// </summary>
        /// <param name="channel">需要复位高报的AI通道</param>
        /// <remarks>
        /// 该方法将AI通道的高报测试值复位到正常范围内，执行以下操作：
        /// 1. 将测试值设置为正常范围内的值（通常是50%量程值）
        /// 2. 将测试值写入测试PLC的相应地址
        /// 3. 更新高报测试状态
        /// </remarks>
        private async void ExecuteResetAIHighAlarm(ChannelMapping channel)
        {
            try
            {
                // 实现重置AI高报警测试信号的逻辑
                // 直接执行业务逻辑，不弹出消息框
                await _testPlc.WriteAnalogValueAsync(channel.TestPLCCommunicationAddress.Substring(1),
                    ChannelRangeConversion.RealValueToPercentage(channel, AISetValue));
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"重置AI高报警测试信号失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 确认AI高报测试
        /// </summary>
        /// <param name="channel">需要确认高报的AI通道</param>
        /// <remarks>
        /// 该方法确认AI通道的高报功能是否正常，执行以下操作：
        /// 1. 将通道的高报状态设置为"通过"
        /// 2. 更新高报测试状态
        /// 3. 检查并更新通道的总体测试状态
        /// </remarks>
        private async void ExecuteConfirmAIHighAlarm(ChannelMapping channel)
        {
            try
            {
                if (channel != null)
                {
                    // 假设高报和高高报通过一个按钮确认，或者需要拆分
                    // 为简化，这里合并处理，实际可能需要根据UI设计调整
                    _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.HighAlarm, true, DateTime.Now);
                    // 如果高高报是独立控制和确认的，需要独立的ManualTestItem和调用
                    _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.HighHighAlarm, true, DateTime.Now);

                    // 检查通道是否完成所有测试，如果是则自动保存
                    await CheckAndSaveCompletedChannelAsync(channel);

                    RaisePropertyChanged(nameof(CurrentChannel));
                    RaisePropertyChanged(nameof(AllChannels));
                    UpdatePointStatistics();
                    RefreshBatchStatus();
                    ExportTestResultsCommand.RaiseCanExecuteChanged();
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"确认AI高报警失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ExecuteConfirmAIHighAlarm Error: {ex.Message}");
            }
        }
        /// <summary>
        /// 发送AI低报测试值
        /// </summary>
        /// <param name="channel">需要测试低报的AI通道</param>
        /// <remarks>
        /// 该方法发送低报测试值到测试PLC，执行以下操作：
        /// 1. 检查通道是否配置了低报值
        /// 2. 将低报值转换为百分比值
        /// 3. 将低报值写入测试PLC的相应地址
        /// 4. 更新低报测试状态
        /// </remarks>
        private async void ExecuteSendAILowAlarm(ChannelMapping channel)
        {
            try
            {
                if (channel.LowLimit.HasValue)
                {
                    await _testPlc.WriteAnalogValueAsync(channel.TestPLCCommunicationAddress.Substring(1), ChannelRangeConversion.RealValueToPercentage(channel, channel.LowLimit.Value) - 5f);
                }
                else
                {
                    System.Diagnostics.Debug.WriteLine($"Cannot send AI Low Alarm for {channel.VariableName}: LowLimit is null.");
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"发送AI低报警测试信号失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 发送AI低低报测试值
        /// </summary>
        /// <param name="channel">需要测试低低报的AI通道</param>
        /// <remarks>
        /// 该方法发送低低报测试值到测试PLC，执行以下操作：
        /// 1. 检查通道是否配置了低低报值
        /// 2. 将低低报值转换为百分比值
        /// 3. 将低低报值写入测试PLC的相应地址
        /// 4. 更新低低报测试状态
        /// </remarks>
        private async void ExecuteSendAILowLowAlarm(ChannelMapping channel)
        {
            try
            {
                if (channel.LowLowLimit.HasValue)
                {
                    await _testPlc.WriteAnalogValueAsync(channel.TestPLCCommunicationAddress.Substring(1), ChannelRangeConversion.RealValueToPercentage(channel, channel.LowLowLimit.Value) - 5f);
                }
                else
                {
                    System.Diagnostics.Debug.WriteLine($"Cannot send AI Low Low Alarm for {channel.VariableName}: LowLowLimit is null.");
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"发送AI低报警测试信号失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 复位AI低报测试
        /// </summary>
        /// <param name="channel">需要复位低报的AI通道</param>
        /// <remarks>
        /// 该方法将AI通道的低报测试值复位到正常范围内，执行以下操作：
        /// 1. 将测试值设置为正常范围内的值（通常是50%量程值）
        /// 2. 将测试值写入测试PLC的相应地址
        /// 3. 更新低报测试状态
        /// </remarks>
        private async void ExecuteResetAILowAlarm(ChannelMapping channel)
        {
            try
            {
                // 实现重置AI低报警测试信号的逻辑
                // 直接执行业务逻辑，不弹出消息框
                await _testPlc.WriteAnalogValueAsync(channel.TestPLCCommunicationAddress.Substring(1),
                    ChannelRangeConversion.RealValueToPercentage(channel, AISetValue));
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"重置AI低报警测试信号失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 确认AI低报测试
        /// </summary>
        /// <param name="channel">需要确认低报的AI通道</param>
        /// <remarks>
        /// 该方法确认AI通道的低报功能是否正常，执行以下操作：
        /// 1. 将通道的低报状态设置为"通过"
        /// 2. 更新低报测试状态
        /// 3. 检查并更新通道的总体测试状态
        /// </remarks>
        private async void ExecuteConfirmAILowAlarm(ChannelMapping channel)
        {
            try
            {
                if (channel != null)
                {
                    _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.LowAlarm, true, DateTime.Now);
                    // 如果低低报是独立控制和确认的，需要独立的ManualTestItem和调用
                    _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.LowLowAlarm, true, DateTime.Now);

                    // 检查通道是否完成所有测试，如果是则自动保存
                    await CheckAndSaveCompletedChannelAsync(channel);

                    RaisePropertyChanged(nameof(CurrentChannel));
                    RaisePropertyChanged(nameof(AllChannels));
                    UpdatePointStatistics();
                    RefreshBatchStatus();
                    ExportTestResultsCommand.RaiseCanExecuteChanged();
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"确认AI低报警失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ExecuteConfirmAILowAlarm Error: {ex.Message}");
            }
        }
        /// <summary>
        /// 确认AI报警值设定测试
        /// </summary>
        /// <param name="channel">需要确认报警设定值的AI通道</param>
        /// <remarks>
        /// 该方法确认AI通道的报警值设定功能是否正常，执行以下操作：
        /// 1. 将通道的报警值设定状态设置为"通过"
        /// 2. 更新报警值设定测试状态
        /// 3. 检查并更新通道的总体测试状态
        /// </remarks>
        private async void ExecuteConfirmAIAlarmValueSet(ChannelMapping channel)
        {
            try
            {
                if (channel != null)
                {
                    _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.AlarmValueSet, true, DateTime.Now);

                    // 检查通道是否完成所有测试，如果是则自动保存
                    await CheckAndSaveCompletedChannelAsync(channel);

                    RaisePropertyChanged(nameof(CurrentChannel));
                    RaisePropertyChanged(nameof(AllChannels));
                    UpdatePointStatistics();
                    RefreshBatchStatus();
                    ExportTestResultsCommand.RaiseCanExecuteChanged();
                }
            }
            catch (Exception ex)
            {
                // MessageBox.Show($"确认AI低报警失败: {ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error); // 原有代码的笔误，应为AI报警值设定
                await _messageService.ShowAsync("错误", $"确认AI报警值设定失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ExecuteConfirmAIAlarmValueSet Error: {ex.Message}");
            }
        }
        /// <summary>
        /// 发送AI维护功能测试信号
        /// </summary>
        /// <remarks>
        /// 该方法关闭DI通道的手动测试窗口，并清空当前选中的通道和测试结果。
        /// </remarks>
        private void ExecuteCloseDIManualTest()
        {
            try
            {
                // 设置DI手动测试窗口打开状态为false
                IsDIManualTestOpen = false;

                // 检查当前通道是否通过了测试
                if (CurrentChannel != null && CurrentChannel.ShowValueStatus == "通过")
                {
                    // 如果手动测试已通过，检查是否所有测试都已完成
                    _testTaskManager.CompleteAllTestsAsync();
                }

                // 刷新批次状态
                RefreshBatchStatus();

                // 更新点位统计
                UpdatePointStatistics();
            }
            catch (Exception ex)
            {
                _ = _messageService.ShowAsync("错误", $"关闭DI手动测试窗口失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 发送DI测试信号
        /// </summary>
        /// <param name="channel">需要测试的DI通道</param>
        /// <remarks>
        /// 该方法发送DI测试信号到测试PLC，执行以下操作：
        /// 1. 验证通道参数的有效性
        /// 2. 调用ManualTestIoService发送DI测试信号（激活状态）
        /// 3. 处理操作结果并显示相应的反馈
        /// </remarks>
        private async void ExecuteSendDITest(ChannelMapping channel)
        {
            try
            {
                if (channel == null)
                {
                    await _messageService.ShowAsync("错误", "通道信息无效", MessageBoxButton.OK);
                    return;
                }

                // 调用服务发送DI测试信号
                bool success = await _manualTestIoService.SendDITestSignalAsync(channel);
                
                if (!success)
                {
                    await _messageService.ShowAsync("错误", "发送DI测试信号失败，请检查通道配置和PLC连接", MessageBoxButton.OK);
                }
                // 注意：成功时不显示消息，避免频繁弹窗干扰操作员
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"发送DI测试信号失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ExecuteSendDITest Error: {ex.Message}");
            }
        }
        /// <summary>
        /// 复位DI测试信号
        /// </summary>
        /// <param name="channel">需要复位的DI通道</param>
        /// <remarks>
        /// 该方法将DI通道的测试信号复位到非激活状态，执行以下操作：
        /// 1. 检查通道是否有效
        /// 2. 将DI信号设置为非激活状态
        /// 3. 将DI状态写入测试PLC的相应地址
        /// 4. 更新DI测试状态
        /// </remarks>
        private async void ExecuteResetDI(ChannelMapping channel)
        {
            try
            {
                if (channel == null)
                {
                    await _messageService.ShowAsync("错误", "通道信息无效", MessageBoxButton.OK);
                    return;
                }

                // 调用服务复位DI测试信号
                bool success = await _manualTestIoService.ResetDITestSignalAsync(channel);
                
                if (!success)
                {
                    await _messageService.ShowAsync("错误", "复位DI测试信号失败，请检查通道配置和PLC连接", MessageBoxButton.OK);
                }
                // 注意：成功时不显示消息，避免频繁弹窗干扰操作员
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"复位DI测试信号失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ExecuteResetDI Error: {ex.Message}");
            }
        }
        /// <summary>
        /// 确认DI测试
        /// </summary>
        /// <param name="channel">需要确认的DI通道</param>
        /// <remarks>
        /// 该方法确认DI通道的显示值是否正确，执行以下操作：
        /// 1. 将通道的显示值状态设置为"通过"
        /// 2. 更新DI测试状态
        /// 3. 检查并更新通道的总体测试状态
        /// </remarks>
        private async void ExecuteConfirmDI(ChannelMapping channel)
        {
            try
            {
                if (channel != null)
                {
                    _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.ShowValue, true, DateTime.Now);

                    // 检查通道是否完成所有测试，如果是则自动保存
                    await CheckAndSaveCompletedChannelAsync(channel);

                    RaisePropertyChanged(nameof(CurrentChannel));
                    RaisePropertyChanged(nameof(AllChannels));
                    CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                    UpdatePointStatistics();
                    RefreshBatchStatus();
                    ExportTestResultsCommand.RaiseCanExecuteChanged();
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"确认DI测试失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ExecuteConfirmDI Error: {ex.Message}");
            }
        }
        #endregion

        #region 8、手动测试-AO
        /// <summary>
        /// 打开AO通道手动测试窗口
        /// </summary>
        /// <param name="channel">需要手动测试的AO通道</param>
        /// <remarks>
        /// 该方法打开AO通道的手动测试窗口，执行以下操作：
        /// 1. 设置当前选中的通道
        /// 2. 初始化手动测试状态
        /// 3. 打开AO手动测试窗口
        /// 4. 启动AO数值监控服务
        /// </remarks>
        private async void OpenAOManualTest(ChannelMapping channel)
        {
            try
            {
                if (channel != null && (channel.ModuleType?.ToLower() == "ao" || channel.ModuleType?.ToLower() == "aonone"))
                {
                    CurrentChannel = channel;
                    CurrentTestResult = channel;

                    _channelStateManager.BeginManualTest(channel);

                    RaisePropertyChanged(nameof(CurrentChannel));
                    RaisePropertyChanged(nameof(AllChannels));
                    CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                    UpdatePointStatistics();
                    RefreshBatchStatus();
                    ExportTestResultsCommand.RaiseCanExecuteChanged();

                    IsAOManualTestOpen = true;
                    AOCurrentValue = string.Empty; // 清空上次的值
                    AOMonitorStatus = "停止监测";

                    // 启动AO数值监控服务，每 0.5 秒读取一次并更新绑定值
                    _manualTestIoService.StartAOValueMonitoring(CurrentChannel, (currentValue) =>
                    {
                        Application.Current.Dispatcher.Invoke(() =>
                        {
                            AOCurrentValue = currentValue;
                        });
                    });
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"打开AO手动测试窗口失败: {ex.Message}");
                await _messageService.ShowAsync("错误", $"打开AO手动测试窗口失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 关闭AO通道手动测试窗口
        /// </summary>
        /// <remarks>
        /// 该方法关闭AO通道的手动测试窗口，并清空当前选中的通道和测试结果。
        /// </remarks>
        private async void ExecuteCloseAOManualTest()
        {
            try
            {
                // 设置AO手动测试窗口打开状态为false
                IsAOManualTestOpen = false;

                // 停止AO数值监控
                _manualTestIoService.StopAll();

                // 清空AO当前值显示
                AOCurrentValue = string.Empty;
                AOMonitorStatus = "开始监测";

                // 检查当前通道是否通过了测试
                if (CurrentChannel != null && CurrentChannel.ShowValueStatus == "通过")
                {
                    // 如果手动测试已通过，检查是否所有测试都已完成
                    _testTaskManager.CompleteAllTestsAsync();
                }

                // 刷新批次状态
                RefreshBatchStatus();

                // 更新点位统计
                UpdatePointStatistics();
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"关闭AO手动测试窗口失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 启动AO监测
        /// </summary>
        /// <param name="channel">需要监测的AO通道</param>
        /// <remarks>
        /// 该方法用于启动或重启AO通道的监测过程。
        /// 实际的监控逻辑已通过ManualTestIoService的StartAOValueMonitoring方法实现。
        /// </remarks>
        private async void ExecuteStartAOMonitor(ChannelMapping channel)
        {
            try
            {
                if (channel != null && channel.ShowValueStatus != "通过" && IsAOManualTestOpen)
                {
                    // 设置当前监测的AO通道
                    CurrentChannel = channel;
                    CurrentTestResult = channel;
                    
                    // AO监控已在OpenAOManualTest中通过服务启动，此处仅更新状态
                    AOMonitorStatus = "停止监测";
                    
                    // 如果需要重新启动监控，可以调用服务的监控方法
                    if (string.IsNullOrEmpty(AOCurrentValue) || AOCurrentValue == "监控异常")
                    {
                        _manualTestIoService.StartAOValueMonitoring(CurrentChannel, (currentValue) =>
                        {
                            Application.Current.Dispatcher.Invoke(() =>
                            {
                                AOCurrentValue = currentValue;
                            });
                        });
                    }
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"启动AO监测失败: {ex.Message}", MessageBoxButton.OK);
            }
        }

        //private async void ExecuteSaveAO0(ChannelMapping channel)
        //{
        //    channel.Value0Percent = Convert.ToDouble(AOCurrentValue);
        //}
        //private async void ExecuteSaveAO25(ChannelMapping channel)
        //{
        //    channel.Value25Percent = Convert.ToDouble(AOCurrentValue);
        //}
        //private async void ExecuteSaveAO50(ChannelMapping channel)
        //{
        //    channel.Value50Percent = Convert.ToDouble(AOCurrentValue);
        //}
        //private async void ExecuteSaveAO75(ChannelMapping channel)
        //{
        //    channel.Value75Percent = Convert.ToDouble(AOCurrentValue);
        //}
        //private async void ExecuteSaveAO100(ChannelMapping channel)
        //{
        //    channel.Value100Percent = Convert.ToDouble(AOCurrentValue);
        //}
        /// <summary>
        /// 确认AO测试
        /// </summary>
        /// <param name="channel">需要确认的AO通道</param>
        /// <remarks>
        /// 该方法确认AO通道的显示值是否正确，执行以下操作：
        /// 1. 将通道的显示值状态设置为"通过"
        /// 2. 更新AO测试状态
        /// 3. 检查并更新通道的总体测试状态
        /// </remarks>
        private async void ExecuteConfirmAO(ChannelMapping channel)
        {
            try
            {
                if (channel != null)
                {
                    _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.ShowValue, true, DateTime.Now);
                    // AO的其他手动子测试项如TrendCheck, ReportCheck有单独的确认命令

                    // 检查通道是否完成所有测试，如果是则自动保存
                    await CheckAndSaveCompletedChannelAsync(channel);

                    RaisePropertyChanged(nameof(CurrentChannel));
                    RaisePropertyChanged(nameof(AllChannels));
                    CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                    UpdatePointStatistics();
                    RefreshBatchStatus();
                    ExportTestResultsCommand.RaiseCanExecuteChanged();
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"确认AO测试失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"打开DO手动测试窗口失败: {ex.Message}");
                await _messageService.ShowAsync("错误", $"打开DO手动测试窗口失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        /// <summary>
        /// 关闭DO通道手动测试窗口
        /// </summary>
        /// <remarks>
        /// 该方法关闭历史记录查看窗口。
        /// </remarks>
        private void CloseHistoryRecords()
        {
            IsHistoryRecordsOpen = false;
        }
        /// <summary>
        /// 恢复选中的历史测试记录
        /// </summary>
        /// <remarks>
        /// 该方法用于恢复用户在历史记录窗口中选择的测试批次记录。
        /// 执行流程：
        /// 1. 验证是否选择了测试批次
        /// 2. 显示确认对话框，提醒用户当前数据将被覆盖
        /// 3. 调用测试记录服务恢复选中批次的测试记录
        /// 4. 更新AllChannels集合和原始通道集合
        /// 5. 更新批次信息和点位统计数据
        /// 6. 关闭历史记录窗口并显示成功消息
        /// </remarks>
        private async void RestoreTestRecords()
        {
            if (SelectedTestBatch == null)
            {
                await _messageService.ShowAsync("提示", "请先选择一个测试批次", MessageBoxButton.OK);
                return;
            }

            bool wasPopupOpen = IsHistoryRecordsOpen;
            try
            {
                IsHistoryRecordsOpen = false; 

                ConfirmDialogView confirmDialog = new ConfirmDialogView(
                    $"确定要恢复测试批次 {SelectedTestBatch.TestTag} 的记录吗？当前数据将被覆盖。",
                    "确认恢复");
                confirmDialog.Owner = Application.Current.MainWindow;
                bool? dialogResult = confirmDialog.ShowDialog();

                if (dialogResult == true)
                {
                    IsLoading = true;
                    StatusMessage = $"正在恢复测试记录 {SelectedTestBatch.TestTag}...";
                    var records = await _testRecordService.RestoreTestRecordsAsync(SelectedTestBatch.TestTag);

                    if (records != null && records.Any())
                    {
                        if (AllChannels is not null) { AllChannels.Clear(); }
                        if (OriginalAllChannels is not null) { OriginalAllChannels.Clear(); }
                        AllChannels = new ObservableCollection<ChannelMapping>(records.OrderBy(c => c.TestId));
                        OriginalAllChannels = new ObservableCollection<ChannelMapping>(AllChannels);
                        await UpdateBatchInfoAsync();
                        UpdatePointStatistics();
                        await _messageService.ShowAsync("恢复成功", $"已成功恢复 {records.Count} 条测试记录", MessageBoxButton.OK);
                    }
                    else
                    {
                        StatusMessage = $"测试记录恢复失败"; 
                        await _messageService.ShowAsync("提示", "未找到要恢复的测试记录，或记录为空。", MessageBoxButton.OK);
                        if (wasPopupOpen) IsHistoryRecordsOpen = true; 
                    }
                }
                else 
                {
                    if (wasPopupOpen) IsHistoryRecordsOpen = true;
                }
            }
            catch (Exception ex)
            {
                StatusMessage = $"恢复测试记录时出错: {ex.Message}"; 
                System.Diagnostics.Debug.WriteLine($"恢复测试记录时出错: {ex.Message}");
                await _messageService.ShowAsync("错误", $"恢复测试记录时出错: {ex.Message}", MessageBoxButton.OK);
                if (wasPopupOpen) IsHistoryRecordsOpen = true; 
            }
            finally
            {
                IsLoading = false;
                if (!string.IsNullOrEmpty(StatusMessage) && StatusMessage != $"正在恢复测试记录 {SelectedTestBatch?.TestTag}...")
                {
                    // Don't clear a specific status message like "测试记录恢复失败"
                }
                else
                {
                    StatusMessage = string.Empty;
                }
            }
        }

        #endregion

        private async void ExecuteAllocateChannels()
        {
            try
            {
                IsLoading = true;
                StatusMessage = "正在分配通道...";

                if (AllChannels == null || !AllChannels.Any())
                {
                    await _messageService.ShowAsync("提示", "没有可用的通道需要分配。", MessageBoxButton.OK);
                    return;
                }

                var allocatedChannels = await _channelMappingService.AllocateChannelsTestAsync(new ObservableCollection<ChannelMapping>(AllChannels)); // 传递副本以防服务修改原始集合

                if (allocatedChannels != null)
                {
                    AllChannels = allocatedChannels;
                    OriginalAllChannels = new ObservableCollection<ChannelMapping>(AllChannels); // 更新原始备份

                    UpdateCurrentChannels();
                    RaisePropertyChanged(nameof(AllChannels));
                    UpdatePointStatistics();
                    await RefreshBatchesFromChannelsAsync(); // 分配后需要刷新批次列表

                    await _messageService.ShowAsync("成功", "通道分配完成。", MessageBoxButton.OK);
                }
                else
                {
                    await _messageService.ShowAsync("失败", "通道分配失败或未返回任何通道。", MessageBoxButton.OK);
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"分配通道时发生错误: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ExecuteAllocateChannels Error: {ex.Message}");
            }
            finally
            {
                IsLoading = false;
                StatusMessage = string.Empty;
            }
        }

        // 确保有一个方法可以从当前的 AllChannels 更新 Batches 集合
        private async Task RefreshBatchesFromChannelsAsync()
        {
            if (AllChannels == null) 
            {
                Batches = new ObservableCollection<BatchInfo>();
                SelectedBatch = null;
                RaisePropertyChanged(nameof(Batches));
                return;
            }
            var batchInfoList = await _channelMappingService.ExtractBatchInfoAsync(AllChannels);
            Batches = new ObservableCollection<BatchInfo>(batchInfoList.OrderBy(b => b.BatchName));
            
            if (SelectedBatch != null && !Batches.Any(b => b.BatchName == SelectedBatch.BatchName))
            {
                SelectedBatch = null; 
            }
            // else if (SelectedBatch == null && Batches.Any())
            // {
            //     SelectedBatch = Batches.FirstOrDefault(); // 可选：默认选择第一个
            // }
            RaisePropertyChanged(nameof(Batches));
        }

        private async void ClearChannelAllocationsAsync()
        {
            try
            {
                IsLoading = true;
                StatusMessage = "正在清除通道分配信息...";

                if (AllChannels == null || !AllChannels.Any())
                {
                    await _messageService.ShowAsync("提示", "没有通道分配信息需要清除。", MessageBoxButton.OK);
                    return;
                }

                var result = await _messageService.ShowAsync("确认操作", "确定要清除所有通道的分配信息吗？这将重置它们的测试状态。", MessageBoxButton.YesNo);
                if (result != MessageBoxResult.Yes)
                {
                    return;
                }
                
                // 服务层方法应该返回状态被 ChannelStateManager 重置后的通道集合
                var channelsAfterClearing = await _channelMappingService.ClearAllChannelAllocationsAsync(new ObservableCollection<ChannelMapping>(AllChannels)); 

                AllChannels = new ObservableCollection<ChannelMapping>(channelsAfterClearing);
                OriginalAllChannels = new ObservableCollection<ChannelMapping>(AllChannels); 
                
                // UI 更新
                UpdateCurrentChannels(); // 这会基于 SelectedChannelType 过滤新的 AllChannels
                                       // 如果 SelectedChannelType 本身依赖于批次，可能也需要重置
                RaisePropertyChanged(nameof(AllChannels)); 
                UpdatePointStatistics();
                await RefreshBatchesFromChannelsAsync(); // 批次信息会改变，SelectedBatch 可能会被置null
                
                // 重置与批次选择和测试流程相关的UI状态
                SelectedBatch = null; // 清除分配后，当前选定批次通常应失效
                // SelectedChannelType = null; // 如果通道类型选择依赖于批次，也应重置
                IsWiringCompleteBtnEnabled = true; // 清除后应可重新接线
                IsStartTestButtonEnabled = false; // 清除后不能直接开始测试
                // 其他可能依赖于通道分配状态的UI元素也应考虑重置

                await _messageService.ShowAsync("成功", "通道分配信息已清除。", MessageBoxButton.OK);
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"清除通道分配信息时发生错误: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ClearChannelAllocationsAsync Error: {ex.Message}");
            }
            finally
            {
                IsLoading = false;
                StatusMessage = string.Empty;
            }
        }

        private async void DeleteTestBatch()
        {
            if (SelectedTestBatch == null)
            {
                await _messageService.ShowAsync("提示", "请先选择一个测试批次", MessageBoxButton.OK); 
                return;
            }

            bool wasPopupOpen = IsHistoryRecordsOpen;
            try
            {
                IsHistoryRecordsOpen = false; 

                ConfirmDialogView confirmDialog = new ConfirmDialogView(
                    $"确定要删除测试批次 {SelectedTestBatch.TestTag} 的所有记录吗？此操作不可恢复。",
                    "确认删除");
                confirmDialog.Owner = Application.Current.MainWindow;
                bool? dialogResult = confirmDialog.ShowDialog();

                if (dialogResult == true)
                {
                    IsLoading = true;
                    StatusMessage = $"正在删除测试批次 {SelectedTestBatch.TestTag}...";

                    var success = await _testRecordService.DeleteTestBatchAsync(SelectedTestBatch.TestTag);

                    if (success)
                    {
                        TestBatches.Remove(SelectedTestBatch);
                        SelectedTestBatch = TestBatches.FirstOrDefault();
                        await _messageService.ShowAsync("删除成功", "测试批次已成功删除", MessageBoxButton.OK);
                    }
                    else
                    {
                        await _messageService.ShowAsync("删除失败", "删除测试批次失败", MessageBoxButton.OK);
                    }
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"删除测试批次时出错: {ex.Message}", MessageBoxButton.OK);
            }
            finally
            {
                if (wasPopupOpen) 
                {
                    IsHistoryRecordsOpen = true;
                }
                IsLoading = false;
                StatusMessage = string.Empty;
            }
        }

        /// <summary>
        /// 启动测试
        /// </summary>
        /// <remarks>
        /// 此方法用于启动测试，执行以下操作：
        /// 1. 验证是否选择了测试批次
        /// 2. 检查批次状态是否为"接线已确认"
        /// 3. 筛选出状态为"等待测试"的通道
        /// 4. 调用TestTaskManager启动测试
        /// </remarks>
        private async void StartTest()
        {
            try
            {
                IsLoading = true;
                StatusMessage = "正在启动测试...";

                if (SelectedBatch == null || string.IsNullOrEmpty(SelectedBatch.BatchName))
                {
                    await _messageService.ShowAsync("提示", "请先选择一个测试批次", MessageBoxButton.OK);
                    return;
                }

                if (SelectedBatch.Status != "接线已确认")
                {
                    // await _messageService.ShowAsync("提示", $"批次 '{SelectedBatch.BatchName}' 的接线尚未确认, 或状态为 '{SelectedBatch.Status}', 不能开始测试。", MessageBoxButton.OK);
                    return;
                }

                var channelsToTest = AllChannels
                    .Where(c => c.TestBatch == SelectedBatch.BatchName && c.HardPointTestResult == "等待测试")
                    .ToList();

                if (!channelsToTest.Any())
                {
                    await _messageService.ShowAsync("提示", "当前批次没有需要测试的通道状态为等待测试", MessageBoxButton.OK);
                    return;
                }

                // 调用TestTaskManager启动测试
                // TestTaskManager内部会调用IChannelStateManager.BeginHardPointTest来更新通道状态，包括StartTime
                await _testTaskManager.StartAllTasksAsync(channelsToTest);

                // 移除VM中直接修改通道状态的逻辑
                // foreach (var channel in channelsToTest)
                // {
                //     // channel.StartTime = DateTime.Now; // <<-- 已移除，由ChannelStateManager处理
                //     // channel.TestResultStatus = 0; // 标记为测试中，或由ChannelStateManager处理
                //     // channel.ResultText = "硬点测试中...";
                // }

                StatusMessage = $"批次 '{SelectedBatch.BatchName}' 的测试已启动。";
                IsStartTestButtonEnabled = false; // 测试启动后，禁用开始按钮，直到当前测试完成或可进行其他操作
                RefreshBatchStatus(); // 更新批次状态，可能会变为"测试中"
                UpdatePointStatistics();
                // ExportTestResultsCommand.RaiseCanExecuteChanged(); // 根据实际逻辑，测试开始后可能还不能导出
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"启动测试失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"StartTest Error: {ex.Message}");
            }
            finally
            {
                IsLoading = false;
                // StatusMessage = string.Empty; // 保留测试已启动的消息
            }
        }

        /// <summary>
        /// 重新测试
        /// </summary>
        /// <param name="channelToRetest">需要重新测试的通道</param>
        /// <remarks>
        /// 此方法用于重新测试，执行以下操作：
        /// 1. 验证是否选择了测试批次
        /// 2. 检查批次状态是否为"接线已确认"
        /// 3. 筛选出状态为"等待测试"的通道
        /// 4. 调用TestTaskManager进行重新测试
        /// </remarks>
        private async void Retest(ChannelMapping channelToRetest)
        {
            try
            {
                IsLoading = true;
                StatusMessage = $"正在准备重新测试通道: {channelToRetest.VariableName}...";

                // 调用TestTaskManager进行重新测试
                // TestTaskManager内部会调用IChannelStateManager.ResetForRetest来重置通道状态
                await _testTaskManager.RetestChannelAsync(channelToRetest);

                // 移除VM中直接修改通道状态的逻辑
                // channelToRetest.ResultText = "正在尝试重新测试..."; // <<-- 已移除，由ChannelStateManager处理
                // channelToRetest.TestResultStatus = 0; // Reset status
                // channelToRetest.HardPointTestResult = "未测试"; // Or "等待测试"
                // // Reset all sub-test statuses etc. This is now ChannelStateManager's job.
                // channelToRetest.FinalTestTime = null;

                // UI 更新应通过事件机制（如OnChannelStatesModified）或在TestTaskManager调用后显式进行
                // 此处仅设置一个临时状态消息
                StatusMessage = $"通道: {channelToRetest.VariableName} 已准备重新测试。";

                // RefreshBatchStatus(); // 状态已由CSM处理并通过事件更新
                // UpdatePointStatistics();
                // ExportTestResultsCommand.RaiseCanExecuteChanged();
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"重新测试失败: {ex.Message}", MessageBoxButton.OK);
                 System.Diagnostics.Debug.WriteLine($"Retest Error: {ex.Message}");
            }
            finally
            {
                IsLoading = false;
                // StatusMessage = string.Empty; // 保留重新测试准备消息
            }
        }

        /// <summary>
        /// 确认批次选择
        /// </summary>
        /// <remarks>
        /// 此方法用于确认批次选择，执行以下操作：
        /// 1. 验证是否选择了测试批次
        /// 2. 应用选择的批次，筛选通道列表
        /// 3. 更新AllChannels集合和原始通道集合
        /// 4. 更新当前显示的通道列表
        /// 5. 更新点位统计和批次状态
        /// 6. 根据选定批次的状态，更新UI按钮的可用性
        /// 7. 关闭批次选择弹窗
        /// </remarks>
        private async void ConfirmBatchSelection()
        {
            IsLoading = true;
            StatusMessage = "正在确认批次选择...";
            try
            {
                if (SelectedBatch == null)
                {
                    await _messageService.ShowAsync("提示", "请选择一个批次", MessageBoxButton.OK);
                    return;
                }

                // 根据 SelectedBatch 调整视图集合和测试队列
                UpdateCurrentChannels();
                TestQueue = new ObservableCollection<ChannelMapping>(CurrentChannels);
                TestQueuePosition = 0;
                TestQueueStatus = TestQueue.Any() ? "已加载批次队列" : "队列为空";

                // 更新点位统计和批次状态
                UpdatePointStatistics(); // 基于新的 AllChannels 更新统计
                RefreshBatchStatus();    // 刷新当前选中批次的状态显示

                // 根据选定批次的状态，更新UI按钮的可用性
                IsWiringCompleteBtnEnabled = (SelectedBatch.Status == "未开始" || SelectedBatch.Status == "测试中" || SelectedBatch.Status == "接线已确认");
                
                // "通道硬点自动测试" 按钮的启用逻辑：
                // 1. 批次状态必须是 "接线已确认"
                // 2. 且该批次中至少有一个通道的硬点测试结果是 "等待测试"
                IsStartTestButtonEnabled = SelectedBatch.Status == "接线已确认" &&
                                           AllChannels.Any(c => c.TestBatch == SelectedBatch.BatchName && c.HardPointTestResult == "等待测试");


                // 之前在这里有一段逻辑，如果批次状态为 "未开始"，则将批次内所有通道状态更新为 "等待测试"
                // 这段逻辑已移除，因为状态的变更（从未测试到等待测试）应该由 FinishWiring (接线完成) 动作触发，
                // 并通过 TestTaskManager -> IChannelStateManager.PrepareForWiringConfirmation 来执行。
                // 直接在选择批次时更改状态不符合流程。

                IsBatchSelectionOpen = false; // 关闭批次选择弹窗
                StatusMessage = $"已选择批次: {SelectedBatch.BatchName}";

                // 更新测试队列只显示当前批次
                TestQueue = new ObservableCollection<ChannelMapping>(CurrentChannels);
                TestQueuePosition = 0;
                TestQueueStatus = TestQueue.Any()?"已加载批次队列":"队列为空";
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"确认批次选择失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ConfirmBatchSelection Error: {ex.Message}");
            }
            finally
            {
                IsLoading = false;
                // StatusMessage = string.Empty; // 保留批次选择成功消息
            }
        }

        #region Helper Methods (Placeholders for missing methods)

        private void UpdateCurrentChannels()
        {
            if (AllChannels == null)
            {
                CurrentChannels = new ObservableCollection<ChannelMapping>();
                TestQueue = new ObservableCollection<ChannelMapping>(CurrentChannels);
                TestQueuePosition = 0;
                TestQueueStatus = "队列为空";
                return;
            }

            // 1) 按批次过滤
            IEnumerable<ChannelMapping> batchFilteredChannels = SelectedBatch == null
                ? AllChannels
                : AllChannels.Where(c => c.TestBatch == SelectedBatch.BatchName);

            // 2) 按通道类型过滤
            IEnumerable<ChannelMapping> typeFilteredChannels;
            switch (SelectedChannelType)
            {
                case "AI通道":
                    typeFilteredChannels = batchFilteredChannels.Where(c => c.ModuleType?.ToUpper() == "AI");
                    break;
                case "AO通道":
                    typeFilteredChannels = batchFilteredChannels.Where(c => c.ModuleType?.ToUpper() == "AO");
                    break;
                case "DI通道":
                    typeFilteredChannels = batchFilteredChannels.Where(c => c.ModuleType?.ToUpper() == "DI");
                    break;
                case "DO通道":
                    typeFilteredChannels = batchFilteredChannels.Where(c => c.ModuleType?.ToUpper() == "DO");
                    break;
                case "AINone通道":
                    typeFilteredChannels = batchFilteredChannels.Where(c => c.ModuleType?.ToUpper() == "AINONE");
                    break;
                case "AONone通道":
                    typeFilteredChannels = batchFilteredChannels.Where(c => c.ModuleType?.ToUpper() == "AONONE");
                    break;
                case "DINone通道":
                    typeFilteredChannels = batchFilteredChannels.Where(c => c.ModuleType?.ToUpper() == "DINONE");
                    break;
                case "DONone通道":
                    typeFilteredChannels = batchFilteredChannels.Where(c => c.ModuleType?.ToUpper() == "DONONE");
                    break;
                default: // "所有类型" 或其他未指定情况
                    typeFilteredChannels = batchFilteredChannels;
                    break;
            }

            // 3) 按测试结果过滤
            IEnumerable<ChannelMapping> resultFilteredChannels;
            switch (SelectedResultFilter)
            {
                case "通过":
                    resultFilteredChannels = typeFilteredChannels.Where(c => c.OverallStatus == OverallResultStatus.Passed);
                    break;
                case "失败":
                    resultFilteredChannels = typeFilteredChannels.Where(c => c.OverallStatus == OverallResultStatus.Failed);
                    break;
                case "未测试":
                    resultFilteredChannels = typeFilteredChannels.Where(c => c.OverallStatus == OverallResultStatus.NotTested || c.OverallStatus == OverallResultStatus.InProgress || c.HardPointTestResult == "等待测试");
                    break;
                case "跳过":
                    resultFilteredChannels = typeFilteredChannels.Where(c => c.OverallStatus == OverallResultStatus.Skipped);
                    break;
                default: // "所有结果" 或其他未指定情况
                    resultFilteredChannels = typeFilteredChannels;
                    break;
            }

            CurrentChannels = new ObservableCollection<ChannelMapping>(resultFilteredChannels.OrderBy(c => c.TestId));

            // 同步更新测试队列
            // TestQueue 的更新也应该在 ConfirmBatchSelection 方法中，确保 TestQueue 和 CurrentChannels 一致。
            // 在 UpdateCurrentChannels 内部直接更新 TestQueue 是可以的，
            // 因为 ConfirmBatchSelection, OnBatchSelected, ApplyResultFilter 都会调用此方法。
            TestQueue = new ObservableCollection<ChannelMapping>(CurrentChannels);
            TestQueuePosition = 0;
            TestQueueStatus = TestQueue.Any() ? "已加载批次队列" : "队列为空";
        }

        private void UpdatePointStatistics()
        {
            // Placeholder: Logic to update point count statistics
            if (AllChannels == null)
            {
                TotalPointCount   = "总点位: 0";
                TestedPointCount  = "已测试点位: 0";
                WaitingPointCount = "待测试点位: 0";
                SuccessPointCount = "成功点位: 0";
                FailurePointCount = "失败点位: 0";
                return;
            }

            int total    = AllChannels.Count;
            int tested   = AllChannels.Count(c => c.OverallStatus == OverallResultStatus.Passed || c.OverallStatus == OverallResultStatus.Failed || c.OverallStatus == OverallResultStatus.Skipped);
            int waiting  = AllChannels.Count(c => c.HardPointTestResult == "等待测试" || c.OverallStatus == OverallResultStatus.NotTested || c.OverallStatus == OverallResultStatus.InProgress);
            int success  = AllChannels.Count(c => c.OverallStatus == OverallResultStatus.Passed);
            int failure  = AllChannels.Count(c => c.OverallStatus == OverallResultStatus.Failed);

            TotalPointCount   = $"总点位: {total}";
            TestedPointCount  = $"已测试点位: {tested}";
            WaitingPointCount = $"待测试点位: {waiting}";
            SuccessPointCount = $"成功点位: {success}";
            FailurePointCount = $"失败点位: {failure}";
        }

        private async Task RefreshBatchStatus() // Made async Task due to internal async calls
        {
            // Placeholder: Logic to refresh batch status information
            // This might involve re-calculating status based on AllChannels
            // and then updating the Batches collection or the SelectedBatch.
            await UpdateBatchInfoAsync(); // This method was already present and seems relevant
        }

        private void OnBatchSelected()
        {
            // Placeholder: Logic for when a batch is selected.
            // This might involve updating IsWiringCompleteBtnEnabled, IsStartTestButtonEnabled,
            // and filtering channels based on the new SelectedBatch.
            if (SelectedBatch == null) return;

            // 当批次选择变化时，直接调用UpdateCurrentChannels，它会处理所有过滤条件
            UpdateCurrentChannels(); 
        }

        private void ApplyResultFilter()
        {
            // 此方法现在由 UpdateCurrentChannels() 统一处理。
            // 所有对 CurrentChannels 的更新都应通过 UpdateCurrentChannels()。
            UpdateCurrentChannels();
        }

        private async void FinishWiring() 
        {
            System.Diagnostics.Debug.WriteLine("DataEditViewModel.FinishWiring() 方法开始执行。");
            if (SelectedBatch == null) 
            {
                await _messageService.ShowAsync("提示", "请先选择一个批次进行接线确认。", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine("DataEditViewModel.FinishWiring(): 未选择批次，操作中止。");
                return;
            }
            var channelsInBatch = AllChannels?.Where(c => c.TestBatch == SelectedBatch.BatchName).ToList() ?? new List<ChannelMapping>();
            System.Diagnostics.Debug.WriteLine($"DataEditViewModel.FinishWiring(): 选定批次 {SelectedBatch.BatchName}，包含 {channelsInBatch.Count} 个通道。");

            // 调用 TestTaskManager 准备通道状态
            // TestTaskManager 内部会调用 ChannelStateManager，并最终通过事件更新UI和ViewModel状态
            // isConfirmed 参数设为 false，因为测试不应由此调用自动开始
            await _testTaskManager.ConfirmWiringCompleteAsync(SelectedBatch, false, channelsInBatch); 
            System.Diagnostics.Debug.WriteLine("DataEditViewModel.FinishWiring(): _testTaskManager.ConfirmWiringCompleteAsync 调用完成。");

            // 接线完成按钮在点击后通常应禁用，直到状态允许再次操作
            IsWiringCompleteBtnEnabled = false; 
            // (ConfirmWiringCompleteCommand as DelegateCommand)?.RaiseCanExecuteChanged(); // 假设 ConfirmWiringCompleteCommand 是 DelegateCommand
            ConfirmWiringCompleteCommand.RaiseCanExecuteChanged();


            // IsStartTestButtonEnabled 的状态将由 OnChannelStatesModified 根据实际通道状态来更新
            // RefreshBatchStatus() 也将由事件触发，无需在此处显式调用
            System.Diagnostics.Debug.WriteLine("DataEditViewModel.FinishWiring() 方法执行完毕。");
        }

        private bool CanExecuteConfirmWiringComplete()
        {
            // Placeholder: Logic to determine if ConfirmWiringComplete can execute
            return SelectedBatch != null && (SelectedBatch.Status == "未开始" || SelectedBatch.Status == "测试中");
        }

        private async void CancelBatchSelection() // Placeholder for DelegateCommand. Made async to align with potential ShowAsync calls.
        {
            IsBatchSelectionOpen = false;
            // Potentially reset selected batch or other UI states if needed.
        }

        /// <summary>
        /// 导出测试结果到 Excel
        /// </summary>
        private async void ExportTestResults()
        {
            try
            {
                // 1) 确认有通道数据
                if (AllChannels == null || !AllChannels.Any())
                {
                    await _messageService.ShowAsync("导出失败", "没有可导出的测试结果数据。", MessageBoxButton.OK);
                    return;
                }

                // 2) 根据当前批次过滤（如果已选择批次）
                IEnumerable<ChannelMapping> channelsToExport = SelectedBatch == null
                    ? AllChannels
                    : AllChannels.Where(c => c.TestBatch == SelectedBatch.BatchName);

                // 3) 仅导出已完成（通过 / 失败 / 跳过）的通道
                var completedChannels = channelsToExport.Where(c =>
                    c.OverallStatus == OverallResultStatus.Passed ||
                    c.OverallStatus == OverallResultStatus.Failed ||
                    c.OverallStatus == OverallResultStatus.Skipped).ToList();

                if (!completedChannels.Any())
                {
                    await _messageService.ShowAsync("导出提示", "当前筛选范围内没有已完成的测试点位可导出。", MessageBoxButton.OK);
                    return;
                }

                // 4) 调用导出服务。若 filePath 传 null，则服务内部会弹出保存对话框。
                bool success = await _testResultExportService.ExportToExcelAsync(completedChannels, null);

                if (success)
                {
                    await _messageService.ShowAsync("成功", "测试结果导出成功！", MessageBoxButton.OK);
                }
                else
                {
                    await _messageService.ShowAsync("失败", "测试结果导出失败，请检查日志或重试。", MessageBoxButton.OK);
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("异常", $"导出过程中发生错误: {ex.Message}", MessageBoxButton.OK);
            }
        }

        /// <summary>
        /// 判断当前是否可以导出测试结果
        /// </summary>
        private bool CanExportTestResults()
        {
            if (AllChannels == null || !AllChannels.Any()) return false;

            IEnumerable<ChannelMapping> channels = SelectedBatch == null
                ? AllChannels
                : AllChannels.Where(c => c.TestBatch == SelectedBatch.BatchName);

            return channels.Any(c => c.OverallStatus == OverallResultStatus.Passed ||
                                      c.OverallStatus == OverallResultStatus.Failed ||
                                      c.OverallStatus == OverallResultStatus.Skipped);
        }

        // Placeholders for Manual Test command targets
        private async void OpenDIManualTest(ChannelMapping channel) 
        { 
            if (channel != null && (channel.ModuleType?.ToLower() == "di" || channel.ModuleType?.ToLower() == "dinone"))
            {
                CurrentChannel = channel;
                CurrentTestResult = channel;
                _channelStateManager.BeginManualTest(channel); 
                RaisePropertyChanged(nameof(CurrentChannel));
                CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                UpdatePointStatistics();
                RefreshBatchStatus(); 
                ExportTestResultsCommand.RaiseCanExecuteChanged();
                IsDIManualTestOpen = true; 
                DICurrentValue = string.Empty;
                // Add monitoring loop if needed, similar to AI/AO/DO
            }
            else
            {
                await _messageService.ShowAsync("错误", "所选通道不是有效的DI类型通道。", MessageBoxButton.OK);
            }
        }
        private void ExecuteCloseDOManualTest() 
        { 
            try
            {
                // 设置DO手动测试窗口打开状态为false
                IsDOManualTestOpen = false; 

                // 停止DO数值监控
                _manualTestIoService.StopAll();

                // 清空DO当前值显示
                DOCurrentValue = string.Empty;
                DOMonitorStatus = "开始监测";

                // 检查当前通道是否通过了测试
                if (CurrentChannel != null && CurrentChannel.ShowValueStatus == "通过")
                {
                    _testTaskManager.CompleteAllTestsAsync();
                }

                // 刷新批次状态
                RefreshBatchStatus();

                // 更新点位统计
                UpdatePointStatistics();
            }
            catch (Exception ex)
            {
                _ = _messageService.ShowAsync("错误", $"关闭DO手动测试窗口失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        private async void ExecuteSendAIMaintenance(ChannelMapping channel) 
        { 
            if (channel == null || string.IsNullOrEmpty(channel.MaintenanceEnableSwitchPointCommAddress))
            {
                await _messageService.ShowAsync("提示", "未配置AI维护使能开关点位地址。", MessageBoxButton.OK);
                return;
            }
            try
            {
                await _targetPlc.WriteDigitalValueAsync(channel.MaintenanceEnableSwitchPointCommAddress, true);
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"发送AI维护使能信号失败: {ex.Message}", MessageBoxButton.OK);
            }
        } 
        private async void ExecuteResetAIMaintenance(ChannelMapping channel) 
        { 
             if (channel == null || string.IsNullOrEmpty(channel.MaintenanceEnableSwitchPointCommAddress))
            {
                await _messageService.ShowAsync("提示", "未配置AI维护使能开关点位地址。", MessageBoxButton.OK);
                return;
            }
            try
            {
                await _targetPlc.WriteDigitalValueAsync(channel.MaintenanceEnableSwitchPointCommAddress, false);
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"复位AI维护使能信号失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        private void ExecuteConfirmAIMaintenance(ChannelMapping channel) 
        { 
            if (channel != null) 
            {
                _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.MaintenanceFunction, true, DateTime.Now);
                RaisePropertyChanged(nameof(CurrentChannel));
                CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                UpdatePointStatistics();
                RefreshBatchStatus();
                ExportTestResultsCommand.RaiseCanExecuteChanged();
            }
        }
        private void ExecuteConfirmAITrendCheck(ChannelMapping channel) 
        { 
            if (channel != null) 
            {
                _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.TrendCheck, true, DateTime.Now);
                RaisePropertyChanged(nameof(CurrentChannel));
                CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                UpdatePointStatistics();
                RefreshBatchStatus();
                ExportTestResultsCommand.RaiseCanExecuteChanged();
            }
        }
        private void ExecuteConfirmAIReportCheck(ChannelMapping channel) 
        { 
            if (channel != null) 
            {
                _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.ReportCheck, true, DateTime.Now);
                RaisePropertyChanged(nameof(CurrentChannel));
                CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                UpdatePointStatistics();
                RefreshBatchStatus();
                ExportTestResultsCommand.RaiseCanExecuteChanged();
            }
        }
        private void ExecuteConfirmAOTrendCheck(ChannelMapping channel) 
        { 
            if (channel != null) 
            {
                _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.TrendCheck, true, DateTime.Now);
                RaisePropertyChanged(nameof(CurrentChannel));
                CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                UpdatePointStatistics();
                RefreshBatchStatus();
                ExportTestResultsCommand.RaiseCanExecuteChanged();
            }
        }
        private void ExecuteConfirmAOReportCheck(ChannelMapping channel) 
        { 
            if (channel != null) 
            {
                _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.ReportCheck, true, DateTime.Now);
                RaisePropertyChanged(nameof(CurrentChannel));
                CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                UpdatePointStatistics();
                RefreshBatchStatus();
                ExportTestResultsCommand.RaiseCanExecuteChanged();
            }
        }
        private async void ExecuteStartDOMonitor(ChannelMapping channel) 
        { 
            try
            {
                if (channel != null && channel.ShowValueStatus != "通过" && IsDOManualTestOpen)
                {
                    // 设置当前监测的DO通道
                    CurrentChannel = channel;
                    CurrentTestResult = channel;
                    
                    // DO监控已在OpenDOManualTest中通过服务启动，此处仅更新状态
                    DOMonitorStatus = "停止监测";
                    
                    // 如果需要重新启动监控，可以调用服务的监控方法
                    if (string.IsNullOrEmpty(DOCurrentValue) || DOCurrentValue == "监控异常")
                    {
                        _manualTestIoService.StartDOValueMonitoring(CurrentChannel, (currentValue) =>
                        {
                            Application.Current.Dispatcher.Invoke(() =>
                            {
                                DOCurrentValue = currentValue;
                            });
                        });
                    }
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"启动DO监测失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
        private async void ExecuteConfirmDO(ChannelMapping channel) 
        { 
            try
            {
                if (channel != null) 
                {
                    _channelStateManager.SetManualSubTestOutcome(channel, ManualTestItem.ShowValue, true, DateTime.Now);

                    // 检查通道是否完成所有测试，如果是则自动保存
                    await CheckAndSaveCompletedChannelAsync(channel);

                    RaisePropertyChanged(nameof(CurrentChannel));
                    RaisePropertyChanged(nameof(AllChannels));
                    CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                    UpdatePointStatistics();
                    RefreshBatchStatus();
                    ExportTestResultsCommand.RaiseCanExecuteChanged();
                }
            }
            catch (Exception ex)
            {
                await _messageService.ShowAsync("错误", $"确认DO测试失败: {ex.Message}", MessageBoxButton.OK);
                System.Diagnostics.Debug.WriteLine($"ExecuteConfirmDO Error: {ex.Message}");
            }
        }

        #endregion

        /// <summary>
        /// 查看硬点测试错误详情
        /// </summary>
        /// <param name="channel">相关通道</param>
        private async void ExecuteShowErrorDetail(ChannelMapping channel)
        {
            try
            {
                if (channel != null && !string.IsNullOrWhiteSpace(channel.HardPointErrorDetail))
                {
                    await _messageService.ShowAsync("错误详情", channel.HardPointErrorDetail, System.Windows.MessageBoxButton.OK);
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"显示错误详情失败: {ex.Message}");
            }
        }

        /// <summary>
        /// 打开DO通道手动测试窗口
        /// </summary>
        /// <param name="channel">需要手动测试的DO通道</param>
        /// <remarks>
        /// 该方法打开DO通道的手动测试窗口，执行以下操作：
        /// 1. 设置当前选中的通道
        /// 2. 初始化手动测试状态
        /// 3. 打开DO手动测试窗口
        /// 4. 启动DO数值监控服务
        /// </remarks>
        private async void OpenDOManualTest(ChannelMapping channel)
        {
            try
            {
                if (channel != null && (channel.ModuleType?.ToLower() == "do" || channel.ModuleType?.ToLower() == "donone"))
                {
                    CurrentChannel = channel;
                    CurrentTestResult = channel;

                    _channelStateManager.BeginManualTest(channel);

                    RaisePropertyChanged(nameof(CurrentChannel));
                    RaisePropertyChanged(nameof(AllChannels));
                    CollectionViewSource.GetDefaultView(AllChannels)?.Refresh();
                    UpdatePointStatistics();
                    RefreshBatchStatus();
                    ExportTestResultsCommand.RaiseCanExecuteChanged();

                    IsDOManualTestOpen = true;
                    DOCurrentValue = string.Empty; // 清空上次的值
                    DOMonitorStatus = "停止监测";

                    // 启动DO数值监控服务，每 0.5 秒读取一次并更新绑定值
                    _manualTestIoService.StartDOValueMonitoring(CurrentChannel, (currentValue) =>
                    {
                        Application.Current.Dispatcher.Invoke(() =>
                        {
                            DOCurrentValue = currentValue;
                        });
                    });
                }
                else
                {
                    await _messageService.ShowAsync("错误", "所选通道不是有效的DO类型通道。", MessageBoxButton.OK);
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"打开DO手动测试窗口失败: {ex.Message}");
                await _messageService.ShowAsync("错误", $"打开DO手动测试窗口失败: {ex.Message}", MessageBoxButton.OK);
            }
        }
    }

    public class ModuleInfo : Prism.Mvvm.BindableBase 
    {
        private string _moduleName;
        public string ModuleName { get => _moduleName; set => SetProperty(ref _moduleName, value); }

        private string _stationName; 
        public string StationName { get => _stationName; set => SetProperty(ref _stationName, value); }

        private string _moduleType;
        public string ModuleType { get => _moduleType; set => SetProperty(ref _moduleType, value); }

        private int _channelCount;
        public int ChannelCount { get => _channelCount; set => SetProperty(ref _channelCount, value); }

        private bool _isSelected;
        public bool IsSelected { get => _isSelected; set => SetProperty(ref _isSelected, value); }
    }
}
