use std::fs;
use std::fs::OpenOptions;
use std::os::windows::fs::FileExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dokan::{CreateFileInfo, DiskSpaceInfo, DOKAN_IO_SECURITY_CONTEXT, Drive, FileInfo, FileSystemHandler, FillDataError, FindData, MountError, MountFlags, OperationError, OperationInfo, VolumeInfo};
use lru::LruCache;
use widestring::{U16CStr, U16CString};
use winapi::shared::ntstatus::{STATUS_ACCESS_DENIED, STATUS_INVALID_DEVICE_REQUEST, STATUS_NDIS_FILE_NOT_FOUND, STATUS_OBJECT_NAME_NOT_FOUND};
use winapi::um::winnt::{FILE_CASE_PRESERVED_NAMES, FILE_PERSISTENT_ACLS, FILE_READ_ONLY_VOLUME, FILE_UNICODE_ON_DISK, FILE_VOLUME_IS_COMPRESSED};

use crate::sevenZip::{ArchiveFileInfo, sevenZip};
use crate::utils::console::{ConsoleType, writeConsole};
use crate::utils::util::{removeVirtualDrive, StringToSystemTime};

const FILE_ATTRIBUTES_ARCHIVE: u32 = 32;
const FILE_ATTRIBUTES_DIRECTORY: u32 = 16;
const FILE_ATTRIBUTES_LOCALLYINCOMPLETE: u32 = 512;
const FILE_ATTRIBUTES_NORMAL: u32 = 0;
const FILE_ATTRIBUTES_READONLY: u32 = 1;
const FILE_ATTRIBUTES_TEMPORARY: u32 = 256;

/// 替换文件，文件不存在时创建文件
const FILE_SUPERSEDE: u32 = 0;
/// 打开文件，文件不存在时返回错误
const FILE_OPEN: u32 = 1;
/// 创建文件，文件存在时返回错误
const FILE_CREATE: u32 = 2;
/// 打开文件，文件不存在时创建文件
const FILE_OPEN_IF: u32 = 3;
/// 打开文件并覆盖，文件不存在时返回错误
const FILE_OVERWRITE: u32 = 4;
/// 打开文件并覆盖，文件不存在时创建文件
const FILE_OVERWRITE_IF: u32 = 5;

#[derive(Debug)]
pub struct ArchiveFS {
    /// SevenZIP程序类
    sevenZip: sevenZip,
    /// 压缩包路径
    archivePath: PathBuf,
    /// 压缩包密码
    password: Option<String>,
    /// 临时释放路径
    extractPath: PathBuf,
    /// 缓存大小(单位: MB)
    cacheSize: u64,
    /// 是否只读挂载
    readOnly: bool,
    /// 压缩包文件信息
    archiveFileInfoList: Vec<ArchiveFileInfo>,
    /// 缓存信息
    cacheInfoList: Mutex<LruCache<ArchiveFileInfo, PathBuf>>,
    /// 挂载后是否打开
    open: bool,
    /// 挂载卷标名
    volumeName: String,
    /// 是否为调试模式
    isDebugMode: bool,
}

impl ArchiveFS {
    pub(crate) fn new(archivePath: &Path, extractPath: &Path, password: Option<&str>, cacheSize: u64, readOnly: bool, archiveFileInfoList: Vec<ArchiveFileInfo>, open: bool, volumeName: &str, isDebugMode: bool) -> ArchiveFS {
        fs::create_dir_all(extractPath).ok();
        ArchiveFS {
            sevenZip: sevenZip::new().unwrap(),
            archivePath: (*archivePath.to_path_buf()).to_owned(),
            password: password.map(|password| password.to_string()),
            extractPath: (*extractPath.to_path_buf()).to_owned(),
            cacheSize,
            readOnly,
            archiveFileInfoList,
            cacheInfoList: Mutex::new(LruCache::unbounded()),
            open,
            volumeName: volumeName.to_string(),
            isDebugMode,
        }
    }

