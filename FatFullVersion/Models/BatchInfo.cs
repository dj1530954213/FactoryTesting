using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace FatFullVersion.Models
{
    /// <summary>
    /// 批次信息类，用于存储测试批次的相关信息
    /// </summary>
    public class BatchInfo
    {
        /// <summary>
        /// 批次ID
        /// </summary>
        public string BatchId { get; set; }

        /// <summary>
        /// 批次名称
        /// </summary>
        public string BatchName { get; set; }

        /// <summary>
        /// 测试项点数
        /// </summary>
        public int ItemCount { get; set; }

        /// <summary>
        /// 创建时间
        /// </summary>
        public DateTime CreationDate { get; set; }

        /// <summary>
        /// 初次测试时间
        /// </summary>
        public DateTime? FirstTestTime { get; set; }

        /// <summary>
        /// 最终测试时间
        /// </summary>
        public DateTime? LastTestTime { get; set; }

        /// <summary>
        /// 测试状态：已完成、取消、进行中、未开始
        /// </summary>
        public string Status { get; set; }

        /// <summary>
        /// 创建一个新的批次信息实例
        /// </summary>
        public BatchInfo()
        {
            BatchId = Guid.NewGuid().ToString("N");
            CreationDate = DateTime.Now;
            Status = "未开始";
        }

        /// <summary>
        /// 使用给定的批次名称创建一个新的批次信息实例
        /// </summary>
        /// <param name="batchName">批次名称</param>
        /// <param name="itemCount">测试项点数</param>
        public BatchInfo(string batchName, int itemCount)
            : this()
        {
            BatchName = batchName;
            ItemCount = itemCount;
        }
    }
} 