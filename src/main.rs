// 禁用变量命名警告
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
// 禁用未使用代码警告
#![allow(dead_code)]

mod utils;
mod ArchiveFS;
mod sevenzip;

#[cfg(test)]
mod tests;

extern crate dokan;
extern crate widestring;

use std::time::Duration;
use std::{env, fs};
use std::path::{PathBuf};
use rust_embed::RustEmbed;
use utils::util::{convert_str, installDokanDriver};
use dokan::{Drive, unmount, MountFlags, driver_version, set_lib_debug_mode, set_driver_debug_mode};

// 配置内置资源

// x64平台
#[cfg(target_arch = "x86_64")]
#[derive(RustEmbed)]
#[folder = "./assets-x64"]
pub struct Asset;

// x86平台
#[cfg(target_arch = "x86")]
#[derive(RustEmbed)]
#[folder = "./assets-x86"]
pub struct Asset;

// ARM平台
#[cfg(target_arch = "arm")]
#[derive(RustEmbed)]
#[folder = "./assets-ARM64"]
pub struct Asset;

#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate console;
extern crate winapi;

// 设置静态变量
lazy_static! {
    pub static ref TEMP_PATH: PathBuf = PathBuf::from(env::var("temp").unwrap()).join("ArchiveMount");
}

fn main() {
    if driver_version() == 0 {
        println!("安装驱动: {:?}", installDokanDriver());
    }
    println!("驱动版本：{}", driver_version());

    set_lib_debug_mode(true);
    println!("{}", set_driver_debug_mode(true));

    // 压缩包路径
    let archivePath = PathBuf::from(r".\test\test.7z");
    // 挂载路径(如为目录则需目录存在 且 不能在挂载前打开 且 目录需为空目录)
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
        .thread_count(1)
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
        .mount(&ArchiveFS::ArchiveFS::new(&archivePath, extractPath, parentName));
    println!("{:?}", result);
    // 清除临时目录
    if TEMP_PATH.exists() { let _ = fs::remove_dir_all(&*TEMP_PATH); }
}
