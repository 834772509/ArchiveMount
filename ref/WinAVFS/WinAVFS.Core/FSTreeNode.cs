using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace WinAVFS.Core
{
    public class FSTreeNode
    {
        public FSTreeNode Parent { get; private set; }

        public string Name { get; private set; } = "";

        public string FullName { get; private set; } = "";

        public long Length { get; private set; } = 0;

        public long CompressedLength { get; private set; } = 0;

        public DateTime? CreationTime { get; internal set; }

        public DateTime? LastAccessTime { get; internal set; }

        public DateTime? LastWriteTime { get; internal set; }

        public Dictionary<string, FSTreeNode> Children { get; }

        public bool IsDirectory => this.Children != null;

        public object Context { get; internal set; }

        public IntPtr Buffer { get; internal set; } = IntPtr.Zero;

        private bool extracted = false;

        public FSTreeNode() : this(false)
        {
        }

        public FSTreeNode(bool isDirectory)
        {
            if (isDirectory)
            {
                this.Children = new Dictionary<string, FSTreeNode>();
            }
        }

        public FSTreeNode GetOrAddChild(bool isDirectory, string name, long length = 0, long compressedLength = 0,
            object context = null)
        {
            if (this.Children == null)
            {
                return null;
            }

            var caseInsensitiveName = name.ToLower();
            if (this.Children.ContainsKey(caseInsensitiveName))
            {
                return this.Children[caseInsensitiveName];
            }

            var child = new FSTreeNode(isDirectory)
            {
                Parent = this,
                Name = name,
                FullName = $"{this.FullName}\\{name}",
                Length = length,
                CompressedLength = compressedLength,
                Context = context,
            };
            this.Children[caseInsensitiveName] = child;

            if (!isDirectory)
            {
                var parent = this;
                while (parent != null)
                {
                    parent.Length += length;
                    parent.CompressedLength += compressedLength;
                    parent = parent.Parent;
                }
            }

            return child;
        }

        public void FillBuffer(Action<IntPtr> extractAction)
        {
            if (this.extracted || this.IsDirectory)
            {
                return;
            }

            lock (this)
            {
                if (!this.extracted)
                {
                    if (this.Buffer == IntPtr.Zero)
                    {
                        this.Buffer = Marshal.AllocHGlobal((IntPtr) this.Length);
                    }

                    try
                    {
                        extractAction(this.Buffer);
                        this.extracted = true;
                    }
                    catch (Exception ex)
                    {
                        Console.Error.WriteLine(ex.StackTrace);
                    }
                }
            }
        }
    }
}