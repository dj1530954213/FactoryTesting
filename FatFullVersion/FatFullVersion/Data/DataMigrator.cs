using System;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using System.Linq;
using System.Diagnostics;

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

        /// <summary>
        /// 执行数据库迁移
        /// </summary>
        /// <returns>迁移是否成功</returns>
        public static async Task<bool> MigrateAsync(ApplicationDbContext context)
        {
            try
            {
                // 执行挂起的迁移
                await context.Database.MigrateAsync();

                // 执行数据迁移
                await MigrateNanValuesToNullAsync(context);

                return true;
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"数据库迁移失败: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// 将旧的NaN值（-999999999）迁移为NULL
        /// </summary>
        /// <returns>迁移是否成功</returns>
        private static async Task<bool> MigrateNanValuesToNullAsync(ApplicationDbContext context)
        {
            try
            {
                // 检查是否需要迁移
                var needsMigration = await context.ChannelMappings
                    .AnyAsync(c => c.RangeLowerLimitValue == -999999999f ||
                                   c.RangeUpperLimitValue == -999999999f ||
                                   c.SLLSetValueNumber == -999999999f ||
                                   c.SLSetValueNumber == -999999999f ||
                                   c.SHSetValueNumber == -999999999f ||
                                   c.SHHSetValueNumber == -999999999f ||
                                   c.ExpectedValue == -999999999f ||
                                   c.ActualValue == -999999999f ||
                                   c.Value0Percent == -999999999f ||
                                   c.Value25Percent == -999999999f ||
                                   c.Value50Percent == -999999999f ||
                                   c.Value75Percent == -999999999f ||
                                   c.Value100Percent == -999999999f);

                if (!needsMigration)
                {
                    Debug.WriteLine("无需进行NaN值迁移");
                    return true;
                }

                Debug.WriteLine("开始迁移NaN值到NULL...");

                // 使用原生SQL进行批量更新，提高性能
                var updateSqls = new[]
                {
                    "UPDATE ChannelMappings SET RangeLowerLimitValue = NULL WHERE RangeLowerLimitValue = -999999999",
                    "UPDATE ChannelMappings SET RangeUpperLimitValue = NULL WHERE RangeUpperLimitValue = -999999999",
                    "UPDATE ChannelMappings SET SLLSetValueNumber = NULL WHERE SLLSetValueNumber = -999999999",
                    "UPDATE ChannelMappings SET SLSetValueNumber = NULL WHERE SLSetValueNumber = -999999999",
                    "UPDATE ChannelMappings SET SHSetValueNumber = NULL WHERE SHSetValueNumber = -999999999",
                    "UPDATE ChannelMappings SET SHHSetValueNumber = NULL WHERE SHHSetValueNumber = -999999999",
                    "UPDATE ChannelMappings SET ExpectedValue = NULL WHERE ExpectedValue = -999999999",
                    "UPDATE ChannelMappings SET ActualValue = NULL WHERE ActualValue = -999999999",
                    "UPDATE ChannelMappings SET Value0Percent = NULL WHERE Value0Percent = -999999999",
                    "UPDATE ChannelMappings SET Value25Percent = NULL WHERE Value25Percent = -999999999",
                    "UPDATE ChannelMappings SET Value50Percent = NULL WHERE Value50Percent = -999999999",
                    "UPDATE ChannelMappings SET Value75Percent = NULL WHERE Value75Percent = -999999999",
                    "UPDATE ChannelMappings SET Value100Percent = NULL WHERE Value100Percent = -999999999"
                };

                foreach (var sql in updateSqls)
                {
                    var rowsAffected = await context.Database.ExecuteSqlRawAsync(sql);
                    Debug.WriteLine($"执行SQL: {sql}, 影响行数: {rowsAffected}");
                }

                Debug.WriteLine("NaN值迁移完成");
                return true;
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"迁移NaN值时发生错误: {ex.Message}");
                return false;
            }
        }
    }
} 