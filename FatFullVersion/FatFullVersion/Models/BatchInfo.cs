using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using FatFullVersion.Shared;

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

    /// <summary>
    /// 测试批次信息类，用于存储历史测试批次的详细信息
    /// </summary>
    public class TestBatchInfo
    {
        /// <summary>
        /// 测试标识
        /// </summary>
        public string TestTag { get; set; }

        /// <summary>
        /// 创建时间
        /// </summary>
        public DateTime CreatedTime { get; set; }

        /// <summary>
        /// 最后更新时间
        /// </summary>
        public DateTime LastUpdatedTime { get; set; }

        /// <summary>
        /// 总测试点数
        /// </summary>
        public int TotalCount { get; set; }

        /// <summary>
        /// 已测试点数
        /// </summary>
        public int TestedCount { get; set; }

        /// <summary>
        /// 通过点数
        /// </summary>
        public int PassedCount { get; set; }

        /// <summary>
        /// 失败点数
        /// </summary>
        public int FailedCount { get; set; }

        /// <summary>
        /// 通过率
        /// </summary>
        public double PassRate => TotalCount > 0 ? (double)PassedCount / TotalCount * 100 : 0;

        /// <summary>
        /// 测试状态描述
        /// </summary>
        public string Status => GetStatusDescription();

        private string GetStatusDescription()
        {
            if (TestedCount == 0) return "未开始";
            if (TestedCount < TotalCount) return "进行中";
            if (FailedCount > 0) return "部分失败";
            return "已完成";
        }
    }
} 