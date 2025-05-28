using Prism.Mvvm;
using System;
using FatFullVersion.Shared;

namespace FatFullVersion.Models
{
    /// <summary>
    /// 通道映射模型，用于管理和存储通道映射相关信息
    /// </summary>
    public class ChannelMapping : BindableBase
    {
        #region 从ExcelPointData添加的属性

        private Guid _id;
        /// <summary>
        /// 本次测试标识
        /// </summary>
        public Guid Id
        {
            get { return _id; }
            set { SetProperty(ref _id, value); }
        }

        private string _testTag;
        /// <summary>
        /// 本次测试标识
        /// </summary>
        public string TestTag
        {
            get { return _testTag; }
            set { SetProperty(ref _testTag, value); }
        }

        private string _moduleName;
        /// <summary>
        /// 模块名称
        /// </summary>
        public string ModuleName
        {
            get { return _moduleName; }
            set { SetProperty(ref _moduleName, value); }
        }
        
        private string _moduleType;
        /// <summary>
        /// 模块类型
        /// </summary>
        public string ModuleType
        {
            get { return _moduleType; }
            set { SetProperty(ref _moduleType, value); }
        }
        
        private string _powerSupplyType;
        /// <summary>
        /// 供电类型（有源/无源）
        /// </summary>
        public string PowerSupplyType
        {
            get { return _powerSupplyType; }
            set { SetProperty(ref _powerSupplyType, value); }
        }
        
        private string _wireSystem;
        /// <summary>
        /// 线制
        /// </summary>
        public string WireSystem
        {
            get { return _wireSystem; }
            set { SetProperty(ref _wireSystem, value); }
        }
        
        private string _tag;
        /// <summary>
        /// 位号
        /// </summary>
        public string Tag
        {
            get { return _tag; }
            set { SetProperty(ref _tag, value); }
        }
        
        private string _stationName;
        /// <summary>
        /// 场站名
        /// </summary>
        public string StationName
        {
            get { return _stationName; }
            set { SetProperty(ref _stationName, value); }
        }

        private string _variableName;

        /// <summary>
        /// 变量名称(HMI)
        /// </summary>
        public string VariableName
        {
            get { return _variableName; }
            set { SetProperty(ref _variableName, value); }
        }

        private string _variableDescription;
        /// <summary>
        /// 变量描述
        /// </summary>
        public string VariableDescription
        {
            get { return _variableDescription; }
            set { SetProperty(ref _variableDescription, value); }
        }
        
        private string _dataType;
        /// <summary>
        /// 数据类型
        /// </summary>
        public string DataType
        {
            get { return _dataType; }
            set { SetProperty(ref _dataType, value); }
        }

        private string _channelTag;
        /// <summary>
        /// 通道位号
        /// </summary>
        public string ChannelTag
        {
            get { return _channelTag; }
            set { SetProperty(ref _channelTag, value); }
        }

        private string _accessProperty;
        /// <summary>
        /// 读写属性
        /// </summary>
        public string AccessProperty
        {
            get { return _accessProperty; }
            set { SetProperty(ref _accessProperty, value); }
        }
        
        private string _saveHistory;
        /// <summary>
        /// 保存历史
        /// </summary>
        public string SaveHistory
        {
            get { return _saveHistory; }
            set { SetProperty(ref _saveHistory, value); }
        }
        
        private string _powerFailureProtection;
        /// <summary>
        /// 掉电保护
        /// </summary>
        public string PowerFailureProtection
        {
            get { return _powerFailureProtection; }
            set { SetProperty(ref _powerFailureProtection, value); }
        }
        
        private string _rangeLowerLimit;
        /// <summary>
        /// 量程低限
        /// </summary>
        public string RangeLowerLimit
        {
            get { return _rangeLowerLimit; }
            set { SetProperty(ref _rangeLowerLimit, value); }
        }
        
        private float? _rangeLowerLimitValue;
        /// <summary>
        /// 量程低限数值 (AI/AO必填，统一为float?)
        /// </summary>
        public float? RangeLowerLimitValue
        {
            get { return _rangeLowerLimitValue; }
            set { SetProperty(ref _rangeLowerLimitValue, value); }
        }
        
