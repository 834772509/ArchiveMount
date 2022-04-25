use std::{env, fs};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use dokan::MountError;

use crate::{ArchiveFS, QUIET, TEMP_PATH};
use crate::sevenZip::sevenZip;
use crate::utils::console::{ConsoleType, writeConsole};
use crate::utils::util::{installDokanDriver, registerFileMenu, uninstallDokanDriver, unregisterFileMenu};

#[derive(Parser, Debug)]
#[clap(version)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    /// hide console
    #[clap(short, long)]
    quiet: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// install the Archive Mount driver
    install {
        /// Register file menu
        #[clap(short, long)]
        register: bool,
    },
    /// uninstall the Archive Mount driver
    uninstall {},
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
        /// Do not nest mount points
        #[clap(short, long)]
        notNest: bool,
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
    },
}


pub fn cli() {
    let cli: Cli = Cli::parse();
    // 创建控制台
    if !cli.quiet {
        unsafe {
            winapi::um::consoleapi::AllocConsole();
            winapi::um::consoleapi::SetConsoleCtrlHandler(None, 0);
        }
    }
    *QUIET.lock().unwrap() = cli.quiet;

    // 处理CLI
    match &cli.command {
        // 需要实现所有的子命令
        Commands::install { register } => {
            if dokan::driver_version() > 0 {
                writeConsole(ConsoleType::Info, "driver installed");
                return;
            }
            writeConsole(ConsoleType::Info, "install driver");
            let result = installDokanDriver();
            if result.is_err() || !result.unwrap() {
                writeConsole(ConsoleType::Err, "Driver installation failed");
                return;
            }
            // 注册右键菜单
            if *register {
                let programPath = Path::new(&env::var("windir").unwrap()).join(r"System32").join(env::current_exe().unwrap().file_name().unwrap());
                if fs::copy(env::current_exe().unwrap(), &programPath).is_err() || registerFileMenu(&programPath).is_err() {
                    writeConsole(ConsoleType::Warning, "Registration process failed");
                }
                writeConsole(ConsoleType::Info, "Registration process successfully");
            }
            writeConsole(ConsoleType::Success, "Driver installed successfully");
        }
        Commands::uninstall {} => {
            if dokan::driver_version() == 0 {
                writeConsole(ConsoleType::Err, "driver not installed");
                return;
            }
            writeConsole(ConsoleType::Info, "uninstall driver");
            let result = uninstallDokanDriver();
            if result.is_err() || !result.unwrap() {
                writeConsole(ConsoleType::Err, "Driver uninstall failed");
                return;
            }
            let _ = unregisterFileMenu();
            writeConsole(ConsoleType::Success, "Driver uninstall successfully");
        }
        Commands::mount { archivePath, mountPath, tempPath, password, threadCount, cacheSize, notNest, open, volumeName, debug } => {
            if !cli.quiet {
                writeConsole(ConsoleType::Info, &*format!("Mounting archive: {}", archivePath.to_str().unwrap()));
            }
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
                writeConsole(ConsoleType::Err, "The archive does not exist");
                return;
            }
            // 处理挂载路径
            let mut mountPath = mountPath.clone();
            if mountPath.is_dir() && mountPath.metadata().unwrap().len() != 0 {
                // 挂载路径为目录则需 1.目录存在 2.不能在挂载前打开 3.目录为空目录
                writeConsole(ConsoleType::Err, "The mount path is not empty, please specify an empty directory");
                return;
            } else {
                // 如果挂载点为目录尝试创建目录
                let _ = fs::create_dir_all(&mountPath);
                if *notNest {
                    mountPath = mountPath.join(archivePath.file_name().unwrap());
                }
            }
            // let parentName = match *notNest {
            //     true => None,
            //     false => Some(archivePath.file_name().unwrap().to_str().unwrap().to_string())
            // };
            // 处理缓存目录
            let extractPath = if let Some(tempPath) = tempPath { tempPath.clone() } else { TEMP_PATH.join("ArchiveTemp") }.join(&archivePath.file_name().unwrap());
            let password = password.as_ref().map(|password| password.as_str());
            // 读取压缩包信息
            if !cli.quiet {
                writeConsole(ConsoleType::Info, "Reading archive list......");
            }
            let archiveFileInfoList = sevenZip::new().unwrap().listArchiveFiles(&*archivePath, password).unwrap();
            if archiveFileInfoList.is_empty() {
                writeConsole(ConsoleType::Err, "The Archive information is not detected, please confirm it is the correct archive or encrypted archive");
                return;
            }
            // 开始挂载
            let archiveFS = ArchiveFS::ArchiveFS::new(&*archivePath, &extractPath, password, *cacheSize, archiveFileInfoList, *open, volumeName, if cli.quiet { false } else { *debug });
            let result = archiveFS.mount(&*mountPath, *threadCount);
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
            if ArchiveFS::ArchiveFS::unmount(mountPath) {
                writeConsole(ConsoleType::Success, "unmount successfully");
                return;
            }
            writeConsole(ConsoleType::Err, "unmount failed");
        }
    }
}
