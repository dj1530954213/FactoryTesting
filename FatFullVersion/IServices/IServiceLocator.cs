using System;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// 服务定位器接口，用于解析命名服务
    /// </summary>
    public interface IServiceLocator
    {
        /// <summary>
        /// 解析指定名称的服务
        /// </summary>
        /// <typeparam name="T">服务类型</typeparam>
        /// <param name="name">服务名称</param>
        /// <returns>服务实例</returns>
        T ResolveNamed<T>(string name);
    }
} 