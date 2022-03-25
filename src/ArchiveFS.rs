use std::path::{PathBuf, Path};
use std::fs;
use std::os::windows::fs::FileExt;
use std::time::{Duration, UNIX_EPOCH};
use crate::utils::util::convert_str;
use crate::sevenzip::sevenZip;
use crate::sevenzip::ArchiveFileInfo;
use crate::utils::console::{writeConsole, ConsoleType};
use dokan::{FileSystemHandler, DOKAN_IO_SECURITY_CONTEXT, CreateFileInfo, OperationInfo, OperationError, FileInfo, FindData, FillDataError, VolumeInfo, DiskSpaceInfo, FileTimeInfo, FindStreamData};
use widestring::{U16CStr, U16CString};
use winapi::shared::ntstatus::{STATUS_ACCESS_DENIED, STATUS_NOT_IMPLEMENTED, STATUS_NOT_A_DIRECTORY, STATUS_INVALID_PARAMETER};
use winapi::um::{securitybaseapi};
use winapi::um::winnt::{FILE_CASE_PRESERVED_NAMES, FILE_UNICODE_ON_DISK, FILE_READ_ONLY_VOLUME, FILE_VOLUME_IS_COMPRESSED};
use std::ffi::c_void;
use std::sync::mpsc::SyncSender;
use winapi::_core::pin::Pin;
use dokan::{MountFlags};

#[derive(Debug)]
pub struct ArchiveFS {
    /// SevenZIP程序类
    sevenZip: sevenZip,
    /// 压缩包路径
    archivePath: PathBuf,
    /// 临时释放路径
    extractPath: PathBuf,
    /// 挂载路径根文件名
    parentName: String,
    /// 压缩包文件信息
    archiveFileInfoList: Vec<ArchiveFileInfo>,
}

