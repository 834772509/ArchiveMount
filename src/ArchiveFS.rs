use std::path::{PathBuf, Path};
use std::fs;
use std::os::windows::fs::FileExt;
use std::time::{UNIX_EPOCH};
use std::ffi::{c_void};
use crate::utils::util::{convert_str, StringToSystemTime};
use crate::sevenzip::{sevenZip, ArchiveFileInfo};
use crate::utils::console::{writeConsole, ConsoleType};
use dokan::{FileSystemHandler, DOKAN_IO_SECURITY_CONTEXT, CreateFileInfo, OperationInfo, OperationError, FileInfo, FindData, FillDataError, VolumeInfo, DiskSpaceInfo, FileTimeInfo, FindStreamData};
use widestring::{U16CStr};
use winapi::shared::ntstatus::{STATUS_ACCESS_DENIED, STATUS_NOT_IMPLEMENTED, STATUS_NDIS_FILE_NOT_FOUND, STATUS_INVALID_DEVICE_REQUEST, STATUS_OBJECT_NAME_NOT_FOUND};
use winapi::um::winnt::{FILE_CASE_PRESERVED_NAMES, FILE_UNICODE_ON_DISK, FILE_READ_ONLY_VOLUME, FILE_VOLUME_IS_COMPRESSED, FILE_PERSISTENT_ACLS};

#[derive(Debug)]
pub struct ArchiveFS {
    /// SevenZIP程序类
    sevenZip: sevenZip,
    /// 压缩包路径
    archivePath: PathBuf,
    /// 临时释放路径
    extractPath: PathBuf,
    /// 压缩包文件信息
    archiveFileInfoList: Vec<ArchiveFileInfo>,
}

impl ArchiveFS {
    pub(crate) fn new(archivePath: &Path, extractPath: &Path, password: Option<&str>) -> ArchiveFS {
        ArchiveFS {
            sevenZip: sevenZip::new().unwrap(),
            archivePath: (*archivePath.to_path_buf()).to_owned(),
            extractPath: (*extractPath.to_path_buf()).to_owned(),
            archiveFileInfoList: sevenZip::new().unwrap().listArchiveFiles(archivePath, password).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct SevenContext {
    localFilePath: PathBuf,
    FileInfo: ArchiveFileInfo,
}

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
    type Context = Option<SevenContext>;

    // 创建文件对象时调用
    fn create_file(&'b self, file_name: &U16CStr, _security_context: &DOKAN_IO_SECURITY_CONTEXT, _desired_access: u32, _file_attributes: u32, _share_access: u32, create_disposition: u32, _create_options: u32, _info: &mut OperationInfo<'a, 'b, Self>) -> Result<CreateFileInfo<Self::Context>, OperationError> {
        if create_disposition != FILE_OPEN && create_disposition != FILE_OPEN_IF {
            return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
        }

        let file_name = file_name.to_string_lossy();
        if file_name == *"\\" {
            return Ok(CreateFileInfo { context: None, is_dir: true, new_file_created: false });
        }

        for item in self.clone().archiveFileInfoList.iter() {
            let file_name = file_name.trim_start_matches('\\');
            let localFilePath = self.extractPath.join(file_name);
            if file_name == item.Path {
                if !&item.is_dir && !localFilePath.exists() {
                    // 解压文件
                    writeConsole(ConsoleType::Info, &*format!("unzip files: {}\\{}", &*self.archivePath.to_str().unwrap(), &*file_name));
                    if !self.sevenZip.extractFilesFromPath(&*self.archivePath, &*file_name, &self.extractPath).unwrap() && !localFilePath.exists() {
                        println!("File decompression failed");
                        return Err(OperationError::NtStatus(STATUS_ACCESS_DENIED));
                    }
                }
                return Ok(CreateFileInfo {
                    context: Some(SevenContext { localFilePath, FileInfo: item.clone() }),
                    is_dir: item.is_dir,
                    new_file_created: false,
                });
            }
        }
        // println!("创建文件失败: {}", file_name);
        Err(OperationError::NtStatus(STATUS_OBJECT_NAME_NOT_FOUND))
    }

    fn cleanup(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) {}

    fn close_file(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) {}

    /// 读取文件
    fn read_file(&'b self, file_name: &U16CStr, offset: i64, buffer: &mut [u8], _info: &OperationInfo<'a, 'b, Self>, context: &'a Self::Context) -> Result<u32, OperationError> {
        let file_name = file_name.to_string_lossy();
        if let Some(context) = context {
            let file = fs::File::open(&context.localFilePath).unwrap();
            let result = file.seek_read(buffer, offset as u64).unwrap();
            return Ok(result as u32);
        }
        println!("读取文件失败: {}", file_name);
        Err(OperationError::NtStatus(STATUS_INVALID_DEVICE_REQUEST))
    }

    fn write_file(&'b self, _file_name: &U16CStr, _offset: i64, _buffer: &[u8], _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<u32, OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn flush_file_buffers(&'b self, _file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED))
    }

