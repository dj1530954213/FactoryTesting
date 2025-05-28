using System;
using System.IO;
using FatFullVersion.Entities;
using FatFullVersion.Entities.ValueObject;
using FatFullVersion.Models;
using Microsoft.EntityFrameworkCore;

namespace FatFullVersion.Data
{
    /// <summary>
    /// 应用程序数据库上下文类
    /// 负责管理数据库连接和实体映射
    /// </summary>
    public class ApplicationDbContext : DbContext
    {
        public DbSet<PlcConnectionConfig> PlcConnections { get; set; }
        public DbSet<ComparisonTable> ComparisonTables { get; set; }
        public DbSet<Models.ChannelMapping> ChannelMappings { get; set; }

        public ApplicationDbContext(DbContextOptions<ApplicationDbContext> options) : base(options)
        {

        }
        
        /// <summary>
        /// 配置实体映射关系
        /// </summary>
        /// <param name="modelBuilder">模型构建器</param>
        protected override void OnModelCreating(ModelBuilder modelBuilder)
        {
            base.OnModelCreating(modelBuilder);
            
            // 配置ChannelMapping实体
            modelBuilder.Entity<Models.ChannelMapping>(entity =>
            {
                // 配置Id属性为主键
                entity.HasKey(e => e.Id);
                
                // 配置TestTag索引，用于快速查找同一测试批次的记录
                entity.HasIndex(e => e.TestTag);
                
                // 配置CreatedTime自动生成
                entity.Property(e => e.CreatedTime)
                      .HasDefaultValueSql("CURRENT_TIMESTAMP");

                // 配置可空的数值字段 - 解决NaN值存储问题
                entity.Property(e => e.RangeLowerLimitValue)
                      .IsRequired(false);  // 允许NULL
                
                entity.Property(e => e.RangeUpperLimitValue)
                      .IsRequired(false);  // 允许NULL
                
                entity.Property(e => e.SLLSetValueNumber)
                      .IsRequired(false);  // 允许NULL
                      
                entity.Property(e => e.SLSetValueNumber)
                      .IsRequired(false);  // 允许NULL
                      
                entity.Property(e => e.SHSetValueNumber)
                      .IsRequired(false);  // 允许NULL
                      
                entity.Property(e => e.SHHSetValueNumber)
                      .IsRequired(false);  // 允许NULL
                
                entity.Property(e => e.ExpectedValue)
                      .IsRequired(false);  // 允许NULL
                      
                entity.Property(e => e.ActualValue)
                      .IsRequired(false);  // 允许NULL
                
                entity.Property(e => e.Value0Percent)
                      .IsRequired(false);  // 允许NULL
                      
                entity.Property(e => e.Value25Percent)
                      .IsRequired(false);  // 允许NULL
                      
                entity.Property(e => e.Value50Percent)
                      .IsRequired(false);  // 允许NULL
                      
                entity.Property(e => e.Value75Percent)
                      .IsRequired(false);  // 允许NULL
                      
                entity.Property(e => e.Value100Percent)
                      .IsRequired(false);  // 允许NULL

                // 配置可空的时间字段
                entity.Property(e => e.StartTime)
                      .IsRequired(false);
                      
                entity.Property(e => e.EndTime)
                      .IsRequired(false);

                // 指定对应的表名
                entity.ToTable("ChannelMappings");
            });
        }
    }
} 
