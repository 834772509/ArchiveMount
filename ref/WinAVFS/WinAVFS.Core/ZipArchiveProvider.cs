using System;
using System.IO;
using System.IO.Compression;
using System.Text;

namespace WinAVFS.Core
{
    public class ZipArchiveProvider : IArchiveProvider
    {
        private readonly ZipArchive archive;

        public ZipArchiveProvider(string path)
        {
            Console.WriteLine($"Opening archive {path}");
            this.archive = new ZipArchive(new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read),
                ZipArchiveMode.Read, false, Encoding.Default);
        }

        public void Dispose()
        {
            this.archive?.Dispose();
        }

        public FSTree ReadFSTree()
        {
            var root = new FSTreeNode(true);
            foreach (var entry in archive.Entries)
            {
                Console.WriteLine($"Loading {entry.FullName} into FS tree");
                var paths = entry.FullName.Split('/', '\\');
                var node = root;
                for (var i = 0; i < paths.Length - 1; i++)
                {
                    node = node.GetOrAddChild(true, paths[i]);
                }

                var name = paths[paths.Length - 1];
                if (!string.IsNullOrEmpty(name))
                {
                    node = node.GetOrAddChild(false, name, entry.Length, entry.CompressedLength, entry);
                    node.LastWriteTime = entry.LastWriteTime.DateTime;
                }

                node.Context = entry;
            }

            Console.WriteLine($"Loaded {archive.Entries.Count} entries from archive");
            return new FSTree {Root = root};
        }

        public void ExtractFileUnmanaged(FSTreeNode node, IntPtr buffer)
        {
            if (!(node.Context is ZipArchiveEntry entry))
            {
                throw new ArgumentException();
            }

            unsafe
            {
                using var source = entry.Open();
                using var target = new UnmanagedMemoryStream((byte*) buffer.ToPointer(), node.Length, node.Length,
                    FileAccess.Write);
                source.CopyTo(target);
            }
        }
    }
}