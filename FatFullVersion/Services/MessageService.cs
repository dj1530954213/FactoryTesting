using FatFullVersion.IServices;
using System.Threading.Tasks;
using System.Windows;

namespace FatFullVersion.Services
{
    /// <summary>
    /// 消息服务实现类，提供消息显示相关功能
    /// </summary>
    public class MessageService : IMessageService
    {
        /// <summary>
        /// 获取消息
        /// </summary>
        /// <returns>消息内容</returns>
        public string GetMessage()
        {
            return "Hello from the Message Service";
        }

        /// <summary>
        /// 显示消息对话框
        /// </summary>
        /// <param name="title">标题</param>
        /// <param name="message">消息内容</param>
        /// <param name="button">按钮选项</param>
        /// <returns>用户选择的结果</returns>
        public async Task<MessageBoxResult> ShowAsync(string title, string message, MessageBoxButton button)
        {
            return await Application.Current.Dispatcher.InvokeAsync(() =>
            {
                return MessageBox.Show(message, title, button);
            });
        }
    }
} 