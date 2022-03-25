using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using DokanNet;
using DokanNet.Logging;
using Microsoft.Win32;
using SevenZip;

namespace Free_Mount
{
    /// <summary>
    /// MainWindow.xaml 的互動邏輯
    /// </summary>
    public partial class MainWindow : Window
    {
        public MainWindow()
        {
            InitializeComponent();
            /*log.Clear();
            SevenZipExtractor extractor = new SevenZipExtractor(@"F:\free mount test\test files.tar");
            foreach (ArchiveFileInfo i in extractor.ArchiveFileData) {
                log.AppendText($"Filename: {i.FileName}\n");
            }*/
            foreach (char c in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".ToCharArray()) {
                if (!System.IO.Directory.Exists($"{c}:\\")) {
                    Drive_Letter.Items.Add(c);
                }
            }
            Drive_Letter.SelectedIndex = Drive_Letter.Items.Count - 1;
        }

        private void Select_archive_file(object sender, RoutedEventArgs e)
        {
            OpenFileDialog dialog = new OpenFileDialog();
            if (dialog.ShowDialog() == true)
                Selected_archive_file.Content = dialog.FileName;
        }

        private void mount_archive(object sender, RoutedEventArgs e)
        {
            (sender as Button).IsEnabled = false;
            string file = Selected_archive_file.Content as string,
                   pwd = Archive_password.Text,
                   drive = $"{Drive_Letter.SelectedItem}:\\";
            Task.Run(() => {
                FileSystem f = new FileSystem(file, pwd, log);
                f.Mount(drive, new NullLogger());
            });
        }
    }
}
