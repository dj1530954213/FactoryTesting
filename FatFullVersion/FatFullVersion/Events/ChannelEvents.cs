using Prism.Events;
using System;
using System.Collections.Generic;

namespace FatFullVersion.Events
{
    public class ChannelStatesModifiedEvent : PubSubEvent<List<Guid>> { }
} 