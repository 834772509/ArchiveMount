// 禁用变量命名警告
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
// 禁用未使用代码警告
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::thread::Thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::Local;
use dokan::{Drive, driver_version, MountFlags, unmount};

use crate::ArchiveFS;
use crate::sevenZip::sevenZip;
use crate::TEMP_PATH;
use crate::utils::util::convert_str;
use crate::utils::util::StringToSystemTime;

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
fn test_timestamp1() {
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let ms = since_the_epoch.as_secs() as i64 * 1000i64 + (since_the_epoch.subsec_nanos() as f64 / 1_000_000.0) as i64;
    // println!("{}", ms);
    println!("{}", Local::now().timestamp_millis());
}

#[derive(Debug)]
struct testStruct {
    // testType: HashMap<i32, i32>,
    testType: Vec<(String, i32)>,
}

#[test]
fn test_struct_HashMap() {
    let mut testStruct = testStruct { testType: vec![("bbbb".to_string(), 2), ("aaaa".to_string(), 1)] };
    testStruct.testType.sort_by(|a, b| a.1.cmp(&b.1));
    println!("{:?}", testStruct.testType);
    // testStruct.testType.insert(0, 1);
    // testStruct.testType.insert(1, 1);
    // println!("{:#?}", testStruct);
}

#[test]
fn test_ArchiveFile() {
    let archivePath = PathBuf::from(r".\test\test.7z");
    let extractPath = TEMP_PATH.join("ArchiveTemp").join(&archivePath.file_name().unwrap());
    let zip = sevenZip::new().unwrap();
    let list = zip.listArchiveFiles(&*archivePath, None).unwrap();
    let fs = ArchiveFS::ArchiveFS::new(&archivePath.clone(), &*extractPath.clone(), None, list);
    // println!("{:#?}", fs);
}

#[test]
fn test_Mount_ArchiveFile() {
    // set_lib_debug_mode(true);

    // 压缩包路径
    let archivePath = PathBuf::from(r".\test\test.7z");
    // let archivePath = PathBuf::from(r"D:\Project\FirPE\EFI\PETOOLS\PETOOLS.7z");
    // 挂载路径(如为目录则需 1.目录存在 2.不能在挂载前打开 3.目录为空目录)
    let moutPoint = convert_str(r"Z:");
    // 临时解压路径
    let extractPath = TEMP_PATH.join("ArchiveTemp").join(&archivePath.file_name().unwrap());
    // 挂载路径根文件名(默认 压缩包名.后缀 )
    let parentName = archivePath.file_name().unwrap().to_str().unwrap();

    let zip = sevenZip::new().unwrap();
    let list = zip.listArchiveFiles(&*archivePath, None).unwrap();

    // 防止上次未正确卸载
    let _ = unmount(&moutPoint);
    // 挂载

    let myThread = thread::spawn(move || {
        Drive::new()
            // 线程数(0为自动)
            .thread_count(0)
            // 文件系统模式
            // .flags(MountFlags::WRITE_PROTECT | MountFlags::MOUNT_MANAGER | MountFlags::DEBUG | MountFlags::STDERR)
            .flags(MountFlags::WRITE_PROTECT | MountFlags::MOUNT_MANAGER)
            // 挂载路径
            .mount_point(&convert_str(r"Z:"))
            // 超时时间
            .timeout(Duration::from_secs(5))
            // 分配单元大小
            .allocation_unit_size(1024)
            // 扇区大小
            .sector_size(1024)
            // 挂载并阻塞当前线程，直到卷被卸载
            .mount(&ArchiveFS::ArchiveFS::new(&archivePath.clone(), &*extractPath.clone(), None, list));
    });
    // println!("挂载完毕");
    myThread.join();
}
