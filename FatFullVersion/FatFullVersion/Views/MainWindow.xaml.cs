using System.Windows;
using System.Windows.Input;

namespace FatFullVersion.Views
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        public MainWindow()
        {
            InitializeComponent();

            #region 窗口最大化最小化以及拖动逻辑
            //最小化
            BtnMin.Click += (s, e) =>
            {
                this.WindowState = WindowState.Minimized;
            };
            //最大化
            BtnMax.Click += (s, e) =>
            {
                if (this.WindowState == WindowState.Maximized)
                {
                    this.WindowState = WindowState.Normal;
                }
                else
                {
                    this.WindowState = WindowState.Maximized;
                }
            };
            //关闭
            BtnClose.Click += (s, e) =>
            {
                this.Close();
            };
            //拖动
            ColorZone.MouseMove += (s, e) =>
            {
                if (e.LeftButton == MouseButtonState.Pressed)
                {
                    this.DragMove();
                }
            };
            //双击缩放
            ColorZone.MouseDoubleClick += (s, e) =>
            {
                if (this.WindowState == WindowState.Maximized)
                {
                    this.WindowState = WindowState.Normal;
                }
                else if (this.WindowState == WindowState.Normal)
                {
                    this.WindowState = WindowState.Maximized;
                }
            };
            #endregion
        }
    }
}
