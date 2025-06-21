using DryIoc;
using Prism.DryIoc;
using Prism.Ioc;

namespace FatFullVersion.Common
{
    /// <summary>
    /// 容器扩展方法
    /// </summary>
    public static class ContainerExtensions
    {
        /// <summary>
        /// 获取DryIoc容器实例
        /// </summary>
        /// <param name="containerRegistry">容器注册器</param>
        /// <returns>DryIoc容器实例</returns>
        public static IContainer GetContainer(this IContainerRegistry containerRegistry)
        {
            return ((IContainerExtension<IContainer>)containerRegistry).Instance;
        }
    }
} 