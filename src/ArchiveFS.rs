use std::ffi::c_void;
use std::fs;
use std::os::windows::fs::FileExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dokan::{CreateFileInfo, DiskSpaceInfo, DOKAN_IO_SECURITY_CONTEXT, Drive, FileInfo, FileSystemHandler, FileTimeInfo, FillDataError, FindData, FindStreamData, MountError, MountFlags, OperationError, OperationInfo, VolumeInfo};
use lru::LruCache;
use widestring::{U16CStr, U16CString};
use winapi::shared::ntstatus::{STATUS_ACCESS_DENIED, STATUS_INVALID_DEVICE_REQUEST, STATUS_NDIS_FILE_NOT_FOUND, STATUS_NOT_IMPLEMENTED, STATUS_OBJECT_NAME_NOT_FOUND};
use winapi::um::winnt::{FILE_CASE_PRESERVED_NAMES, FILE_PERSISTENT_ACLS, FILE_READ_ONLY_VOLUME, FILE_UNICODE_ON_DISK, FILE_VOLUME_IS_COMPRESSED};

use crate::QUIET;
use crate::sevenZip::{ArchiveFileInfo, sevenZip};
use crate::utils::console::{ConsoleType, writeConsole};
use crate::utils::util::StringToSystemTime;

const FILE_ATTRIBUTES_ARCHIVE: u32 = 32;
const FILE_ATTRIBUTES_DIRECTORY: u32 = 16;
const FILE_ATTRIBUTES_LOCALLYINCOMPLETE: u32 = 512;
const FILE_ATTRIBUTES_NORMAL: u32 = 0;
const FILE_ATTRIBUTES_READONLY: u32 = 1;
const FILE_ATTRIBUTES_TEMPORARY: u32 = 256;

const FILE_SUPERSEDE: u32 = 0;
const FILE_OPEN: u32 = 1;
const FILE_CREATE: u32 = 2;
const FILE_OPEN_IF: u32 = 3;
const FILE_OVERWRITE: u32 = 4;
const FILE_OVERWRITE_IF: u32 = 5;
const FILE_MAXIMUM_DISPOSITION: u32 = 5;

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
    /// 压缩包文件信息
    archiveFileInfoList: Vec<ArchiveFileInfo>,
    /// 缓存信息
    cacheInfoList: Mutex<LruCache<ArchiveFileInfo, PathBuf>>,
    /// 挂载后是否打开
    open: bool,
    /// 挂载卷标名
    volumeName: String,
    /// 挂载点父目录名
    parentName: Option<String>,
    /// 是否为调试模式
    isDebugMode: bool,
}

impl ArchiveFS {
    pub(crate) fn new(archivePath: &Path, extractPath: &Path, password: Option<&str>, cacheSize: u64, archiveFileInfoList: Vec<ArchiveFileInfo>, open: bool, volumeName: &str, isDebugMode: bool) -> ArchiveFS {
        let _ = fs::create_dir_all(extractPath);
        ArchiveFS {
            sevenZip: sevenZip::new().unwrap(),
            archivePath: (*archivePath.to_path_buf()).to_owned(),
            password: password.map(|password| password.to_string()),
            extractPath: (*extractPath.to_path_buf()).to_owned(),
            cacheSize,
            archiveFileInfoList,
            cacheInfoList: Mutex::new(LruCache::unbounded()),
            open,
            volumeName: volumeName.to_string(),
            parentName: None,
            isDebugMode,
        }
    }

    /// 挂载
    pub fn mount(&self, mountPath: &Path, threadCount: u16) -> Result<(), MountError> {
        Drive::new()
            // 线程数(0为自动)
            .thread_count(threadCount)
            // 文件系统模式
            .flags(MountFlags::WRITE_PROTECT | MountFlags::MOUNT_MANAGER | MountFlags::ENABLE_UNOUNT_NETWORK_DRIVE)
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
        dokan::unmount(&convert_str(mountPath.to_str().unwrap()))
    }
}

