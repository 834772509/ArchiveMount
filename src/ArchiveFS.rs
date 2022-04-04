use std::path::{PathBuf, Path};
use std::fs;
use std::os::windows::fs::FileExt;
use std::time::{Duration, UNIX_EPOCH};
use crate::utils::util::convert_str;
use crate::sevenzip::sevenZip;
use crate::sevenzip::ArchiveFileInfo;
use crate::utils::console::{writeConsole, ConsoleType};
use dokan::{FileSystemHandler, DOKAN_IO_SECURITY_CONTEXT, CreateFileInfo, OperationInfo, OperationError, FileInfo, FindData, FillDataError, VolumeInfo, DiskSpaceInfo, FileTimeInfo, FindStreamData};
use widestring::{U16CStr};
use winapi::shared::ntstatus::{STATUS_ACCESS_DENIED, STATUS_NOT_IMPLEMENTED, STATUS_DISK_FULL, STATUS_SUCCESS, STATUS_NDIS_FILE_NOT_FOUND};
use winapi::um::winnt::{FILE_CASE_PRESERVED_NAMES, FILE_UNICODE_ON_DISK, FILE_READ_ONLY_VOLUME, FILE_VOLUME_IS_COMPRESSED};
use std::ffi::c_void;
use std::fs::File;

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

