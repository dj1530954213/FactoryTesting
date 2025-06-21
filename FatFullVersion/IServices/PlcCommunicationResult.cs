using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace FatFullVersion.IServices
{
    /// <summary>
    /// PLC通信操作结果类，统一处理PLC通信的结果和异常
    /// </summary>
    /// <typeparam name="T">返回数据的类型</typeparam>
    public class PlcCommunicationResult<T>
    {
        /// <summary>
        /// 操作是否成功
        /// </summary>
        public bool IsSuccess { get; set; }

        /// <summary>
        /// 返回的数据
        /// </summary>
        public T Data { get; set; }

        /// <summary>
        /// 错误代码
        /// </summary>
        public int ErrorCode { get; set; }

        /// <summary>
        /// 错误消息
        /// </summary>
        public string ErrorMessage { get; set; }

        /// <summary>
        /// 详细的异常信息
        /// </summary>
        public Exception Exception { get; set; }

        /// <summary>
        /// 创建成功的结果
        /// </summary>
        /// <param name="data">返回数据</param>
        /// <returns>成功的通信结果</returns>
        public static PlcCommunicationResult<T> CreateSuccessResult(T data)
        {
            return new PlcCommunicationResult<T>
            {
                IsSuccess = true,
                Data = data,
                ErrorCode = 0,
                ErrorMessage = string.Empty
            };
        }

        /// <summary>
        /// 创建失败的结果
        /// </summary>
        /// <param name="errorMessage">错误消息</param>
        /// <param name="errorCode">错误代码</param>
        /// <returns>失败的通信结果</returns>
        public static PlcCommunicationResult<T> CreateFailedResult(string errorMessage, int errorCode = -1)
        {
            return new PlcCommunicationResult<T>
            {
                IsSuccess = false,
                ErrorCode = errorCode,
                ErrorMessage = errorMessage
            };
        }

        /// <summary>
        /// 从异常创建失败的结果
        /// </summary>
        /// <param name="exception">异常</param>
        /// <param name="errorCode">错误代码</param>
        /// <returns>失败的通信结果</returns>
        public static PlcCommunicationResult<T> CreateExceptionResult(Exception exception, int errorCode = -1)
        {
            return new PlcCommunicationResult<T>
            {
                IsSuccess = false,
                ErrorCode = errorCode,
                ErrorMessage = exception.Message,
                Exception = exception
            };
        }
    }

    /// <summary>
    /// 无返回数据的PLC通信操作结果类
    /// </summary>
    public class PlcCommunicationResult : PlcCommunicationResult<object>
    {
        /// <summary>
        /// 创建成功的结果
        /// </summary>
        /// <returns>成功的通信结果</returns>
        public static PlcCommunicationResult CreateSuccessResult()
        {
            return new PlcCommunicationResult
            {
                IsSuccess = true,
                ErrorCode = 0,
                ErrorMessage = string.Empty
            };
        }

        /// <summary>
        /// 创建失败的结果
        /// </summary>
        /// <param name="errorMessage">错误消息</param>
        /// <param name="errorCode">错误代码</param>
        /// <returns>失败的通信结果</returns>
        public static new PlcCommunicationResult CreateFailedResult(string errorMessage, int errorCode = -1)
        {
            return new PlcCommunicationResult
            {
                IsSuccess = false,
                ErrorCode = errorCode,
                ErrorMessage = errorMessage
            };
        }

        /// <summary>
        /// 从异常创建失败的结果
        /// </summary>
        /// <param name="exception">异常</param>
        /// <param name="errorCode">错误代码</param>
        /// <returns>失败的通信结果</returns>
        public static new PlcCommunicationResult CreateExceptionResult(Exception exception, int errorCode = -1)
        {
            return new PlcCommunicationResult
            {
                IsSuccess = false,
                ErrorCode = errorCode,
                ErrorMessage = exception.Message,
                Exception = exception
            };
        }
    }
} 