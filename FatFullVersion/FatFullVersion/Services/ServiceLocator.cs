using System;
using FatFullVersion.IServices;
using Prism.Ioc;

namespace FatFullVersion.Services
{
    /// <summary>
    /// 服务定位器实现类
    /// </summary>
    public class ServiceLocator : IServiceLocator
    {
        private readonly IContainerProvider _containerProvider;

        /// <summary>
        /// 构造函数
        /// </summary>
        /// <param name="containerProvider">容器提供者</param>
        public ServiceLocator(IContainerProvider containerProvider)
        {
            _containerProvider = containerProvider ?? throw new ArgumentNullException(nameof(containerProvider));
        }

        /// <summary>
        /// 解析指定名称的服务
        /// </summary>
        /// <typeparam name="T">服务类型</typeparam>
        /// <param name="name">服务名称</param>
        /// <returns>服务实例</returns>
        public T ResolveNamed<T>(string name)
        {
            return _containerProvider.Resolve<T>(name);
        }
    }
} 