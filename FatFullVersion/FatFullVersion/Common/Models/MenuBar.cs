using Prism.Mvvm;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace FatFullVersion.Common.Models
{
    /// <summary>
    /// 系统导航菜单
    /// </summary>
    public class MenuBar:BindableBase
    {
        //菜单图标
        public string Icon { get; set; }
        //菜单标题
        public string Title { get; set; }
        //菜单命名空间
        public string NameSpace { get; set; }
    }
}
