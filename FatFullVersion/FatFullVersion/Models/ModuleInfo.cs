using System;
using Prism.Mvvm;

namespace FatFullVersion.Models
{
    /// <summary>
    /// 模块信息模型，用于模块跳过功能
    /// </summary>
    public class ModuleInfo : BindableBase
    {
        private string _moduleName;
        /// <summary>
        /// 模块名称
        /// </summary>
        public string ModuleName
        {
            get { return _moduleName; }
            set { SetProperty(ref _moduleName, value); }
        }

        private int _channelCount;
        /// <summary>
        /// 通道数量
        /// </summary>
        public int ChannelCount
        {
            get { return _channelCount; }
            set { SetProperty(ref _channelCount, value); }
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

        private bool _isSelected;
        /// <summary>
        /// 是否选中
        /// </summary>
        public bool IsSelected
        {
            get { return _isSelected; }
            set { SetProperty(ref _isSelected, value); }
        }
    }
} 