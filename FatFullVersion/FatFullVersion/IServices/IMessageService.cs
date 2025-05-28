using System.Threading.Tasks;
using System.Windows;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// 消息服务接口，定义显示消息和对话框的功能
    /// </summary>
    public interface IMessageService
    {
        /// <summary>
        /// 获取消息
        /// </summary>
        /// <returns>消息内容</returns>
        string GetMessage();

        /// <summary>
        /// 显示消息对话框
        /// </summary>
        /// <param name="title">标题</param>
        /// <param name="message">消息内容</param>
        /// <param name="button">按钮选项</param>
        /// <returns>用户选择的结果</returns>
        Task<MessageBoxResult> ShowAsync(string title, string message, MessageBoxButton button);
    }
} 