        private string _rangeUpperLimit;
        /// <summary>
        /// 量程高限
        /// </summary>
        public string RangeUpperLimit
        {
            get { return _rangeUpperLimit; }
            set { SetProperty(ref _rangeUpperLimit, value); }
        }
        
        private float? _rangeUpperLimitValue;
        /// <summary>
        /// 量程高限数值 (AI/AO必填，统一为float?)
        /// </summary>
        public float? RangeUpperLimitValue
        {
            get { return _rangeUpperLimitValue; }
            set { SetProperty(ref _rangeUpperLimitValue, value); }
        }
        
        private string _sllSetValue;
        /// <summary>
        /// SLL设定值
        /// </summary>
        public string SLLSetValue
        {
            get { return _sllSetValue; }
            set { SetProperty(ref _sllSetValue, value); }
        }
        
        private float? _sllSetValueNumber;
        /// <summary>
        /// SLL设定值数值 (由SLLSetValue解析，可为空)
        /// </summary>
        public float? SLLSetValueNumber
        {
            get { return _sllSetValueNumber; }
            set { SetProperty(ref _sllSetValueNumber, value); RaisePropertyChanged(nameof(LowLowLimit)); }
        }
        
        private string _sllSetPoint;
        /// <summary>
        /// SLL设定点位
        /// </summary>
        public string SLLSetPoint
        {
            get { return _sllSetPoint; }
            set { SetProperty(ref _sllSetPoint, value); }
        }
        
        private string _sllSetPointPLCAddress;
        /// <summary>
        /// SLL设定点位_PLC地址
        /// </summary>
        public string SLLSetPointPLCAddress
        {
            get { return _sllSetPointPLCAddress; }
            set { SetProperty(ref _sllSetPointPLCAddress, value); }
        }
        
        private string _sllSetPointCommAddress;
        /// <summary>
        /// SLL设定点位_通讯地址
        /// </summary>
        public string SLLSetPointCommAddress
        {
            get { return _sllSetPointCommAddress; }
            set { SetProperty(ref _sllSetPointCommAddress, value); }
        }
        
        private string _slSetValue;
        /// <summary>
        /// SL设定值
        /// </summary>
        public string SLSetValue
        {
            get { return _slSetValue; }
            set { SetProperty(ref _slSetValue, value); }
        }
        
        private float? _slSetValueNumber;
        /// <summary>
        /// SL设定值数值 (由SLSetValue解析，可为空)
        /// </summary>
        public float? SLSetValueNumber
        {
            get { return _slSetValueNumber; }
            set { SetProperty(ref _slSetValueNumber, value); RaisePropertyChanged(nameof(LowLimit)); }
        }
        
        private string _slSetPoint;
        /// <summary>
        /// SL设定点位
        /// </summary>
        public string SLSetPoint
        {
            get { return _slSetPoint; }
            set { SetProperty(ref _slSetPoint, value); }
        }
        
        private string _slSetPointPLCAddress;
        /// <summary>
        /// SL设定点位_PLC地址
        /// </summary>
        public string SLSetPointPLCAddress
        {
            get { return _slSetPointPLCAddress; }
            set { SetProperty(ref _slSetPointPLCAddress, value); }
        }
        
        private string _slSetPointCommAddress;
        /// <summary>
        /// SL设定点位_通讯地址
        /// </summary>
        public string SLSetPointCommAddress
        {
            get { return _slSetPointCommAddress; }
            set { SetProperty(ref _slSetPointCommAddress, value); }
        }
        
        private string _shSetValue;
        /// <summary>
        /// SH设定值
        /// </summary>
        public string SHSetValue
        {
            get { return _shSetValue; }
            set { SetProperty(ref _shSetValue, value); }
        }
        
        private float? _shSetValueNumber;
        /// <summary>
        /// SH设定值数值 (由SHSetValue解析，可为空)
        /// </summary>
        public float? SHSetValueNumber
        {
            get { return _shSetValueNumber; }
            set { SetProperty(ref _shSetValueNumber, value); RaisePropertyChanged(nameof(HighLimit)); }
        }
        
        private string _shSetPoint;
        /// <summary>
        /// SH设定点位
        /// </summary>
        public string SHSetPoint
        {
            get { return _shSetPoint; }
            set { SetProperty(ref _shSetPoint, value); }
        }
        
