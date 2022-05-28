use std::{env, fs};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use dokan::MountError;

use crate::{ArchiveFS, TEMP_PATH};
use crate::sevenZip::sevenZip;
use crate::utils::console::{ConsoleType, writeConsole};
use crate::utils::util::createVirtualDrive;

#[derive(Parser, Debug)]
#[clap(version)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Mount compressed package
    mount {
        /// Archive path
        archivePath: PathBuf,
        /// Mount path
        mountPath: PathBuf,
        /// Temporary path(default temporary directory)
        tempPath: Option<PathBuf>,
        /// Archive password
        #[clap(short, long)]
        password: Option<String>,
        /// Threads(0: auto)
        #[clap(short, long)]
        #[clap(default_value_t = 0_u16)]
        threadCount: u16,
        /// Cache size(unit: MB)
        #[clap(short, long)]
        #[clap(default_value_t = 4096)]
        cacheSize: u64,
        /// read only mount
        #[clap(short, long)]
        readOnly: bool,
        /// nest mount points
        #[clap(short, long)]
        nest: bool,
        /// After mount open explorer
        #[clap(short, long)]
        open: bool,
        /// Mount volume name
        #[clap(short, long)]
        #[clap(default_value_t = String::from("ArchiveMount"))]
        volumeName: String,
        /// Debug mode
        #[clap(short, long)]
        debug: bool,
    },
    /// Unmount compressed package
    unmount {
        /// mount path
        mountPath: PathBuf,
        // commit to Archive
        // #[clap(short, long)]
        // commit: bool,
    },
}


pub fn cli() {
    let cli: Cli = Cli::parse();
    // 处理CLI
    match &cli.command {
        // 需要实现所有的子命令
        Commands::mount { archivePath, mountPath, tempPath, password, threadCount, cacheSize, readOnly, nest, open, volumeName, debug } => {
            writeConsole(ConsoleType::Info, &*format!("Mounting archive: {}", archivePath.to_str().unwrap()));
            if dokan::driver_version() == 0 {
                writeConsole(ConsoleType::Err, "driver not installed, Please make sure you install the Dokan driver");
                return;
            }
            // 处理相对目录
            let mut archivePath = archivePath.clone();
            if archivePath.is_relative() {
                archivePath = env::current_dir().unwrap().join(archivePath);
            }
            if !archivePath.exists() {
                writeConsole(ConsoleType::Err, "The archive does not exist, if the path contains spaces please use quotation marks.");
                return;
            }

            // 处理挂载路径
            let mut mountPath = mountPath.clone();
            if mountPath.is_dir() {
                if mountPath.metadata().unwrap().len() != 0 {
                    // 挂载路径为目录则需 1.目录存在 2.不能在挂载前打开 3.目录为空目录
                    writeConsole(ConsoleType::Err, "The mount path is not empty, please specify an empty directory");
                    return;
                }
                // 尝试创建目录
                fs::create_dir_all(&mountPath).ok();
            }

            if *nest {
                mountPath = if mountPath.is_dir() {
                    mountPath.join(archivePath.file_name().unwrap())
                } else {
                    let mountParent = TEMP_PATH.join("MountPoint");
                    let mountPint = mountParent.join(archivePath.file_name().unwrap());
                    if fs::create_dir_all(&mountPint).is_err() || !createVirtualDrive(&*mountParent, &*mountPath) {
                        writeConsole(ConsoleType::Err, "Create virtual mount point failed, please try to unnested mount point");
                        return;
                    };
                    mountPint
                    // mountPath.join(archivePath.file_name().unwrap())
                };
            }

            // 处理缓存目录
            let extractPath = if let Some(tempPath) = tempPath { tempPath.clone() } else { TEMP_PATH.join("ArchiveTemp") }.join(&archivePath.file_name().unwrap());
            let password = password.as_ref().map(|password| password.as_str());

            // 读取压缩包信息
            writeConsole(ConsoleType::Info, "Reading archive list......");
            let archiveFileInfoList = sevenZip::new().unwrap().listArchiveFiles(&*archivePath, password).unwrap();
            if archiveFileInfoList.is_empty() {
                writeConsole(ConsoleType::Err, "The Archive information is not detected, please confirm it is the correct archive or encrypted archive");
                return;
            }

            // 开始挂载
            let archiveFS = ArchiveFS::ArchiveFS::new(&*archivePath, &extractPath, password, *cacheSize, *readOnly, archiveFileInfoList, *open, volumeName, *debug);
            let result = archiveFS.mount(&*mountPath, *threadCount);

            // 处理错误信息
            if let Err(err) = result {
                match err {
                    MountError::Error => { writeConsole(ConsoleType::Err, "An error occurred"); }
                    MountError::DriveLetterError => { writeConsole(ConsoleType::Err, "Drive letter error"); }
                    MountError::DriverInstallError => { writeConsole(ConsoleType::Err, "Can't install the Dokan driver"); }
                    MountError::StartError => { writeConsole(ConsoleType::Err, "The driver responds that something is wrong"); }
                    MountError::MountError => { writeConsole(ConsoleType::Err, "Can't assign a drive letter or mount point"); }
                    MountError::MountPointError => { writeConsole(ConsoleType::Err, "The mount point is invalid"); }
                    MountError::VersionError => { writeConsole(ConsoleType::Err, "The Dokan version that this wrapper is targeting is incompatible with the loaded Dokan library"); }
                }
            }
        }
        Commands::unmount { mountPath } => {
            if !ArchiveFS::ArchiveFS::unmount(mountPath) {
                writeConsole(ConsoleType::Err, "unmount failed");
                return;
            }
            writeConsole(ConsoleType::Success, "unmount successfully");
        }
    }
}