    /// 挂载
    pub fn mount(&self, mountPath: &Path, threadCount: u16) -> Result<(), MountError> {
        let mut flags = MountFlags::MOUNT_MANAGER;
        if self.readOnly {
            flags = flags | MountFlags::WRITE_PROTECT;
        }
        Drive::new()
            // 线程数(0为自动)
            .thread_count(threadCount)
            // 文件系统模式
            .flags(flags)
            // 挂载路径
            .mount_point(&convert_str(mountPath.to_str().unwrap()))
            // 超时时间
            .timeout(Duration::from_secs(5))
            // 分配单元大小
            .allocation_unit_size(1024)
            // 扇区大小
            .sector_size(1024)
            // 挂载并阻塞当前线程，直到卷被卸载
            .mount(self)
    }

    /// 卸载
    pub fn unmount(mountPath: &Path) -> bool {
        // 由dokan卸载钩子处理
        dokan::unmount(&convert_str(mountPath.to_str().unwrap()))
    }
}

#[derive(Debug, Clone)]
pub struct SevenContext {
    FileInfo: ArchiveFileInfo,
    localFilePath: PathBuf,
}

impl<'a, 'b: 'a> FileSystemHandler<'a, 'b> for ArchiveFS {
    type Context = Option<SevenContext>;

    /// 创建文件对象时调用
    fn create_file(&'b self, file_name: &U16CStr, _security_context: &DOKAN_IO_SECURITY_CONTEXT, _desired_access: u32, _file_attributes: u32, _share_access: u32, create_disposition: u32, _create_options: u32, info: &mut OperationInfo<'a, 'b, Self>) -> Result<CreateFileInfo<Self::Context>, OperationError> {
        let file_name = file_name.to_string_lossy();
        // 去除首 / 的路径
        let file_name_match = file_name.trim_start_matches("\\");
        // 文件实际缓存路径
        let localFilePath = self.extractPath.join(&file_name_match);

        // 排除特殊情况(回收站、卷标目录)
        if file_name.to_lowercase().contains("desktop.ini") || file_name.to_lowercase().contains("recycle.bin") || file_name.to_lowercase().contains("system volume information") {
            return Err(OperationError::NtStatus(STATUS_OBJECT_NAME_NOT_FOUND));
        }

        if file_name == "\\" {
            return Ok(CreateFileInfo { context: None, is_dir: true, new_file_created: false });
        }

        // writeConsole(ConsoleType::Info, &*format!("Create file: {}, createDisposition: {}, file_attributes: {}", file_name, create_disposition, file_attributes));

        // 目前问题：FILE_OPEN 需要创建文件，但按照文档要求是直接报错。如创建文件则二进制程序无法运行(对应配置文件干扰)、创建文件无限循环
        // 已知 memFS也存在此问题
        // 思路：过滤所有系统文件，FILE_OPEN 直接创建文件

        match create_disposition {
            FILE_OPEN | FILE_OPEN_IF => {
                // 匹配文件列表
                for item in self.archiveFileInfoList.iter() {
                    if file_name_match.to_lowercase() == item.Path.to_lowercase() {
                        // 更新缓存列表
                        let mut cacheList = self.cacheInfoList.lock().unwrap();
                        let _ = cacheList.get(item);
                        // 返回基本信息
                        return Ok(CreateFileInfo {
                            context: Some(SevenContext { localFilePath, FileInfo: item.clone() }),
                            is_dir: item.is_dir,
                            new_file_created: false,
                        });
                    }
                }

                // 过滤 系统文件、无关文件
                if create_disposition == FILE_OPEN || create_disposition == FILE_OVERWRITE {
                    if self.isDebugMode {
                        writeConsole(ConsoleType::Warning, &*format!("Create file failed: {}, createDisposition: {}", file_name, create_disposition));
                    }
                    return Err(OperationError::NtStatus(STATUS_OBJECT_NAME_NOT_FOUND));
                }

                if !localFilePath.exists() {
                    if info.is_dir() { fs::create_dir_all(&localFilePath).ok(); } else { fs::File::create(&localFilePath).ok(); }
                    // unsafe {
                    //     let fileName = localFilePath.as_os_str().encode_wide().chain(once(0)).collect::<Vec<u16>>();
                    //     let handle = winapi::um::fileapi::CreateFileW(fileName.as_ptr(), desired_access, share_access, null_mut(), create_disposition, file_attributes, NULL);
                    //     if handle == INVALID_HANDLE_VALUE {
                    //         return Err(OperationError::NtStatus(STATUS_OBJECT_NAME_NOT_FOUND));
                    //     }
                    // }
                }

                // 判断文件是否位于临时目录(由程序写出)
                if localFilePath.exists() {
                    return Ok(CreateFileInfo {
                        context: Some(SevenContext {
                            localFilePath: localFilePath.clone(),
                            FileInfo: ArchiveFileInfo {
                                Path: file_name_match.to_string(),
                                Size: localFilePath.metadata().unwrap().len(),
                                PackedSize: 0,
                                Modified: "".to_string(),
                                Created: None,
                                is_dir: info.is_dir(),
                            },
                        }),
                        is_dir: info.is_dir(),
                        new_file_created: false,
                    });
                }
            }
            FILE_CREATE | FILE_OVERWRITE_IF => {
                // 尝试创建文件
                if !localFilePath.exists() {
                    if info.is_dir() { fs::create_dir_all(&localFilePath).ok(); } else { fs::File::create(&localFilePath).ok(); }
                }
                return Ok(CreateFileInfo {
                    context: Some(SevenContext {
                        localFilePath: localFilePath.clone(),
                        FileInfo: ArchiveFileInfo {
                            Path: file_name_match.to_string(),
                            Size: 0,
                            PackedSize: 0,
                            Modified: "".to_string(),
                            Created: None,
                            is_dir: info.is_dir(),
                        },
                    }),
                    is_dir: info.is_dir(),
                    new_file_created: false,
                });
            }
            _ => {}
        }

        if self.isDebugMode {
            writeConsole(ConsoleType::Warning, &*format!("Create file failed: {}, createDisposition: {}", file_name, create_disposition));
        }
        Err(OperationError::NtStatus(STATUS_OBJECT_NAME_NOT_FOUND))
    }

    fn cleanup(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) {}

    fn close_file(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) {}

    /// 读取文件
    fn read_file(&'b self, file_name: &U16CStr, offset: i64, buffer: &mut [u8], _info: &OperationInfo<'a, 'b, Self>, context: &'a Self::Context) -> Result<u32, OperationError> {
        let file_name = file_name.to_string_lossy();
        if let Some(context) = context {
            if context.FileInfo.is_dir {
                return Err(OperationError::NtStatus(STATUS_INVALID_DEVICE_REQUEST));
            }

            if !context.localFilePath.exists() {
                let mut cacheList = self.cacheInfoList.lock().unwrap();
                // 自动清理缓存(循环当 缓存总大小 + 当前需要解压文件大小 >= 设置缓存大小)
                while (cacheList.iter().map(|item| item.0.Size).sum::<u64>() + context.FileInfo.Size) / 1024 / 1024 >= self.cacheSize as u64 {
                    if let Some(lruInfo) = cacheList.pop_lru() {
                        if self.isDebugMode { writeConsole(ConsoleType::Info, &*format!("Delete Cache: {}", &lruInfo.1.display())); }
                        fs::remove_file(&lruInfo.1).ok();
                    } else {
                        break;
                    }
                }

                // 解压文件
                let file_name = file_name.trim_start_matches("\\");
                if self.isDebugMode {
                    writeConsole(ConsoleType::Info, &*format!("Extracting file: {}\\{}", &*self.archivePath.to_str().unwrap(), &*file_name));
                }
                if !self.sevenZip.extractFilesFromPath(&*self.archivePath, self.password.as_deref(), &*file_name, &self.extractPath).unwrap() && !context.localFilePath.exists() {
                    if self.isDebugMode {
                        writeConsole(ConsoleType::Warning, &*format!("Extract file failed: {}\\{}", &*self.archivePath.to_str().unwrap(), &*file_name));
                    }
                    return Err(OperationError::NtStatus(STATUS_INVALID_DEVICE_REQUEST));
                }

                // 增加缓存信息
                cacheList.put(context.FileInfo.clone(), context.localFilePath.clone());
            }
            if !context.localFilePath.exists() {
                return Err(OperationError::NtStatus(STATUS_INVALID_DEVICE_REQUEST));
            }

            let file = fs::File::open(&context.localFilePath).unwrap();
            let result = file.seek_read(buffer, offset as u64).unwrap();
            return Ok(result as u32);
        }
        if self.isDebugMode {
            writeConsole(ConsoleType::Warning, &*format!("Read file failed: {}", file_name));
        }
        Err(OperationError::NtStatus(STATUS_INVALID_DEVICE_REQUEST))
    }

    /// 写入文件
    fn write_file(&'b self, file_name: &U16CStr, offset: i64, buffer: &[u8], info: &OperationInfo<'a, 'b, Self>, context: &'a Self::Context) -> Result<u32, OperationError> {
        let file_name = file_name.to_string_lossy();

        if let Some(context) = context {
            if info.is_dir() {
                fs::create_dir_all(&context.localFilePath).ok();
                return Ok(0);
                // return Err(OperationError::NtStatus(STATUS_INVALID_DEVICE_REQUEST));
            }
            // 以追加模式打开文件并尝试写入
            if let Ok(file) = OpenOptions::new().append(true).open(&context.localFilePath) {
                if let Ok(result) = file.seek_write(buffer, offset as u64) {
                    return Ok(result as u32);
                }
            }
        }

        if self.isDebugMode {
            writeConsole(ConsoleType::Warning, &*format!("Write file failed: {}", file_name));
        }
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    /// 获取文件信息
    fn get_file_information(&'b self, file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, context: &'a Self::Context) -> Result<FileInfo, OperationError> {
        let file_name = file_name.to_string_lossy();
        // 判断根目录
        if file_name == *"\\" {
            return Ok(FileInfo { attributes: FILE_ATTRIBUTES_DIRECTORY, creation_time: UNIX_EPOCH, last_access_time: UNIX_EPOCH, last_write_time: UNIX_EPOCH, file_size: 0, number_of_links: 0, file_index: 0 });
        }

        if let Some(context) = context {
            let modifiedTime = StringToSystemTime(&*context.FileInfo.Modified).unwrap_or_else(|_| SystemTime::now());
            return Ok(FileInfo {
                attributes: if context.FileInfo.is_dir { FILE_ATTRIBUTES_DIRECTORY } else { FILE_ATTRIBUTES_NORMAL },
                creation_time: modifiedTime,
                last_access_time: modifiedTime,
                last_write_time: modifiedTime,
                file_size: context.FileInfo.Size,
                number_of_links: 0,
                file_index: 0,
            });
        }
        if self.isDebugMode {
            writeConsole(ConsoleType::Warning, &*format!("Read fileInfo failed: {}", file_name));
        }
        Err(OperationError::NtStatus(STATUS_NDIS_FILE_NOT_FOUND))
    }

    /// 列出目录中的所有子项
    fn find_files(&'b self, file_name: &U16CStr, mut fill_find_data: impl FnMut(&FindData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        let matchPath = file_name.to_string_lossy();
        let mut totalFiles = Vec::new();

        // 列出压缩包内部文件
        for item in self.archiveFileInfoList.iter() {
            let filePath = format!(r"\{}", item.Path);
            // 筛选出匹配的文件(前面匹配、不等于自身、父路径匹配)
            if filePath.find(&matchPath) == Some(0) && filePath != matchPath && Path::new(&filePath).parent().unwrap().to_str().unwrap() == matchPath {
                let fileName = Path::new(&item.Path).file_name().unwrap().to_str().unwrap();
                let modifiedTime = StringToSystemTime(&*item.Modified).unwrap_or_else(|_| SystemTime::now());
                totalFiles.push(&*item.Path);
                fill_find_data(&FindData {
                    attributes: if item.is_dir { FILE_ATTRIBUTES_DIRECTORY } else { FILE_ATTRIBUTES_NORMAL },
                    creation_time: modifiedTime,
                    last_access_time: modifiedTime,
                    last_write_time: modifiedTime,
                    file_size: item.Size,
                    file_name: convert_str(fileName),
                })?;
            }
        }

        // 列出缓存目录文件
        if !self.readOnly {
            let path = self.extractPath.join(matchPath.trim_start_matches("\\"));
            if path.exists() {
                for entry in path.read_dir().unwrap() {
                    if let Ok(item) = entry {
                        // 排除临时解压的文件
                        if totalFiles.iter().filter(|relativePath| item.path().ends_with(relativePath)).count() > 0 {
                            continue;
                        }
                        let metadata = &item.path().metadata().unwrap();
                        fill_find_data(&FindData {
                            attributes: if item.path().is_dir() { FILE_ATTRIBUTES_DIRECTORY } else { FILE_ATTRIBUTES_NORMAL },
                            creation_time: SystemTime::now(),
                            last_access_time: SystemTime::now(),
                            last_write_time: SystemTime::now(),
                            file_size: metadata.len(),
                            file_name: convert_str(item.path().file_name().unwrap().to_str().unwrap()),
                        })?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 设置虚拟文件系统信息
    fn get_disk_free_space(&'b self, _info: &OperationInfo<'a, 'b, Self>) -> Result<DiskSpaceInfo, OperationError> {
        // 计算 设定缓存大小 - 当前缓存占用大小
        let cacheList = self.cacheInfoList.lock().unwrap();
        let currentAvailableSize = self.cacheSize as u64 * 1024 * 1024 - (cacheList.iter().map(|item| item.0.Size).sum::<u64>());
        Ok(DiskSpaceInfo {
            // 存储空间总大小(缓存最大空间)
            byte_count: self.cacheSize as u64 * 1024 * 1024,
            // 可用空间量
            free_byte_count: currentAvailableSize,
            // 调用线程关联的用户可用的可用空间总量
            available_byte_count: currentAvailableSize,
        })
    }

    /// 获取卷信息
    fn get_volume_information(&'b self, _info: &OperationInfo<'a, 'b, Self>) -> Result<VolumeInfo, OperationError> {
        let mut fs_flags = FILE_CASE_PRESERVED_NAMES | FILE_UNICODE_ON_DISK | FILE_VOLUME_IS_COMPRESSED | FILE_PERSISTENT_ACLS;
        if self.readOnly {
            fs_flags = fs_flags | FILE_READ_ONLY_VOLUME;
        }
        Ok(VolumeInfo {
            name: convert_str(&self.volumeName),
            serial_number: 0,
            max_component_length: 255,
            fs_flags,
            fs_name: convert_str("NTFS"),
        })
    }

    /// 挂载后钩子
    fn mounted(&'b self, info: &OperationInfo<'a, 'b, Self>) -> Result<(), OperationError> {
        let mount_point = info.mount_point().unwrap().to_string_lossy();
        // if Path::new(&mount_point).is_dir() {
        //     writeConsole(ConsoleType::Err, "Mounted archive failed");
        //     process::exit(0x0100);
        // }
        writeConsole(ConsoleType::Success, "Mounted archive successfully");
        if self.open {
            let _ = Command::new("explorer").arg(mount_point).output().unwrap();
        }
        Ok(())
    }

    /// 卸载后钩子
    fn unmounted(&'b self, info: &OperationInfo<'a, 'b, Self>) -> Result<(), OperationError> {
        let mount_point = info.mount_point().unwrap().to_string_lossy();
        // 尝试卸载虚拟盘符
        removeVirtualDrive(Path::new(&mount_point));
        // 尝试删除挂载目录
        fs::remove_dir_all(&mount_point).ok();
        // 清理缓存目录
        fs::remove_dir_all(&self.extractPath).ok();
        Ok(())
    }

    fn delete_file(&'b self, file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, context: &'a Self::Context) -> Result<(), OperationError> {
        let file_name = file_name.to_string_lossy();
        if let Some(context) = context {
            if context.localFilePath.exists() {
                fs::remove_file(&context.localFilePath).ok();
                return Ok(());
            }
        }
        writeConsole(ConsoleType::Warning, &*format!("Delete file failed: {}", file_name));
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn delete_directory(&'b self, file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        let file_name = file_name.to_string_lossy();
        writeConsole(ConsoleType::Warning, &*format!("Delete directory failed: {}", file_name));
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn move_file(&'b self, file_name: &U16CStr, _new_file_name: &U16CStr, _replace_if_existing: bool, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        let file_name = file_name.to_string_lossy();
        writeConsole(ConsoleType::Warning, &*format!("Move file failed: {}", file_name));

        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn set_end_of_file(&'b self, _file_name: &U16CStr, _offset: i64, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn set_allocation_size(&'b self, _file_name: &U16CStr, _alloc_size: i64, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn lock_file(&'b self, _file_name: &U16CStr, _offset: i64, _length: i64, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn unlock_file(&'b self, _file_name: &U16CStr, _offset: i64, _length: i64, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }
}

fn convert_str(s: impl AsRef<str>) -> U16CString {
    unsafe { U16CString::from_str_unchecked(s) }
}
