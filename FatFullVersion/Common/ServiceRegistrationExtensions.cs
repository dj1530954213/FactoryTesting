using System;
using Prism.Ioc;
using Prism.DryIoc;

namespace FatFullVersion.Common
{
    /// <summary>
    /// 服务注册扩展方法
    /// </summary>
    public static class ServiceRegistrationExtensions
    {
        /// <summary>
        /// 注册命名服务
        /// </summary>
        /// <typeparam name="T">服务类型</typeparam>
        /// <param name="containerRegistry">容器注册器</param>
        /// <param name="factory">工厂方法</param>
        /// <param name="name">服务名称</param>
        //public static void RegisterNamedSingleton<T>(this IContainerRegistry containerRegistry, Func<T> factory, string name)
        //{
        //    if (containerRegistry is DryIocContainerRegistry dryIocRegistry)
        //    {
        //        dryIocRegistry.GetContainer().RegisterInstance(factory(), serviceKey: name);
        //    }
        //    else
        //    {
        //        containerRegistry.RegisterSingleton(factory, name);
        //    }
        //}
    }
} 