        private string _shSetPointPLCAddress;
        /// <summary>
        /// SH设定点位_PLC地址
        /// </summary>
        public string SHSetPointPLCAddress
        {
            get { return _shSetPointPLCAddress; }
            set { SetProperty(ref _shSetPointPLCAddress, value); }
        }
        
        private string _shSetPointCommAddress;
        /// <summary>
        /// SH设定点位_通讯地址
        /// </summary>
        public string SHSetPointCommAddress
        {
            get { return _shSetPointCommAddress; }
            set { SetProperty(ref _shSetPointCommAddress, value); }
        }
        
        private string _shhSetValue;
        /// <summary>
        /// SHH设定值
        /// </summary>
        public string SHHSetValue
        {
            get { return _shhSetValue; }
            set { SetProperty(ref _shhSetValue, value); }
        }
        
        private float? _shhSetValueNumber;
        /// <summary>
        /// SHH设定值数值 (由SHHSetValue解析，可为空)
        /// </summary>
        public float? SHHSetValueNumber
        {
            get { return _shhSetValueNumber; }
            set { SetProperty(ref _shhSetValueNumber, value); RaisePropertyChanged(nameof(HighHighLimit)); }
        }
        
        private string _shhSetPoint;
        /// <summary>
        /// SHH设定点位
        /// </summary>
        public string SHHSetPoint
        {
            get { return _shhSetPoint; }
            set { SetProperty(ref _shhSetPoint, value); }
        }
        
        private string _shhSetPointPLCAddress;
        /// <summary>
        /// SHH设定点位_PLC地址
        /// </summary>
        public string SHHSetPointPLCAddress
        {
            get { return _shhSetPointPLCAddress; }
            set { SetProperty(ref _shhSetPointPLCAddress, value); }
        }
        
        private string _shhSetPointCommAddress;
        /// <summary>
        /// SHH设定点位_通讯地址
        /// </summary>
        public string SHHSetPointCommAddress
        {
            get { return _shhSetPointCommAddress; }
            set { SetProperty(ref _shhSetPointCommAddress, value); }
        }
        
        private string _llAlarm;
        /// <summary>
        /// LL报警
        /// </summary>
        public string LLAlarm
        {
            get { return _llAlarm; }
            set { SetProperty(ref _llAlarm, value); }
        }
        
        private string _llAlarmPLCAddress;
        /// <summary>
        /// LL报警_PLC地址
        /// </summary>
        public string LLAlarmPLCAddress
        {
            get { return _llAlarmPLCAddress; }
            set { SetProperty(ref _llAlarmPLCAddress, value); }
        }
        
        private string _llAlarmCommAddress;
        /// <summary>
        /// LL报警_通讯地址
        /// </summary>
        public string LLAlarmCommAddress
        {
            get { return _llAlarmCommAddress; }
            set { SetProperty(ref _llAlarmCommAddress, value); }
        }
        
        private string _lAlarm;
        /// <summary>
        /// L报警
        /// </summary>
        public string LAlarm
        {
            get { return _lAlarm; }
            set { SetProperty(ref _lAlarm, value); }
        }
        
        private string _lAlarmPLCAddress;
        /// <summary>
        /// L报警_PLC地址
        /// </summary>
        public string LAlarmPLCAddress
        {
            get { return _lAlarmPLCAddress; }
            set { SetProperty(ref _lAlarmPLCAddress, value); }
        }
        
        private string _lAlarmCommAddress;
        /// <summary>
        /// L报警_通讯地址
        /// </summary>
        public string LAlarmCommAddress
        {
            get { return _lAlarmCommAddress; }
            set { SetProperty(ref _lAlarmCommAddress, value); }
        }
        
        private string _hAlarm;
        /// <summary>
        /// H报警
        /// </summary>
        public string HAlarm
        {
            get { return _hAlarm; }
            set { SetProperty(ref _hAlarm, value); }
        }
        
        private string _hAlarmPLCAddress;
        /// <summary>
        /// H报警_PLC地址
        /// </summary>
        public string HAlarmPLCAddress
        {
            get { return _hAlarmPLCAddress; }
            set { SetProperty(ref _hAlarmPLCAddress, value); }
        }
        
        private string _hAlarmCommAddress;
        /// <summary>
        /// H报警_通讯地址
        /// </summary>
        public string HAlarmCommAddress
        {
            get { return _hAlarmCommAddress; }
            set { SetProperty(ref _hAlarmCommAddress, value); }
        }
        