impl<'a, 'b: 'a> FileSystemHandler<'a, 'b> for ArchiveFS {
    type Context = Option<File>;

    // 创建文件对象时调用
    fn create_file(&'b self,
                   file_name: &U16CStr, security_context: &DOKAN_IO_SECURITY_CONTEXT,
                   _desired_access: u32, _file_attributes: u32, _share_access: u32,
                   create_disposition: u32,
                   _create_options: u32, info: &mut OperationInfo<'a, 'b, Self>) -> Result<CreateFileInfo<Self::Context>, OperationError> {
        if create_disposition != FILE_OPEN && create_disposition != FILE_OPEN_IF {
            return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
        }

        let file_name = file_name.to_string_lossy();
        if file_name == "\\".to_string() || file_name == format!("\\{}", &self.parentName) {
            return Ok(CreateFileInfo { context: None, is_dir: true, new_file_created: false });
        }

        for item in self.archiveFileInfoList.iter() {
            let file_name = file_name.trim_start_matches(format!("\\{}\\", self.parentName).as_str());
            let localFilePath = &self.extractPath.join(file_name);

            if file_name == item.Path {
                if !localFilePath.exists() && !self.sevenZip.extractFilesFromPath(&*self.archivePath, &*file_name, &self.extractPath).unwrap() {
                    println!("File decompression failed");
                    return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
                }
                return Ok(CreateFileInfo {
                    context: Some(fs::File::open(localFilePath).unwrap()),
                    is_dir: false,
                    new_file_created: false,
                });
            }
        }

        // if file_name == r"\test.7z\程序文件.exe.Config".to_string() { return Ok(CreateFileInfo { context: None, is_dir: false, new_file_created: false }); }

        println!("创建文件失败: {:?}", file_name);
        return Err(OperationError::NtStatus(STATUS_NDIS_FILE_NOT_FOUND));
        // return Err(OperationError::NtStatus(STATUS_DISK_FULL));
        // return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
    }

    fn cleanup(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) {}

    fn close_file(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) {}

    /// 读取文件
    fn read_file(&'b self, file_name: &U16CStr, offset: i64, buffer: &mut [u8], _info: &OperationInfo<'a, 'b, Self>, context: &'a Self::Context) -> Result<u32, OperationError> {
        let file_name = file_name.to_string_lossy();
        // println!("读取文件: {}, buffer Size: {}, offset: {}", file_name.to_string_lossy(), buffer.len(), offset);
        if let Some(file) = context {
            let result = file.seek_read(buffer, offset as u64).unwrap();
            return Ok(result as u32);
        }
        println!("读取文件失败: {}", file_name);
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
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
                attributes: FILE_ATTRIBUTES_DIRECTORY,
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
                return Ok(FileInfo {
                    // https://docs.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
                    attributes: if item.Attributes == "D" { FILE_ATTRIBUTES_DIRECTORY } else { FILE_ATTRIBUTES_NORMAL },
                    creation_time: UNIX_EPOCH,
                    last_access_time: UNIX_EPOCH + Duration::from_secs(1),
                    last_write_time: UNIX_EPOCH + Duration::from_secs(2),
                    file_size: item.Size,
                    number_of_links: 0,
                    file_index: item.Size * 2,
                });
            }
        }
        println!("查看文件信息错误: {}", file_name);
        return Err(OperationError::NtStatus(STATUS_NDIS_FILE_NOT_FOUND));
    }

    // 列出目录中的所有子项
    fn find_files(&'b self, file_name: &U16CStr, mut fill_find_data: impl FnMut(&FindData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        let path = file_name.to_string_lossy();

        // 挂载根路径显示 压缩包名.格式 目录
        if path == "\\".to_string() {
            fill_find_data(&FindData {
                attributes: FILE_ATTRIBUTES_DIRECTORY,
                creation_time: UNIX_EPOCH,
                last_access_time: UNIX_EPOCH + Duration::from_secs(1),
                last_write_time: UNIX_EPOCH + Duration::from_secs(2),
                file_size: self.archivePath.metadata().unwrap().len(),
                file_name: convert_str(&self.parentName),
            })?;
            return Ok(());
        }

        // 列出压缩包内文件结构
        for item in self.archiveFileInfoList.iter() {
            let filePath = format!(r"\{}\{}", &self.parentName, item.Path);
            // 筛选出匹配的文件(前面匹配、不等于自身、父路径匹配)
            if filePath.find(&path) == Some(0) && filePath != path && Path::new(&filePath).parent().unwrap().to_str().unwrap() == &path {
                let fileName = Path::new(&item.Path).file_name().unwrap().to_str().unwrap();
                fill_find_data(&FindData {
                    attributes: if item.Attributes == "D" { FILE_ATTRIBUTES_DIRECTORY } else { FILE_ATTRIBUTES_NORMAL },
                    creation_time: UNIX_EPOCH,
                    last_access_time: UNIX_EPOCH + Duration::from_secs(1),
                    last_write_time: UNIX_EPOCH + Duration::from_secs(2),
                    file_size: item.Size,
                    file_name: convert_str(fileName),
                })?;
            }
        }

        if path == format!(r"\{}", &self.parentName).to_string() {
            fill_find_data(&FindData {
                attributes: FILE_ATTRIBUTES_DIRECTORY,
                creation_time: UNIX_EPOCH,
                last_access_time: UNIX_EPOCH + Duration::from_secs(1),
                last_write_time: UNIX_EPOCH + Duration::from_secs(2),
                file_size: 0,
                file_name: convert_str(&self.parentName),
            })?;
            return Ok(());
        }

        println!("枚举路径失败: {}", &path);
        return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
    }

    fn find_files_with_pattern(&'b self, _file_name: &U16CStr, _pattern: &U16CStr, _fill_find_data: impl FnMut(&FindData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        return Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED));
    }

    fn set_file_attributes(&'b self, _file_name: &U16CStr, _file_attributes: u32, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        return Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED));
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
    fn get_file_security(&'b self, file_name: &U16CStr, _security_information: u32, security_descriptor: *mut c_void, _buffer_length: u32, _info: &OperationInfo<'a, 'b, Self>, context: &'a Self::Context) -> Result<u32, OperationError> {
        let file_name = file_name.to_string_lossy();
        if let Some(file) = context {
            unsafe {
                // winapi::um::securitybaseapi::GetFileSecurityW(
                //     aaa,
                //     bbb,
                //     ccc,
                //     ddd,
                //     eee,
                // );
            }
        }
        println!("获取安全信息失败: {}", file_name);
        return Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED));
    }

    fn set_file_security(&'b self, _file_name: &U16CStr, _security_information: u32, _security_descriptor: *mut c_void, _buffer_length: u32, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn find_streams(&'b self, _file_name: &U16CStr, _fill_find_stream_data: impl FnMut(&FindStreamData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        return Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED));
    }
}
