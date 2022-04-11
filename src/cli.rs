use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use dokan::MountFlags;

use crate::{ArchiveFS, TEMP_PATH};
use crate::sevenZip::sevenZip;
use crate::utils::console::{ConsoleType, writeConsole};
use crate::utils::util::{convert_str, installDokanDriver, uninstallDokanDriver};

#[derive(Parser)]
#[clap(version)]
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
        /// temporary file path(default temporary directory)
        tempPath: Option<PathBuf>,
        /// compressed package password
        #[clap(short, long)]
        password: Option<String>,
        /// Threads(default Automatic)
        #[clap(short, long)]
        threadCount: Option<u16>,
        /// cache size(default 4096 MB)
        #[clap(short, long)]
        #[clap(default_value_t = 4096)]
        cacheSize: u16,
        /// Do not nest mount points
        #[clap(short, long)]
        notNest: bool,
        /// Open path after mount
        #[clap(short, long)]
        open: bool,
    },
    /// Unmount compressed package
    unmount {
        /// mount path
        mountPath: PathBuf,
    },
}

pub fn cli() {
    let cli = Cli::parse();
    match &cli.command {
        // 需要实现所有的子命令
        Commands::install {} => {
            if dokan::driver_version() > 0 {
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
            if dokan::driver_version() == 0 {
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
        Commands::mount { archivePath, mountPath, tempPath, password, threadCount, cacheSize, notNest, open } => {
            let extractPath = if let Some(tempPath) = tempPath { tempPath.clone() } else { TEMP_PATH.join("ArchiveTemp") }.join(&archivePath.file_name().unwrap());
            let password = password.as_ref().map(|password| password.as_str());
            if !archivePath.exists() {
                writeConsole(ConsoleType::Err, "package does not exist");
                return;
            }
            let archiveFileInfoList = sevenZip::new().unwrap().listArchiveFiles(archivePath, password).unwrap();
            if archiveFileInfoList.is_empty() {
                writeConsole(ConsoleType::Err, "The compressed package information is not detected, please confirm it is the correct compressed package or encrypted compressed package");
                return;
            }

            let mut mountPath = mountPath.clone();
            if mountPath.is_dir() && mountPath.metadata().unwrap().len() != 0 {
                // 挂载路径(如为目录则需 1.目录存在 2.不能在挂载前打开 3.目录为空目录)
                writeConsole(ConsoleType::Err, "The mount path is not empty, please specify an empty directory");
                return;
            } else if fs::create_dir_all(&mountPath).is_ok() && !*notNest {
                // 将挂载点重定向到 \挂载路径\压缩包名.7z\ 目录
                mountPath = mountPath.join(archivePath.file_name().unwrap());
                let _ = fs::create_dir_all(&mountPath);
            }
            writeConsole(ConsoleType::Info, &*format!("Mount compressed package: {}", archivePath.to_str().unwrap()));
            // 挂载
            let archiveFs = ArchiveFS::ArchiveFS::new(archivePath, &extractPath, password, *cacheSize, archiveFileInfoList);
            let _result = dokan::Drive::new()
                // 线程数(0为自动)
                .thread_count(threadCount.unwrap_or(0))
                // 文件系统模式
                .flags(MountFlags::WRITE_PROTECT | MountFlags::MOUNT_MANAGER)
                // 挂载路径
                .mount_point(&convert_str(mountPath.to_str().unwrap()))
                // 超时时间
                .timeout(Duration::from_secs(5))
                // 分配单元大小
                .allocation_unit_size(1024)
                // 扇区大小
                .sector_size(1024)
                // 挂载并阻塞当前线程，直到卷被卸载
                .mount(&archiveFs);
            // if *open { let _ = Command::new(mountPath).output().unwrap(); }
        }
        Commands::unmount { mountPath } => {
            if dokan::unmount(&convert_str(mountPath.to_str().unwrap())) {
                writeConsole(ConsoleType::Success, "unmount successfully");
                return;
            }
            writeConsole(ConsoleType::Err, "unmount failed");
        }
    }
}