        private string _hhAlarm;
        /// <summary>
        /// HH报警
        /// </summary>
        public string HHAlarm
        {
            get { return _hhAlarm; }
            set { SetProperty(ref _hhAlarm, value); }
        }
        
        private string _hhAlarmPLCAddress;
        /// <summary>
        /// HH报警_PLC地址
        /// </summary>
        public string HHAlarmPLCAddress
        {
            get { return _hhAlarmPLCAddress; }
            set { SetProperty(ref _hhAlarmPLCAddress, value); }
        }
        
        private string _hhAlarmCommAddress;
        /// <summary>
        /// HH报警_通讯地址
        /// </summary>
        public string HHAlarmCommAddress
        {
            get { return _hhAlarmCommAddress; }
            set { SetProperty(ref _hhAlarmCommAddress, value); }
        }
        
        private string _maintenanceValueSetting;
        /// <summary>
        /// 维护值设定
        /// </summary>
        public string MaintenanceValueSetting
        {
            get { return _maintenanceValueSetting; }
            set { SetProperty(ref _maintenanceValueSetting, value); }
        }
        
        private string _maintenanceValueSetPoint;
        /// <summary>
        /// 维护值设定点位
        /// </summary>
        public string MaintenanceValueSetPoint
        {
            get { return _maintenanceValueSetPoint; }
            set { SetProperty(ref _maintenanceValueSetPoint, value); }
        }
        
        private string _maintenanceValueSetPointPLCAddress;
        /// <summary>
        /// 维护值设定点位_PLC地址
        /// </summary>
        public string MaintenanceValueSetPointPLCAddress
        {
            get { return _maintenanceValueSetPointPLCAddress; }
            set { SetProperty(ref _maintenanceValueSetPointPLCAddress, value); }
        }
        
        private string _maintenanceValueSetPointCommAddress;
        /// <summary>
        /// 维护值设定点位_通讯地址
        /// </summary>
        public string MaintenanceValueSetPointCommAddress
        {
            get { return _maintenanceValueSetPointCommAddress; }
            set { SetProperty(ref _maintenanceValueSetPointCommAddress, value); }
        }
        
        private string _maintenanceEnableSwitchPoint;
        /// <summary>
        /// 维护使能开关点位
        /// </summary>
        public string MaintenanceEnableSwitchPoint
        {
            get { return _maintenanceEnableSwitchPoint; }
            set { SetProperty(ref _maintenanceEnableSwitchPoint, value); }
        }
        
        private string _maintenanceEnableSwitchPointPLCAddress;
        /// <summary>
        /// 维护使能开关点位_PLC地址
        /// </summary>
        public string MaintenanceEnableSwitchPointPLCAddress
        {
            get { return _maintenanceEnableSwitchPointPLCAddress; }
            set { SetProperty(ref _maintenanceEnableSwitchPointPLCAddress, value); }
        }
        
        private string _maintenanceEnableSwitchPointCommAddress;
        /// <summary>
        /// 维护使能开关点位_通讯地址
        /// </summary>
        public string MaintenanceEnableSwitchPointCommAddress
        {
            get { return _maintenanceEnableSwitchPointCommAddress; }
            set { SetProperty(ref _maintenanceEnableSwitchPointCommAddress, value); }
        }
        
        private string _plcAbsoluteAddress;
        /// <summary>
        /// PLC绝对地址
        /// </summary>
        public string PLCAbsoluteAddress
        {
            get { return _plcAbsoluteAddress; }
            set { SetProperty(ref _plcAbsoluteAddress, value); }
        }

        private string _plcCommunicationAddress;
        /// <summary>
        /// PLC绝对地址
        /// </summary>
        public string PlcCommunicationAddress
        {
            get { return _plcCommunicationAddress; }
            set { SetProperty(ref _plcCommunicationAddress, value); }
        }

        private DateTime _createdTime = DateTime.Now;
        /// <summary>
        /// 创建时间
        /// </summary>
        public DateTime CreatedTime
        {
            get { return _createdTime; }
            set { SetProperty(ref _createdTime, value); }
        }
        
        private DateTime? _updatedTime;
        /// <summary>
        /// 更新时间
        /// </summary>
        public DateTime? UpdatedTime
        {
            get { return _updatedTime; }
            set { SetProperty(ref _updatedTime, value); }
        }
        
