using DokanNet;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.AccessControl;
using System.Text;
using System.Threading.Tasks;
using SevenZip;
using System.Windows.Controls;
using System.Text.RegularExpressions;

namespace Free_Mount {
    class FileSystem : IDokanOperations {
        SevenZipExtractor extractor;
        Dictionary<string, ArchiveFileInfo> file_info_pair = new Dictionary<string, ArchiveFileInfo>();
        Dictionary<string, MemoryStream> extraction_streams = new Dictionary<string, MemoryStream>();
        TextBox log;
        public FileSystem(string archive_file, string pwd, TextBox t) {
            log = t;
            extractor = pwd !="" ? new SevenZipExtractor(archive_file,pwd) : new SevenZipExtractor(archive_file);
            foreach (ArchiveFileInfo info in extractor.ArchiveFileData) {
                file_info_pair.Add(@"\" + info.FileName, info);
            }
        }
        public NtStatus CreateFile(string fileName, DokanNet.FileAccess access, FileShare share, FileMode mode, FileOptions options, FileAttributes attributes, IDokanFileInfo info) {
            if (fileName == @"\" || mode == FileMode.Open)
                return DokanResult.Success;
            return DokanResult.DiskFull;
        }

        public NtStatus FindFiles(string fileName, out IList<FileInformation> files, IDokanFileInfo info) {
            files = new List<FileInformation>();
            //Console.WriteLine("[FindFiles] filename = " + fileName);
            string pattern = "^" +
                             fileName.Replace(@"\", @"\\") +
                             (fileName.EndsWith(@"\") ? "" : @"\\") +
                             @"([^\\]+)$";
            //Console.WriteLine("Pattern is " + pattern);
            foreach (ArchiveFileInfo ainfo in file_info_pair.Values) {
                //Console.WriteLine($"checking \\{ainfo.FileName}");
                Match m = Regex.Match(@"\" + ainfo.FileName, pattern);
                if (m.Success) {
                    //Console.WriteLine($"Matched filename: {m.Groups[1].Value}");
                    files.Add(new FileInformation
                    {
                        FileName = m.Groups[1].Value,
                        Attributes = ainfo.IsDirectory ? FileAttributes.Directory : FileAttributes.ReadOnly,
                        CreationTime = ainfo.CreationTime,
                        LastAccessTime = ainfo.LastAccessTime,
                        LastWriteTime = ainfo.LastWriteTime,
                        Length = ainfo.Size > Int64.MaxValue ? Int64.MaxValue : Convert.ToInt64(ainfo.Size)
                    });
                    //Console.WriteLine($"{m.Groups[1].Value} is under {fileName}");
                }
            }
            return DokanResult.Success;
        }

        public NtStatus Mounted(IDokanFileInfo info) {
            return DokanResult.Success;
        }

        public NtStatus GetVolumeInformation(out string volumeLabel, out FileSystemFeatures features, out string fileSystemName, out uint maximumComponentLength, IDokanFileInfo info) {
            volumeLabel = "Free mount";
            features = FileSystemFeatures.ReadOnlyVolume;
            fileSystemName = string.Empty;
            maximumComponentLength = 256;
            return DokanResult.Success;
        }

        public NtStatus GetFileInformation(string fileName, out FileInformation fileInfo, IDokanFileInfo info) {
            fileInfo = new FileInformation();
            if (fileName == "\\") {
                fileInfo.Attributes = FileAttributes.Directory;
                fileInfo.LastAccessTime = DateTime.Now;
                fileInfo.LastWriteTime = null;
                fileInfo.CreationTime = null;

                return DokanResult.Success;
            } else if (file_info_pair.ContainsKey(fileName)) {
                //Console.WriteLine("[GetFileInformation] File in dict");
                fileInfo.Attributes = file_info_pair[fileName].IsDirectory ? FileAttributes.Directory : FileAttributes.ReadOnly;
                fileInfo.LastAccessTime = file_info_pair[fileName].LastAccessTime;
                fileInfo.LastWriteTime = file_info_pair[fileName].LastWriteTime;
                fileInfo.CreationTime = file_info_pair[fileName].CreationTime;
                fileInfo.Length = file_info_pair[fileName].Size > Int64.MaxValue ? Int64.MaxValue : Convert.ToInt64(file_info_pair[fileName].Size);
                return DokanResult.Success;
            }

            return DokanResult.Error;
        }

        public NtStatus GetDiskFreeSpace(out long freeBytesAvailable, out long totalNumberOfBytes, out long totalNumberOfFreeBytes, IDokanFileInfo info) {
            totalNumberOfFreeBytes = freeBytesAvailable = 1024 * 1024 * 1024;
            totalNumberOfBytes = 0;
            return DokanResult.Success;
        }

        public NtStatus ReadFile(string fileName, byte[] buffer, out int bytesRead, long offset, IDokanFileInfo info) {
            //Console.WriteLine($"reading {fileName}, buffer length {buffer.Length}");
            bytesRead = 0;
            if (file_info_pair.ContainsKey(fileName)) {
                if (!extraction_streams.ContainsKey(fileName)) {
                    extraction_streams[fileName] = new MemoryStream();
                    extractor.ExtractFile(fileName.Substring(1), extraction_streams[fileName]);
                }
                extraction_streams[fileName].Position = offset;
                bytesRead += extraction_streams[fileName].Read(buffer, bytesRead, buffer.Length);
                return DokanResult.Success;
            }
            Console.WriteLine("[ReadFile] error reading " + fileName);
            return DokanResult.Error;
        }

        #region I don't want to touch it
        public void Cleanup(string fileName, IDokanFileInfo info) {
            return;
        }

        public void CloseFile(string fileName, IDokanFileInfo info) {
            return;
        }

        public NtStatus FindFilesWithPattern(string fileName, string searchPattern, out IList<FileInformation> files, IDokanFileInfo info) {
            files = new List<FileInformation>();
            //if (searchPattern == "*")
            //{
            //    if (fileName == "\\")
            //    {
            //        foreach (ArchiveFileInfo ainfo in file_info_pair.Values)
            //        {
            //            if (!ainfo.FileName.Contains("\\"))
            //            {
            //                files.Add(new FileInformation
            //                {
            //                    FileName = ainfo.FileName,
            //                    Attributes = ainfo.IsDirectory ? FileAttributes.Directory : FileAttributes.ReadOnly,
            //                    CreationTime = ainfo.CreationTime,
            //                    LastAccessTime = ainfo.LastAccessTime,
            //                    LastWriteTime = ainfo.LastWriteTime
            //                });
            //            }
            //        }
            //        return DokanResult.Success;
            //    }

            //}
            return DokanResult.NotImplemented;
        }

        public NtStatus DeleteDirectory(string fileName, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus DeleteFile(string fileName, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus FindStreams(string fileName, out IList<FileInformation> streams, IDokanFileInfo info) {
            streams = null;
            return DokanResult.Error;
        }

        public NtStatus FlushFileBuffers(string fileName, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus GetFileSecurity(string fileName, out FileSystemSecurity security, AccessControlSections sections, IDokanFileInfo info) {
            security = null;
            return DokanResult.Error;
        }

        public NtStatus LockFile(string fileName, long offset, long length, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus MoveFile(string oldName, string newName, bool replace, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus SetAllocationSize(string fileName, long length, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus SetEndOfFile(string fileName, long length, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus SetFileAttributes(string fileName, FileAttributes attributes, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus SetFileSecurity(string fileName, FileSystemSecurity security, AccessControlSections sections, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus SetFileTime(string fileName, DateTime? creationTime, DateTime? lastAccessTime, DateTime? lastWriteTime, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus UnlockFile(string fileName, long offset, long length, IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus Unmounted(IDokanFileInfo info) {
            return DokanResult.Error;
        }

        public NtStatus WriteFile(string fileName, byte[] buffer, out int bytesWritten, long offset, IDokanFileInfo info) {
            bytesWritten = 0;
            return DokanResult.Error;
        }

        #endregion
    }
}
