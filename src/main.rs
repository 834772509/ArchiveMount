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
use clap::{Parser, Subcommand};
use rust_embed::RustEmbed;
use dokan::{Drive, unmount, MountFlags, driver_version};
use utils::util::{convert_str, installDokanDriver, uninstallDokanDriver};
use utils::console::{writeConsole, ConsoleType};

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
extern crate clap;

// 设置静态变量
lazy_static! {
    pub static ref TEMP_PATH: PathBuf = PathBuf::from(env::var("temp").unwrap()).join("ArchiveMount");
}


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// install the Archive Mount driver
    install {},
    /// uninstall the Archive Mount driver
    uninstall {},
    /// Mount compressed package
    mount {
        /// compressed package path
        archivePath: PathBuf,
        /// mount path
        mountPath: PathBuf,
        /// temporary file path
        tempPath: Option<PathBuf>,
        /// compressed package password
        #[clap(short, long)]
        password: Option<String>,
        /// Threads(Automatic by default)
        #[clap(short, long)]
        threadCount: Option<u16>,
    },
    /// Unmount compressed package
    unmount {
        /// mount path
        mountPath: PathBuf,
    },
    test {
        name: Option<PathBuf>
    },
}

fn main() {
    // 创建临时目录
    let _ = fs::create_dir(&*TEMP_PATH);
    let cli = Cli::parse();
    match &cli.command {
        // 需要实现所有的子命令
        Commands::install {} => {
            if driver_version() > 0 {
                writeConsole(ConsoleType::Info, "driver installed");
                return;
            }
            writeConsole(ConsoleType::Info, "install driver");
            if let Ok(true) = installDokanDriver() {
                writeConsole(ConsoleType::Success, "Driver installed successfully");
                return;
            }
            writeConsole(ConsoleType::Err, "Driver installation failed");
        }
        Commands::uninstall {} => {
            if driver_version() == 0 {
                writeConsole(ConsoleType::Err, "driver not installed");
                return;
            }
            writeConsole(ConsoleType::Info, "uninstall driver");
            if let Ok(true) = uninstallDokanDriver() {
                writeConsole(ConsoleType::Success, "Driver uninstall successfully");
                return;
            }
            writeConsole(ConsoleType::Err, "Driver uninstall failed");
        }
        Commands::mount { archivePath, mountPath, tempPath, password, threadCount } => {
            if !archivePath.exists() {
                writeConsole(ConsoleType::Err, "package does not exist");
                return;
            }
            writeConsole(ConsoleType::Info, &*format!("Mount compressed package: {}", archivePath.to_str().unwrap()));

            let mountPath = &convert_str(mountPath.to_str().unwrap());
            let extractPath = if let Some(tempPath) = tempPath { tempPath.clone() } else { TEMP_PATH.join("ArchiveTemp") };
            let extractPath = extractPath.join(&archivePath.file_name().unwrap());
            let password = if let Some(password) =  password { Some(password.as_str()) } else { None };

            let _result = Drive::new()
                // 线程数(0为自动)
                .thread_count(threadCount.unwrap_or(0))
                // 文件系统模式
                .flags(MountFlags::WRITE_PROTECT | MountFlags::MOUNT_MANAGER)
                // 挂载路径
                .mount_point(&mountPath)
                // 超时时间
                .timeout(Duration::from_secs(5))
                // 分配单元大小
                .allocation_unit_size(1024)
                // 扇区大小
                .sector_size(1024)
                // 挂载并阻塞当前线程，直到卷被卸载
                .mount(&ArchiveFS::ArchiveFS::new(archivePath, &extractPath, password));
        }
        Commands::unmount { mountPath } => {
            if unmount(&convert_str(mountPath.to_str().unwrap())) {
                writeConsole(ConsoleType::Success, "unmount successfully");
                return;
            }
            writeConsole(ConsoleType::Err, "unmount failed");
        }
        Commands::test { name } => {
            println!("{:?}", name);
        }
    }
    // 清除临时目录
    if TEMP_PATH.exists() { let _ = fs::remove_dir_all(&*TEMP_PATH); }
    let _ = fs::remove_file("./dokan1.dll");
    return;
}