impl ArchiveFS {
    pub(crate) fn new(archivePath: &Path, extractPath: &Path, parentName: &str) -> ArchiveFS {
        ArchiveFS {
            sevenZip: sevenZip::new().unwrap(),
            archivePath: (*archivePath.to_path_buf()).to_owned(),
            extractPath: (*extractPath.to_path_buf()).to_owned(),
            parentName: parentName.to_string(),
            archiveFileInfoList: sevenZip::new().unwrap().listArchiveFiles(archivePath).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct SevenContext {}

impl<'a, 'b: 'a> FileSystemHandler<'a, 'b> for ArchiveFS {
    type Context = Option<SevenContext>;

    // 创建文件对象时调用
    fn create_file(&'b self, file_name: &U16CStr, _security_context: &DOKAN_IO_SECURITY_CONTEXT, _desired_access: u32, _file_attributes: u32, _share_access: u32, _create_disposition: u32, _create_options: u32, info: &mut OperationInfo<'a, 'b, Self>) -> Result<CreateFileInfo<Self::Context>, OperationError> {
        let file_name = file_name.to_string_lossy();

        if file_name == "\\".to_string() || file_name == format!("\\{}", &self.parentName) {
            return Ok(CreateFileInfo { context: None, is_dir: true, new_file_created: false, });
        }

        for item in self.archiveFileInfoList.iter() {
            let file_name = file_name.trim_start_matches(format!("\\{}\\", self.parentName).as_str());
            if file_name == item.Path {
                // println!("创建文件: {:?}", &item);
                return Ok(CreateFileInfo {
                    context: None,
                    is_dir: false,
                    new_file_created: false,
                });
            }
        }

        println!("创建文件失败: {:?}", file_name);
        // return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
        return Err(OperationError::NtStatus(123));
    }

    fn cleanup(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) {}

    fn close_file(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) {}

    /// 读取文件
    fn read_file(&'b self, file_name: &U16CStr, offset: i64, buffer: &mut [u8], _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<u32, OperationError> {
        let file_name = file_name.to_string_lossy();
        // 获取实际相对路径
        let file_name = file_name.trim_start_matches(format!("\\{}\\", self.parentName).as_str());
        let localFilePath = &self.extractPath.join(file_name);
        if !localFilePath.exists() {
            // 解压文件
            if !self.sevenZip.extractFilesFromPath(&*self.archivePath, &*file_name, &self.extractPath).unwrap() {
                println!("File decompression failed");
                return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
            }
            writeConsole(ConsoleType::Info, &*format!("Unzip File: {}\\{}", &*self.archivePath.to_str().unwrap(), &*file_name));
        }
        if !localFilePath.exists() {
            // return Ok(0);
            return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
        }
        // println!("读取文件: {}, buffer Size: {}, offset: {}", file_name, buffer.len(), offset);
        // 读取文件
        let file = fs::File::open(localFilePath).expect("文件打开失败");
        let result = file.seek_read(buffer, offset as u64).unwrap();
        Ok(result as u32)
    }

    fn write_file(&'b self, _file_name: &U16CStr, _offset: i64, _buffer: &[u8], _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<u32, OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn flush_file_buffers(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        return Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED));
    }

    // 获取文件信息
    fn get_file_information(&'b self, file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<FileInfo, OperationError> {
        let file_name = file_name.to_string_lossy();
        if file_name == r"\".to_string() || file_name == format!(r"\{}", &self.parentName) {
            return Ok(FileInfo {
                // https://docs.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
                attributes: 16,
                creation_time: UNIX_EPOCH,
                last_access_time: UNIX_EPOCH + Duration::from_secs(1),
                last_write_time: UNIX_EPOCH + Duration::from_secs(2),
                file_size: 0,
                number_of_links: 0,
                file_index: 0,
            });
        }
        // 获取实际相对路径
        let file_name = file_name.trim_start_matches(format!("\\{}\\", self.parentName).as_str());
        for item in self.archiveFileInfoList.iter() {
            if file_name == item.Path {
                // println!("查看信息: {}", file_name);
                return Ok(FileInfo {
                    // https://docs.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
                    attributes: if item.Attributes == "D" { 16 } else { 128 },
                    creation_time: UNIX_EPOCH,
                    last_access_time: UNIX_EPOCH + Duration::from_secs(1),
                    last_write_time: UNIX_EPOCH + Duration::from_secs(2),
                    file_size: item.Size,
                    number_of_links: 0,
                    file_index: item.Size * 2,
                });
            }
        }

        return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
    }

    // 列出目录中的所有子项
    fn find_files(&'b self, file_name: &U16CStr, mut fill_find_data: impl FnMut(&FindData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        let path = file_name.to_string_lossy();
        // println!("枚举路径: {}", &path);

        // 挂载根路径显示 压缩包名.格式 目录
        if path == "\\".to_string() {
            fill_find_data(&FindData {
                attributes: 16,
                creation_time: UNIX_EPOCH,
                last_access_time: UNIX_EPOCH + Duration::from_secs(1),
                last_write_time: UNIX_EPOCH + Duration::from_secs(2),
                file_size: self.archivePath.metadata().unwrap().len(),
                file_name: convert_str(&self.parentName),
            })?;
            return Ok(());
        }

        for item in self.archiveFileInfoList.iter() {
            let filePath = format!(r"\{}\{}", &self.parentName, item.Path);

            // 筛选出匹配的文件(前面匹配、不等于自身、父路径匹配)
            if filePath.find(&path) == Some(0) && filePath != path && Path::new(&filePath).parent().unwrap().to_str().unwrap() == &path {
                let fileName = Path::new(&item.Path).file_name().unwrap().to_str().unwrap();
                fill_find_data(&FindData {
                    attributes: if item.Attributes == "D" { 16 } else { 128 },
                    creation_time: UNIX_EPOCH,
                    last_access_time: UNIX_EPOCH + Duration::from_secs(1),
                    last_write_time: UNIX_EPOCH + Duration::from_secs(2),
                    file_size: item.Size,
                    file_name: convert_str(fileName),
                })?;
            }
        }
        Ok(())
    }

    fn find_files_with_pattern(&'b self, _file_name: &U16CStr, _pattern: &U16CStr, _fill_find_data: impl FnMut(&FindData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        return Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED));
    }

    fn set_file_attributes(&'b self, _file_name: &U16CStr, _file_attributes: u32, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        return Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED));
    }

    fn set_file_time(&'b self, _file_name: &U16CStr, _creation_time: FileTimeInfo, _last_access_time: FileTimeInfo, _last_write_time: FileTimeInfo, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
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

    // 设置虚拟文件系统信息
    fn get_disk_free_space(&'b self, _info: &OperationInfo<'a, 'b, Self>) -> Result<DiskSpaceInfo, OperationError> {
        Ok(DiskSpaceInfo {
            // 存储空间总大小
            byte_count: 2048 * 1024 * 1024,
            // 可用空间量
            free_byte_count: 2048 * 1024 * 1024,
            // 调用线程关联的用户可用的可用空间总量
            available_byte_count: 2048 * 1024 * 1024,
        })
    }

    // 获取卷信息
    fn get_volume_information(&'b self, _info: &OperationInfo<'a, 'b, Self>) -> Result<VolumeInfo, OperationError> {
        Ok(VolumeInfo {
            name: convert_str("ArchiveMount"),
            serial_number: 1,
            max_component_length: 255,
            fs_flags: FILE_CASE_PRESERVED_NAMES | FILE_UNICODE_ON_DISK | FILE_VOLUME_IS_COMPRESSED | FILE_READ_ONLY_VOLUME,
            fs_name: convert_str("NTFS"),
        })
    }

    fn mounted(&'b self, _info: &OperationInfo<'a, 'b, Self>) -> Result<(), OperationError> {
        Ok(())
    }

    fn unmounted(&'b self, _info: &OperationInfo<'a, 'b, Self>) -> Result<(), OperationError> {
        Ok(())
    }

    // 设置可执行文件安全信息
    fn get_file_security(&'b self, _file_name: &U16CStr, _security_information: u32, _security_descriptor: *mut c_void, _buffer_length: u32, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<u32, OperationError> {
        return Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED));
    }

    fn set_file_security(&'b self, _file_name: &U16CStr, _security_information: u32, _security_descriptor: *mut c_void, _buffer_length: u32, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn find_streams(&'b self, _file_name: &U16CStr, _fill_find_stream_data: impl FnMut(&FindStreamData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        return Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED));
    }
}
