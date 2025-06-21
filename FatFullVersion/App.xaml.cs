using FatFullVersion.Optimizations;
using FatFullVersion.Services;
using FatFullVersion.Services.Interfaces;
using FatFullVersion.Views;
using Prism.Ioc;
using Prism.Modularity;
using System;
using System.Windows;
using FatFullVersion.IServices;
using FatFullVersion.Entities.EntitiesEnum;
using Prism.DryIoc;
using DryIoc;
using FatFullVersion.Common;
using Prism.Events;
using FatFullVersion.Data;
using System.IO;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;

namespace FatFullVersion
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : PrismApplication
    {
        protected override Window CreateShell()
        {
            try
            {
                var window = Container.Resolve<MainWindow>();
                
                // 启用内存优化
                if (window != null)
                {
                    MemoryOptimizations.EnableOptimizations(window);
                }
                
                return window;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"创建主窗口失败：{ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                return null;
            }
        }
        
        //依赖注入点
        protected override void RegisterTypes(IContainerRegistry containerRegistry)
        {
            try
            {
                var container = containerRegistry.GetContainer();

                // 注册数据库上下文
                containerRegistry.RegisterSingleton<ApplicationDbContext>();
                
                // 获取数据库上下文实例
                var dbContext = container.Resolve<ApplicationDbContext>();
                // 注册 EF Core 的 DbContext
                containerRegistry.Register<ApplicationDbContext>(() =>
                {
                    string str =
                        $"Data Source={System.IO.Path.Combine($"{AppContext.BaseDirectory}Data", "fattest.db")}";
                    var options = new DbContextOptionsBuilder<ApplicationDbContext>()
                        .UseSqlite($"Data Source={System.IO.Path.Combine($"{AppContext.BaseDirectory}Data", "fattest.db")}")
                        .Options;

                    return new ApplicationDbContext(options);
                });

                containerRegistry.RegisterSingleton<IMessageService, MessageService>();
                //注册点位表处理服务，选择Excel实现
                containerRegistry.RegisterSingleton<IPointDataService, ExcelPointDataService>();
                
                // 注册通道映射服务
                containerRegistry.RegisterSingleton<IChannelMappingService, ChannelMappingService>();

                // 注册仓储层服务，需要传入数据库上下文
                containerRegistry.RegisterSingleton<IRepository, Repository>();

                // 注册服务定位器
                containerRegistry.RegisterSingleton<IServiceLocator, ServiceLocator>();

                // 初始化数据库数据
                try
                {
                    // 初始化数据
                    var repository = container.Resolve<IRepository>();
                    repository.InitializeDatabaseAsync().GetAwaiter().GetResult();
                }
                catch (Exception ex)
                {
                    MessageBox.Show($"初始化数据库数据失败：{ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                    // 继续运行程序，但后续可能出现数据库相关错误
                }

                try
                {
                    // 注册PLC通信工厂 - 分别为测试PLC和被测PLC创建工厂
                    var testPlcFactory = new PlcCommunicationFactory(container.Resolve<IRepository>(), PlcType.TestPlc);
                    var targetPlcFactory = new PlcCommunicationFactory(container.Resolve<IRepository>(), PlcType.TargetPlc);

                    // 将两个工厂注册到容器中
                    container.RegisterInstance(testPlcFactory, serviceKey: "TestPlcFactory");
                    container.RegisterInstance(targetPlcFactory, serviceKey: "TargetPlcFactory");

                    // 分别创建测试PLC和被测PLC的通信实例
                    var testPlcCommunication = testPlcFactory.CreatePlcCommunication();
                    var targetPlcCommunication = targetPlcFactory.CreatePlcCommunication();

                    // 将两个通信实例注册到容器中
                    container.RegisterInstance<IPlcCommunication>(testPlcCommunication, serviceKey: "TestPlcCommunication");
                    container.RegisterInstance<IPlcCommunication>(targetPlcCommunication, serviceKey: "TargetPlcCommunication");
                    
                    // 为DataEditView注册特定的PLC通信服务
                    container.RegisterDelegate<IPlcCommunication>(
                        factoryDelegate: r => r.Resolve<IPlcCommunication>(serviceKey: "TestPlcCommunication"), 
                        serviceKey: "TestPlc");
                    container.RegisterDelegate<IPlcCommunication>(
                        factoryDelegate: r => r.Resolve<IPlcCommunication>(serviceKey: "TargetPlcCommunication"), 
                        serviceKey: "TargetPlc");
                }
                catch (Exception ex)
                {
                    MessageBox.Show($"初始化PLC通信失败：{ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                    // 继续运行程序，但后续可能出现PLC通信相关错误
                }

                // 注册测试任务管理器
                containerRegistry.RegisterSingleton<ITestTaskManager, TestTaskManager>();

                //注册历史恢复服务
                containerRegistry.RegisterSingleton<ITestRecordService, TestRecordService>();
                
                // 注册视图用于导航
                containerRegistry.RegisterForNavigation<DataEditView>();
                containerRegistry.RegisterForNavigation<ConfigEditView>();
                
                try
                {
                    // 直接在容器中注册 DataEditView 的自定义构造函数
                    container.Register<DataEditView>(made: Parameters.Of
                        .Type<IPointDataService>()
                        .Type<IChannelMappingService>()
                        .Type<IEventAggregator>()
                        .Type<ITestTaskManager>()
                        .Type<IPlcCommunication>(serviceKey: "TestPlc")
                        .Type<IPlcCommunication>(serviceKey: "TargetPlc")
                        .Type<IMessageService>()
                        .Type<IChannelRangerSettingService>());
                }
                catch (Exception ex)
                {
                    MessageBox.Show($"注册DataEditView失败：{ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                    // 继续运行程序，但后续可能出现DataEditView相关错误
                }

                // 注册量程设置服务
                containerRegistry.RegisterSingleton<IChannelRangerSettingService, ChannelRangerSettingService>();

                // 注册测试结果导出服务
                containerRegistry.RegisterSingleton<ITestResultExportService, TestResultExportService>();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"注册服务失败：{ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                throw; // 如果注册服务失败，则应用无法正常运行，需要抛出异常终止
            }
        }


        //模块注册点
        protected override void ConfigureModuleCatalog(IModuleCatalog moduleCatalog)
        {
        }

        /// <summary>
        /// 应用程序启动事件
        /// </summary>
        /// <param name="e">启动事件参数</param>
        protected override void OnStartup(StartupEventArgs e)
        {
            try
            {
                base.OnStartup(e);
            }
            catch (Exception ex)
            {
                MessageBox.Show($"应用程序启动失败：{ex.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
                Current.Shutdown(-1);
            }
        }

        /// <summary>
        /// 应用程序异常处理
        /// </summary>
        /// <param name="sender">发送者</param>
        /// <param name="e">异常事件参数</param>
        private void Application_DispatcherUnhandledException(object sender, System.Windows.Threading.DispatcherUnhandledExceptionEventArgs e)
        {
            MessageBox.Show($"发生未处理的异常：{e.Exception.Message}", "错误", MessageBoxButton.OK, MessageBoxImage.Error);
            e.Handled = true; // 设置为已处理，防止应用程序崩溃
        }
    }
}
