using System;
using Prism.Events;

namespace FatFullVersion.Events
{
    /// <summary>
    /// 测试结果已更新事件，用于通知UI刷新测试结果显示
    /// </summary>
    public class TestResultsUpdatedEvent : PubSubEvent
    {
    }
} 