        #endregion
        
        #region 新增的字段
        
        private string _testBatch;
        /// <summary>
        /// 测试批次
        /// </summary>
        public string TestBatch
        {
            get { return _testBatch; }
            set { SetProperty(ref _testBatch, value); }
        }
        
        private string _testPLCChannelTag;
        /// <summary>
        /// 测试PLC通道位号
        /// </summary>
        public string TestPLCChannelTag
        {
            get { return _testPLCChannelTag; }
            set { SetProperty(ref _testPLCChannelTag, value); }
        }
        
        private string _testPLCCommunicationAddress;
        /// <summary>
        /// 测试PLC通讯地址
        /// </summary>
        public string TestPLCCommunicationAddress
        {
            get { return _testPLCCommunicationAddress; }
            set { SetProperty(ref _testPLCCommunicationAddress, value); }
        }
        
        #endregion
        
        #region 计算属性
        
        /// <summary>
        /// 获取低低限值 (如果未设定则为null)
        /// </summary>
        public float? LowLowLimit => SLLSetValueNumber;

        /// <summary>
        /// 获取低限值 (如果未设定则为null)
        /// </summary>
        public float? LowLimit => SLSetValueNumber;

        /// <summary>
        /// 获取高限值 (如果未设定则为null)
        /// </summary>
        public float? HighLimit => SHSetValueNumber;

        /// <summary>
        /// 获取高高限值 (如果未设定则为null)
        /// </summary>
        public float? HighHighLimit => SHHSetValueNumber;

        #endregion

        #region 上位机检查项

        //TODO:添加上位机检查项相关属性

        private string _trendCheck = "未测试";
        private TestStatus _trendCheckEnum = TestStatus.NotTested;
        public string TrendCheck
        {
            get => _trendCheck;
            set
            {
                if (SetProperty(ref _trendCheck, value))
                {
                    _trendCheckEnum = TestStatusExtensions.Parse(value);
                    RaisePropertyChanged(nameof(TrendCheckEnum));
                }
            }
        }
        public TestStatus TrendCheckEnum
        {
            get => _trendCheckEnum;
            set
            {
                if (SetProperty(ref _trendCheckEnum, value))
                {
                    _trendCheck = value.ToText();
                    RaisePropertyChanged(nameof(TrendCheck));
                }
            }
        }

        private string _reportCheck = "未测试";
        private TestStatus _reportCheckEnum = TestStatus.NotTested;
        public string ReportCheck
        {
            get => _reportCheck;
            set
            {
                if (SetProperty(ref _reportCheck, value))
                {
                    _reportCheckEnum = TestStatusExtensions.Parse(value);
                    RaisePropertyChanged(nameof(ReportCheckEnum));
                }
            }
        }
        public TestStatus ReportCheckEnum
        {
            get => _reportCheckEnum;
            set
            {
                if (SetProperty(ref _reportCheckEnum, value))
                {
                    _reportCheck = value.ToText();
                    RaisePropertyChanged(nameof(ReportCheck));
                }
            }
        }

        #endregion

        public string MonitorStatus { get; set; } = "未检测";

        #region 测试相关字段

        private int _testId;
        /// <summary>
        /// 测试序号
        /// </summary>
        public int TestId
        {
            get { return _testId; }
            set { SetProperty(ref _testId, value); }
        }

        private int _testResultStatus;
        private OverallResultStatus _overallStatus = OverallResultStatus.NotTested;
        /// <summary>
        /// 原始整型测试状态，数据库兼容字段。
        /// </summary>
        public int TestResultStatus
        {
            get { return _testResultStatus; }
            set
            {
                if (SetProperty(ref _testResultStatus, value))
                {
                    _overallStatus = OverallResultStatusExtensions.Parse(value);
                    RaisePropertyChanged(nameof(OverallStatus));
                }
            }
        }

        /// <summary>
        /// 新枚举化的整体测试状态。
        /// </summary>
        public OverallResultStatus OverallStatus
        {
            get => _overallStatus;
            set
            {
                if (SetProperty(ref _overallStatus, value))
                {
                    _testResultStatus = value.ToCode();
                    RaisePropertyChanged(nameof(TestResultStatus));
                }
            }
        }

