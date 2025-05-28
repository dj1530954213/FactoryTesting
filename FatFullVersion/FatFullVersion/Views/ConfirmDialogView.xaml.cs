using System.Windows;

namespace FatFullVersion.Views
{
    public partial class ConfirmDialogView : Window
    {
        public string MessageText { get; private set; }
        public string CaptionText { get; private set; }

        public ConfirmDialogView(string message, string caption)
        {
            InitializeComponent();
            MessageText = message;
            CaptionText = caption;
            this.DataContext = this;
        }

        private void YesButton_Click(object sender, RoutedEventArgs e)
        {
            DialogResult = true;
            Close();
        }

        private void NoButton_Click(object sender, RoutedEventArgs e)
        {
            DialogResult = false;
            Close();
        }
    }
} 