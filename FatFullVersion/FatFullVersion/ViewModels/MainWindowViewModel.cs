using System;
using System.Collections.ObjectModel;
using System.Windows;
using FatFullVersion.Common.Models;
using FatFullVersion.Core;
using FatFullVersion.Views;
using Prism.Commands;
using Prism.Mvvm;
using Prism.Navigation;
using Prism.Navigation.Regions;

namespace FatFullVersion.ViewModels
{
    public class MainWindowViewModel : BindableBase
    {
        private readonly IRegionManager _regionManager;
        private IRegionNavigationJournal _navigationJournal;

        public MainWindowViewModel(IRegionManager regionManager)
        {
            _regionManager = regionManager;
            NavigateCommand = new DelegateCommand<MenuBar>(Navigate);
            MenuBars = new ObservableCollection<MenuBar>();

            CreateMenu();
            RegisterNavigateCommand();
            
            // 应用启动时自动注册DataEditView
            _regionManager.RegisterViewWithRegion(RegionNames.ContentRegion, typeof(DataEditView));
            
            // 创建一个MenuBar对象并导航到DataEditView
            //var initialMenuBar = new MenuBar { NameSpace = "DataEditView", Title = "综合表格编辑" };
            //Navigate(initialMenuBar);
        }


        #region 界面中相关绑定属性以及绑定命令的申明
        //导航菜单数据
        private ObservableCollection<MenuBar> _menuBars;
        public ObservableCollection<MenuBar> MenuBars
        {
            get => _menuBars;
            set
            {
                _menuBars = value;
                RaisePropertyChanged();
            }
        }
        //导航命令
        public DelegateCommand<MenuBar> NavigateCommand { get; set; }
        public DelegateCommand GoBackCommand { get; set; }
        public DelegateCommand GoForWardCommand { get; set; }
        #endregion

        //导航处理方法,通过界面传回的Item的数据来导航
        public void Navigate(MenuBar menuBar)
        {
            try
            {
                if (menuBar == null || string.IsNullOrWhiteSpace(menuBar.NameSpace))
                    return;
                //导航到页面的同时记录到导航日志中，用于后面的返回
                _regionManager.RequestNavigate(RegionNames.ContentRegion, menuBar.NameSpace, back =>
                {
                    //实例化导航日志
                    if (back.Context != null) _navigationJournal = back.Context.NavigationService.Journal;
                });
            }
            catch (Exception)
            {
                // 忽略导航异常
            }
        }
        //导航日志中前进和返回的命令注册
        public void RegisterNavigateCommand()
        {
            //返回命令的委托注册
            GoBackCommand = new DelegateCommand(() =>
            {
                if (_navigationJournal is { CanGoBack: true })
                {
                    _navigationJournal.GoBack();
                }
            });
            //前进命令的委托注册
            GoForWardCommand = new DelegateCommand(() =>
            {
                if (_navigationJournal is { CanGoForward: true })
                {
                    _navigationJournal.GoForward();
                }
            });
        }

        public void CreateMenu()
        {
            MenuBars.Add(new MenuBar() { Icon = "ApplicationEditOutline", NameSpace = "DataEditView", Title = "综合表格编辑" });
            MenuBars.Add(new MenuBar() { Icon = "SettingsApplications", NameSpace = "ConfigEditView", Title = "PLC配置管理" });
            //MenuBars.Add(new MenuBar() { Icon = "SearchWeb", NameSpace = "DataBrowseView", Title = "线上数据查看" });
        }
    }
}