        private string _resultText;
        /// <summary>
        /// 测试结果信息
        /// </summary>
        public string ResultText
        {
            get { return _resultText; }
            set { SetProperty(ref _resultText, value); }
        }

        private string _hardPointTestResult;
        /// <summary>
        /// 硬点通道测试结果
        /// </summary>
        public string HardPointTestResult
        {
            get { return _hardPointTestResult; }
            set { SetProperty(ref _hardPointTestResult, value); }
        }

        /// <summary>
        /// 使用枚举表示的硬点测试状态（过渡期保留旧字符串属性）。
        /// </summary>
        private Shared.TestStatus _hardPointStatus = Shared.TestStatus.NotTested;
        public Shared.TestStatus HardPointStatus
        {
            get => _hardPointStatus;
            set { SetProperty(ref _hardPointStatus, value); _hardPointTestResult = value.ToText(); }
        }

        private DateTime? _testTime;
        /// <summary>
        /// 测试时间
        /// </summary>
        public DateTime? TestTime
        {
            get { return _testTime; }
            set { SetProperty(ref _testTime, value); }
        }

        private DateTime? _finalTestTime;
        /// <summary>
        /// 最终测试时间
        /// </summary>
        public DateTime? FinalTestTime
        {
            get { return _finalTestTime; }
            set 
            { 
                SetProperty(ref _finalTestTime, value);
                RaisePropertyChanged(nameof(TotalTestDuration));
            }
        }

        private string _status;
        /// <summary>
        /// 当前测试状态（通过/失败/取消等）
        /// </summary>
        public string Status
        {
            get { return _status; }
            set { SetProperty(ref _status, value); }
        }

        private DateTime? _startTime;
        /// <summary>
        /// 测试开始时间
        /// </summary>
        public DateTime? StartTime
        {
            get { return _startTime; }
            set { SetProperty(ref _startTime, value); RaisePropertyChanged(nameof(TestDuration)); RaisePropertyChanged(nameof(TotalTestDuration)); }
        }

        private DateTime? _endTime;
        /// <summary>
        /// 测试结束时间
        /// </summary>
        public DateTime? EndTime
        {
            get { return _endTime; }
            set { SetProperty(ref _endTime, value); RaisePropertyChanged(nameof(TestDuration)); }
        }

        /// <summary>
        /// 测试持续时间（秒）
        /// </summary>
        public double TestDuration
        {
            get
            {
                if (_startTime.HasValue && _endTime.HasValue)
                {
                    TimeSpan duration = _endTime.Value - _startTime.Value;
                    return duration.TotalSeconds;
                }
                return 0;
            }
        }

        /// <summary>
        /// 总测试持续时间（秒）
        /// </summary>
        public double TotalTestDuration
        {
            get
            {
                if (_finalTestTime.HasValue && _startTime.HasValue)
                {
                    TimeSpan duration = _finalTestTime.Value - _startTime.Value;
                    return duration.TotalSeconds;
                }
                return 0;
            }
        }

        // RangeLowerLimitValue和RangeUpperLimitValue已经存在，作为量程低限和高限

        private float? _expectedValue;
        /// <summary>
        /// 期望值 (可为空)
        /// </summary>
        public float? ExpectedValue
        {
            get { return _expectedValue; }
            set { SetProperty(ref _expectedValue, value); RaisePropertyChanged(nameof(Deviation)); RaisePropertyChanged(nameof(DeviationPercent)); }
        }

        private float? _actualValue;
        /// <summary>
        /// 实际值 (可为空)
        /// </summary>
        public float? ActualValue
        {
            get { return _actualValue; }
            set
            {
                SetProperty(ref _actualValue, value);
                RaisePropertyChanged(nameof(Deviation));
                RaisePropertyChanged(nameof(DeviationPercent));
            }
        }

        /// <summary>
        /// 偏差值 (如果期望值或实际值为空，则结果也为空)
        /// </summary>
        public float? Deviation
        {
            get
            {
                if (ActualValue.HasValue && ExpectedValue.HasValue)
                    return ActualValue.Value - ExpectedValue.Value;
                return null;
            }
        }

