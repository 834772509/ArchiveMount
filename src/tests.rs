// 禁用变量命名警告
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
// 禁用未使用代码警告
#![allow(dead_code)]

use std::path::Path;
use std::time::Duration;
use std::path::{PathBuf};
use crate::sevenzip::sevenZip;
use crate::utils::util::{StringToSystemTime};
use crate::utils::util::convert_str;
use crate::TEMP_PATH;
use crate::ArchiveFS;
use dokan::{Drive, unmount, MountFlags, driver_version};

#[test]
fn test_listArchiveFiles() {
    let zip = sevenZip::new().unwrap();
    println!("{:#?}", zip.listArchiveFiles(Path::new(r"./test/test.7z"), None));
}

#[test]
fn test_ArchiveFileTime() {
    let zip = sevenZip::new().unwrap();
    let list = zip.listArchiveFiles(Path::new(r"./test/test.7z"), None).unwrap();
    println!("{:?}", StringToSystemTime(&list[0].Modified));
}

#[test]
fn test_Mount_ArchiveFile() {
    // set_lib_debug_mode(true);

    // 压缩包路径
    // let archivePath = PathBuf::from(r".\test\test.7z");
    let archivePath = PathBuf::from(r"D:\Project\FirPE\EFI\PETOOLS\PETOOLS.7z");
    // 挂载路径(如为目录则需 1.目录存在 2.不能在挂载前打开 3.目录为空目录)
    let moutPoint = &convert_str(r"Z:");
    // 临时解压路径
    let extractPath = &*TEMP_PATH.join("ArchiveTemp").join(&archivePath.file_name().unwrap());
    // 挂载路径根文件名(默认 压缩包名.后缀 )
    let parentName = archivePath.file_name().unwrap().to_str().unwrap();

    // 防止上次未正确卸载
    let _ = unmount(&moutPoint);
    // 挂载
    let result = Drive::new()
        // 线程数(0为自动)
        .thread_count(0)
        // 文件系统模式
        // .flags(MountFlags::WRITE_PROTECT | MountFlags::MOUNT_MANAGER | MountFlags::DEBUG | MountFlags::STDERR)
        .flags(MountFlags::WRITE_PROTECT | MountFlags::MOUNT_MANAGER)
        // 挂载路径
        .mount_point(&moutPoint)
        // 超时时间
        .timeout(Duration::from_secs(5))
        // 分配单元大小
        .allocation_unit_size(1024)
        // 扇区大小
        .sector_size(1024)
        // 挂载并阻塞当前线程，直到卷被卸载
        .mount(&ArchiveFS::ArchiveFS::new(&archivePath, extractPath, None));
    println!("{:?}", result);
}
