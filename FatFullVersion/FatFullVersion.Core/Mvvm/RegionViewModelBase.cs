using System;
using Prism.Navigation.Regions;

namespace FatFullVersion.Core.Mvvm
{
    //INavigationAware是用来接受导航时传递的数据的
    public class RegionViewModelBase : ViewModelBase, INavigationAware, IConfirmNavigationRequest
    {
        protected IRegionManager RegionManager { get; private set; }

        public RegionViewModelBase(IRegionManager regionManager)
        {
            RegionManager = regionManager;
        }
        //确认是否允许导航
        public virtual void ConfirmNavigationRequest(NavigationContext navigationContext, Action<bool> continuationCallback)
        {
            continuationCallback(true);
        }
        //每次导航时该实例是否重用原来的实例
        public virtual bool IsNavigationTarget(NavigationContext navigationContext)
        {
            return true;
        }
        //当从现在的界面切换至其他页面的时候触发
        public virtual void OnNavigatedFrom(NavigationContext navigationContext)
        {

        }

        public virtual void OnNavigatedTo(NavigationContext navigationContext)
        {

        }
    }
}