        /// <summary>
        /// 偏差百分比 (如果期望值、实际值为空或期望值为0，则结果为空)
        /// </summary>
        public float? DeviationPercent
        {
            get
            {
                if (ActualValue.HasValue && ExpectedValue.HasValue && ExpectedValue.Value != 0)
                {
                    return ((ActualValue.Value - ExpectedValue.Value) / ExpectedValue.Value) * 100f;
                }
                return null;
            }
        }

        private float? _value0Percent;
        /// <summary>
        /// 0%对比值 (可为空)
        /// </summary>
        public float? Value0Percent
        {
            get { return _value0Percent; }
            set { SetProperty(ref _value0Percent, value); }
        }

        private float? _value25Percent;
        /// <summary>
        /// 25%对比值 (可为空)
        /// </summary>
        public float? Value25Percent
        {
            get { return _value25Percent; }
            set { SetProperty(ref _value25Percent, value); }
        }

        private float? _value50Percent;
        /// <summary>
        /// 50%对比值 (可为空)
        /// </summary>
        public float? Value50Percent
        {
            get { return _value50Percent; }
            set { SetProperty(ref _value50Percent, value); }
        }

        private float? _value75Percent;
        /// <summary>
        /// 75%对比值 (可为空)
        /// </summary>
        public float? Value75Percent
        {
            get { return _value75Percent; }
            set { SetProperty(ref _value75Percent, value); }
        }

        private float? _value100Percent;
        /// <summary>
        /// 100%对比值 (可为空)
        /// </summary>
        public float? Value100Percent
        {
            get { return _value100Percent; }
            set { SetProperty(ref _value100Percent, value); }
        }

        private string _lowLowAlarmStatus = "未测试"; // 保留原字符串字段
        private TestStatus _lowLowAlarmStatusEnum = TestStatus.NotTested;
        public string LowLowAlarmStatus
        {
            get => _lowLowAlarmStatus;
            set
            {
                if (SetProperty(ref _lowLowAlarmStatus, value))
                {
                    _lowLowAlarmStatusEnum = TestStatusExtensions.Parse(value);
                    RaisePropertyChanged(nameof(LowLowAlarmStatusEnum));
                }
            }
        }
        public TestStatus LowLowAlarmStatusEnum
        {
            get => _lowLowAlarmStatusEnum;
            set
            {
                if (SetProperty(ref _lowLowAlarmStatusEnum, value))
                {
                    _lowLowAlarmStatus = value.ToText();
                    RaisePropertyChanged(nameof(LowLowAlarmStatus));
                }
            }
        }

        // LowAlarm
        private string _lowAlarmStatus = "未测试";
        private TestStatus _lowAlarmStatusEnum = TestStatus.NotTested;
        public string LowAlarmStatus
        {
            get => _lowAlarmStatus;
            set
            {
                if (SetProperty(ref _lowAlarmStatus, value))
                {
                    _lowAlarmStatusEnum = TestStatusExtensions.Parse(value);
                    RaisePropertyChanged(nameof(LowAlarmStatusEnum));
                }
            }
        }
        public TestStatus LowAlarmStatusEnum
        {
            get => _lowAlarmStatusEnum;
            set
            {
                if (SetProperty(ref _lowAlarmStatusEnum, value))
                {
                    _lowAlarmStatus = value.ToText();
                    RaisePropertyChanged(nameof(LowAlarmStatus));
                }
            }
        }

        // HighAlarm
        private string _highAlarmStatus = "未测试";
        private TestStatus _highAlarmStatusEnum = TestStatus.NotTested;
        public string HighAlarmStatus
        {
            get => _highAlarmStatus;
            set
            {
                if (SetProperty(ref _highAlarmStatus, value))
                {
                    _highAlarmStatusEnum = TestStatusExtensions.Parse(value);
                    RaisePropertyChanged(nameof(HighAlarmStatusEnum));
                }
            }
        }
        public TestStatus HighAlarmStatusEnum
        {
            get => _highAlarmStatusEnum;
            set
            {
                if (SetProperty(ref _highAlarmStatusEnum, value))
                {
                    _highAlarmStatus = value.ToText();
                    RaisePropertyChanged(nameof(HighAlarmStatus));
                }
            }
        }

