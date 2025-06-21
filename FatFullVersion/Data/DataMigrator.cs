using System;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using System.Linq;

namespace FatFullVersion.Data
{
    /// <summary>
    /// 数据库迁移和初始化辅助类
    /// </summary>
    public static class DataMigrator
    {
        /// <summary>
        /// 确保数据库已创建
        /// </summary>
        /// <param name="context">数据库上下文</param>
        /// <returns>异步任务</returns>
        public static async Task EnsureDatabaseCreatedAsync(ApplicationDbContext context)
        {
            try
            {
                // 确保数据库存在
                await context.Database.EnsureCreatedAsync();
                
                // 检查数据库是否已创建
                if (!await context.Database.CanConnectAsync())
                {
                    throw new Exception("无法连接到数据库");
                }
            }
            catch (Exception ex)
            {
                throw new Exception($"创建数据库失败：{ex.Message}", ex);
            }
        }

        /// <summary>
        /// 迁移数据库到最新版本
        /// </summary>
        /// <param name="context">数据库上下文</param>
        /// <returns>异步任务</returns>
        public static async Task MigrateDatabaseAsync(ApplicationDbContext context)
        {
            try
            {
                // 检查是否有待应用的迁移
                var pendingMigrations = await context.Database.GetPendingMigrationsAsync();
                if (pendingMigrations.Any())
                {
                    // 应用所有挂起的迁移
                    await context.Database.MigrateAsync();
                }
            }
            catch (Exception ex)
            {
                throw new Exception($"迁移数据库失败：{ex.Message}", ex);
            }
        }
    }
} 