#[derive(Debug)]
pub struct SevenContext {
    FileInfo: ArchiveFileInfo,
    localFilePath: PathBuf,
}

impl<'a, 'b: 'a> FileSystemHandler<'a, 'b> for ArchiveFS {
    type Context = Option<SevenContext>;

    /// 创建文件对象时调用
    fn create_file(&'b self, file_name: &U16CStr, _security_context: &DOKAN_IO_SECURITY_CONTEXT, _desired_access: u32, _file_attributes: u32, _share_access: u32, create_disposition: u32, _create_options: u32, _info: &mut OperationInfo<'a, 'b, Self>) -> Result<CreateFileInfo<Self::Context>, OperationError> {
        if create_disposition != FILE_OPEN && create_disposition != FILE_OPEN_IF {
            return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
        }
        let file_name = file_name.to_string_lossy();

        let trimMatch = match &self.parentName {
            None => "\\".to_string(),
            Some(directoryName) => format!("\\{}\\", directoryName)
        };

        if file_name == *"\\" || file_name == trimMatch.trim_end_matches('\\') {
            return Ok(CreateFileInfo { context: None, is_dir: true, new_file_created: false });
        }

        // 匹配文件列表
        for item in self.archiveFileInfoList.iter() {
            let file_name = file_name.trim_start_matches(&trimMatch);
            let localFilePath = self.extractPath.join(file_name);
            if file_name.to_lowercase() == item.Path.to_lowercase() {
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

        if self.isDebugMode {
            writeConsole(ConsoleType::Warning, &*format!("Create file failed: {}", file_name));
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
                        if self.isDebugMode {
                            writeConsole(ConsoleType::Info, &*format!("Delete Cache: {}", &lruInfo.1.display()));
                        }
                        let _ = fs::remove_file(&lruInfo.1);
                    } else {
                        break;
                    }
                }
                // 解压文件
                let trimMatch = match &self.parentName {
                    None => "\\".to_string(),
                    Some(directoryName) => format!("\\{}\\", directoryName)
                };
                let file_name = file_name.trim_start_matches(&trimMatch);
                if self.isDebugMode {
                    writeConsole(ConsoleType::Info, &*format!("Extracting file: {}\\{}", &*self.archivePath.to_str().unwrap(), &*file_name));
                }
                if !self.sevenZip.extractFilesFromPath(&*self.archivePath, self.password.as_deref(), &*file_name, &self.extractPath).unwrap() && !context.localFilePath.exists() {
                    return Err(OperationError::NtStatus(STATUS_INVALID_DEVICE_REQUEST));
                }
                // 增加缓存信息
                cacheList.put(context.FileInfo.clone(), context.localFilePath.clone());
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

    fn write_file(&'b self, _file_name: &U16CStr, _offset: i64, _buffer: &[u8], _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<u32, OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn flush_file_buffers(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED))
    }

    /// 获取文件信息
    fn get_file_information(&'b self, file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, context: &'a Self::Context) -> Result<FileInfo, OperationError> {
        let file_name = file_name.to_string_lossy();
        // 判断目录
        if file_name == *"\\" {
            return Ok(FileInfo { attributes: FILE_ATTRIBUTES_DIRECTORY, creation_time: UNIX_EPOCH, last_access_time: UNIX_EPOCH, last_write_time: UNIX_EPOCH, file_size: 0, number_of_links: 0, file_index: 0 });
        }
        if let Some(parentName) = &self.parentName {
            if file_name == format!("\\{}", parentName) {
                return Ok(FileInfo { attributes: FILE_ATTRIBUTES_DIRECTORY, creation_time: UNIX_EPOCH, last_access_time: UNIX_EPOCH, last_write_time: UNIX_EPOCH, file_size: 0, number_of_links: 0, file_index: 0 });
            }
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

        if self.parentName.is_some() && matchPath == "\\" {
            return Ok(fill_find_data(&FindData {
                attributes: FILE_ATTRIBUTES_DIRECTORY,
                creation_time: SystemTime::now(),
                last_access_time: SystemTime::now(),
                last_write_time: SystemTime::now(),
                file_size: 0,
                file_name: convert_str(&self.parentName.as_ref().unwrap()),
            })?);
        }

        // 列出压缩包内文件结构
        for item in self.archiveFileInfoList.iter() {
            let filePath = match &self.parentName {
                None => format!(r"\{}", item.Path),
                Some(directoryName) => format!(r"\{}\{}", directoryName, item.Path)
            };
            // 筛选出匹配的文件(前面匹配、不等于自身、父路径匹配)
            if filePath.find(&matchPath) == Some(0) && filePath != matchPath && Path::new(&filePath).parent().unwrap().to_str().unwrap() == matchPath {
                let fileName = Path::new(&item.Path).file_name().unwrap().to_str().unwrap();
                let modifiedTime = StringToSystemTime(&*item.Modified).unwrap_or_else(|_| SystemTime::now());
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
        Ok(())
    }

    fn find_files_with_pattern(&'b self, _file_name: &U16CStr, _pattern: &U16CStr, _fill_find_data: impl FnMut(&FindData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED))
    }

    fn set_file_attributes(&'b self, _file_name: &U16CStr, _file_attributes: u32, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED))
    }

    fn set_file_time(&'b self, _file_name: &U16CStr, _creation_time: FileTimeInfo, _last_access_time: FileTimeInfo, _last_write_time: FileTimeInfo, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED))
    }

    fn delete_file(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn delete_directory(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn move_file(&'b self, _file_name: &U16CStr, _new_file_name: &U16CStr, _replace_if_existing: bool, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
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
        Ok(VolumeInfo {
            name: convert_str(&self.volumeName),
            serial_number: 0,
            max_component_length: 255,
            fs_flags: FILE_CASE_PRESERVED_NAMES | FILE_UNICODE_ON_DISK | FILE_VOLUME_IS_COMPRESSED | FILE_READ_ONLY_VOLUME | FILE_PERSISTENT_ACLS,
            fs_name: convert_str("NTFS"),
        })
    }

    fn mounted(&'b self, info: &OperationInfo<'a, 'b, Self>) -> Result<(), OperationError> {
        let mount_point = info.mount_point().unwrap().to_string_lossy();
        // if Path::new(&mount_point).is_dir() {
        //     writeConsole(ConsoleType::Err, "Mounted archive failed");
        //     process::exit(0x0100);
        // }
        if !*QUIET.lock().unwrap() {
            writeConsole(ConsoleType::Success, "Mounted archive successfully");
        }
        if self.open {
            let _ = Command::new("explorer").arg(mount_point).output().unwrap();
        }
        Ok(())
    }

    fn unmounted(&'b self, info: &OperationInfo<'a, 'b, Self>) -> Result<(), OperationError> {
        let mount_point = info.mount_point().unwrap().to_string_lossy();
        // 尝试删除挂载目录
        let _ = fs::remove_dir_all(mount_point);
        // 清理缓存目录
        let _ = fs::remove_dir_all(&self.extractPath);
        Ok(())
    }

    fn get_file_security(&'b self, _file_name: &U16CStr, _security_information: u32, _security_descriptor: *mut c_void, _buffer_length: u32, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<u32, OperationError> {
        Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED))
    }

    fn set_file_security(&'b self, _file_name: &U16CStr, _security_information: u32, _security_descriptor: *mut c_void, _buffer_length: u32, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn find_streams(&'b self, _file_name: &U16CStr, _fill_find_stream_data: impl FnMut(&FindStreamData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED))
    }
}

fn convert_str(s: impl AsRef<str>) -> U16CString {
    unsafe { U16CString::from_str_unchecked(s) }
}