        // HighHighAlarm
        private string _highHighAlarmStatus = "未测试";
        private TestStatus _highHighAlarmStatusEnum = TestStatus.NotTested;
        public string HighHighAlarmStatus
        {
            get => _highHighAlarmStatus;
            set
            {
                if (SetProperty(ref _highHighAlarmStatus, value))
                {
                    _highHighAlarmStatusEnum = TestStatusExtensions.Parse(value);
                    RaisePropertyChanged(nameof(HighHighAlarmStatusEnum));
                }
            }
        }
        public TestStatus HighHighAlarmStatusEnum
        {
            get => _highHighAlarmStatusEnum;
            set
            {
                if (SetProperty(ref _highHighAlarmStatusEnum, value))
                {
                    _highHighAlarmStatus = value.ToText();
                    RaisePropertyChanged(nameof(HighHighAlarmStatus));
                }
            }
        }

        private string _maintenanceFunction = "未测试";
        private TestStatus _maintenanceFunctionEnum = TestStatus.NotTested;
        public string MaintenanceFunction
        {
            get => _maintenanceFunction;
            set
            {
                if (SetProperty(ref _maintenanceFunction, value))
                {
                    _maintenanceFunctionEnum = TestStatusExtensions.Parse(value);
                    RaisePropertyChanged(nameof(MaintenanceFunctionEnum));
                }
            }
        }
        public TestStatus MaintenanceFunctionEnum
        {
            get => _maintenanceFunctionEnum;
            set
            {
                if (SetProperty(ref _maintenanceFunctionEnum, value))
                {
                    _maintenanceFunction = value.ToText();
                    RaisePropertyChanged(nameof(MaintenanceFunction));
                }
            }
        }

        private string _errorMessage;
        /// <summary>
        /// 错误信息
        /// </summary>
        public string ErrorMessage
        {
            get { return _errorMessage; }
            set { SetProperty(ref _errorMessage, value); }
        }

        #endregion

        public string CurrentValue { get; set; } = "--";

        private string _showValueStatus = "未测试"; // 旧字符串字段，保持序列化兼容
        private TestStatus _showValueStatusEnum = TestStatus.NotTested; // 新枚举字段

        /// <summary>
        /// 显示值核对状态 (字符串，兼容旧绑定)
        /// </summary>
        public string ShowValueStatus
        {
            get { return _showValueStatus; }
            set
            {
                if (SetProperty(ref _showValueStatus, value))
                {
                    _showValueStatusEnum = value switch
                    {
                        "通过" => TestStatus.Passed,
                        "失败" => TestStatus.Failed,
                        "跳过" => TestStatus.Skipped,
                        "等待测试" => TestStatus.Waiting,
                        "测试中" => TestStatus.Testing,
                        "N/A" => TestStatus.NotApplicable,
                        _ => TestStatus.NotTested
                    };
                    RaisePropertyChanged(nameof(ShowValueStatusEnum));
                }
            }
        }

        /// <summary>
        /// 显示值核对状态 (枚举，新逻辑使用)
        /// </summary>
        public TestStatus ShowValueStatusEnum
        {
            get => _showValueStatusEnum;
            set
            {
                if (SetProperty(ref _showValueStatusEnum, value))
                {
                    _showValueStatus = value.ToText();
                    RaisePropertyChanged(nameof(ShowValueStatus));
                }
            }
        }

        private string _alarmValueSetStatus = "未测试";
        private TestStatus _alarmValueSetStatusEnum = TestStatus.NotTested;
        public string AlarmValueSetStatus
        {
            get => _alarmValueSetStatus;
            set
            {
                if (SetProperty(ref _alarmValueSetStatus, value))
                {
                    _alarmValueSetStatusEnum = TestStatusExtensions.Parse(value);
                    RaisePropertyChanged(nameof(AlarmValueSetStatusEnum));
                }
            }
        }
        public TestStatus AlarmValueSetStatusEnum
        {
            get => _alarmValueSetStatusEnum;
            set
            {
                if (SetProperty(ref _alarmValueSetStatusEnum, value))
                {
                    _alarmValueSetStatus = value.ToText();
                    RaisePropertyChanged(nameof(AlarmValueSetStatus));
                }
            }
        }

        private string _hardPointErrorDetail;
        /// <summary>
        /// 硬点测试详细错误信息
        /// </summary>
        public string HardPointErrorDetail
        {
            get { return _hardPointErrorDetail; }
            set { SetProperty(ref _hardPointErrorDetail, value); }
        }
    }
}