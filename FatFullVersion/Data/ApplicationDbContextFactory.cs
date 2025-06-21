using System;
using System.IO;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Design;

namespace FatFullVersion.Data
{
    /// <summary>
    /// 设计时数据库上下文工厂类
    /// 用于EF Core工具（如迁移命令）创建数据库上下文实例
    /// </summary>
    public class ApplicationDbContextFactory : IDesignTimeDbContextFactory<ApplicationDbContext>
    {
        /// <summary>
        /// 创建数据库上下文实例
        /// </summary>
        /// <param name="args">命令行参数</param>
        /// <returns>数据库上下文实例</returns>
        public ApplicationDbContext CreateDbContext(string[] args)
        {
            try
            {
                // 获取应用程序根目录
                string appRootPath = AppDomain.CurrentDomain.BaseDirectory;
                
                // 确保Data目录存在
                string dataPath = Path.Combine(appRootPath, "Data");
                Directory.CreateDirectory(dataPath);
                
                // 数据库文件路径
                string dbPath = Path.Combine(dataPath, "fattest.db");

                // 创建数据库选项构建器
                var optionsBuilder = new DbContextOptionsBuilder<ApplicationDbContext>();
                
                // 配置SQLite数据库连接
                optionsBuilder.UseSqlite($"Data Source={dbPath}");

                // 返回新的数据库上下文实例
                return new ApplicationDbContext(optionsBuilder.Options);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"创建数据库上下文失败: {ex.Message}");
                Console.WriteLine($"堆栈跟踪: {ex.StackTrace}");
                throw;
            }
        }
    }
} 