    // 获取文件信息
    fn get_file_information(&'b self, file_name: &U16CStr, _info: &OperationInfo<'a, 'b, Self>, context: &'a Self::Context) -> Result<FileInfo, OperationError> {
        let file_name = file_name.to_string_lossy();
        // 判断目录
        if file_name == *"\\" {
            return Ok(FileInfo { attributes: FILE_ATTRIBUTES_DIRECTORY, creation_time: UNIX_EPOCH, last_access_time: UNIX_EPOCH, last_write_time: UNIX_EPOCH, file_size: 0, number_of_links: 0, file_index: 0 });
        }

        if let Some(context) = context {
            return Ok(FileInfo {
                attributes: if context.FileInfo.is_dir { FILE_ATTRIBUTES_DIRECTORY } else { FILE_ATTRIBUTES_NORMAL },
                creation_time: StringToSystemTime(&*context.FileInfo.Modified),
                last_access_time: StringToSystemTime(&*context.FileInfo.Modified),
                last_write_time: StringToSystemTime(&*context.FileInfo.Modified),
                file_size: context.FileInfo.Size,
                number_of_links: 0,
                file_index: 0,
            });
        }
        Err(OperationError::NtStatus(STATUS_NDIS_FILE_NOT_FOUND))
    }

    // 列出目录中的所有子项
    fn find_files(&'b self, file_name: &U16CStr, mut fill_find_data: impl FnMut(&FindData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        let path = file_name.to_string_lossy();
        // 列出压缩包内文件结构
        for item in self.archiveFileInfoList.iter() {
            let filePath = format!(r"\{}", item.Path);
            // 筛选出匹配的文件(前面匹配、不等于自身、父路径匹配)
            if filePath.find(&path) == Some(0) && filePath != path && Path::new(&filePath).parent().unwrap().to_str().unwrap() == path {
                let fileName = Path::new(&item.Path).file_name().unwrap().to_str().unwrap();
                fill_find_data(&FindData {
                    attributes: if item.is_dir { FILE_ATTRIBUTES_DIRECTORY } else { FILE_ATTRIBUTES_NORMAL },
                    creation_time: StringToSystemTime(&*item.Modified),
                    last_access_time: StringToSystemTime(&*item.Modified),
                    last_write_time: StringToSystemTime(&*item.Modified),
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
            serial_number: 0,
            max_component_length: 255,
            fs_flags: FILE_CASE_PRESERVED_NAMES | FILE_UNICODE_ON_DISK | FILE_VOLUME_IS_COMPRESSED | FILE_READ_ONLY_VOLUME | FILE_PERSISTENT_ACLS,
            fs_name: convert_str("NTFS"),
        })
    }

    fn mounted(&'b self, _info: &OperationInfo<'a, 'b, Self>) -> Result<(), OperationError> {
        Ok(())
    }

    fn unmounted(&'b self, _info: &OperationInfo<'a, 'b, Self>) -> Result<(), OperationError> {
        Ok(())
    }

    // 获取可执行文件安全信息
    fn get_file_security(&'b self, _file_name: &U16CStr, _security_information: u32, _security_descriptor: *mut c_void, _buffer_length: u32, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<u32, OperationError> {
        Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED))
        // let file_name = file_name.to_string_lossy();
        // for item in self.archiveFileInfoList.iter() {
        //     let file_name = file_name.trim_start_matches(format!("\\{}\\", self.parentName).as_str());
        //     let localFilePath = &self.extractPath.join(file_name);
        //     if file_name == item.Path {
        //         let fileName: Vec<u16> = OsStr::new(localFilePath).encode_wide().chain(once(0)).collect();
        //         let mut needLength = buffer_length.clone();
        //         unsafe {
        //             winapi::um::securitybaseapi::GetFileSecurityW(fileName.as_ptr(), security_information, security_descriptor, buffer_length, &mut needLength);
        //         }
        //         return Ok(needLength);
        //     }
        // }
        // println!("获取安全信息失败: {}", file_name);
        // return Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED));
    }

    fn set_file_security(&'b self, _file_name: &U16CStr, _security_information: u32, _security_descriptor: *mut c_void, _buffer_length: u32, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_ACCESS_DENIED))
    }

    fn find_streams(&'b self, _file_name: &U16CStr, _fill_find_stream_data: impl FnMut(&FindStreamData) -> Result<(), FillDataError>, _info: &OperationInfo<'a, 'b, Self>, _context: &'a Self::Context) -> Result<(), OperationError> {
        Err(OperationError::NtStatus(STATUS_NOT_IMPLEMENTED))
    }
}
