using System;
using System.IO;

namespace WinAVFS.Core
{
    public interface IArchiveProvider : IDisposable
    {
        FSTree ReadFSTree();

        void ExtractFileUnmanaged(FSTreeNode node, IntPtr buffer);
    }
}