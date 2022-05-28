use clap::{Parser, Subcommand};
use std::{env, fs};
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;

use crate::{ARCHIVE_MOUNT_PATH, TEMP_PATH, writeEmbedFile};
use crate::utils::console::{ConsoleType, writeConsole};
use crate::utils::util::{installDokanDriver, isInstallDokan, registerFileMenu, String_utils, uninstallDokanDriver, unregisterFileMenu};

#[derive(Parser, Debug)]
#[clap(version)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

// 此处的 mount、unmount 命令仅提供帮助用途，实际参数将转发至 ArchiveMount.exe
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
    },
}


pub fn cli() {
    let cli: Cli = Cli::parse();

    // 配置程序运行环境
    let args: Vec<String> = env::args().skip(1).collect();
    if args.get(0).unwrap_or(&"".to_string()) == "mount" || args.get(0).unwrap_or(&"".to_string()) == "unmount" && !ARCHIVE_MOUNT_PATH.exists() {
        writeEmbedFile("dokan1.dll", &*TEMP_PATH.join("dokan1.dll")).ok();
        writeEmbedFile(ARCHIVE_MOUNT_PATH.file_name().unwrap().to_str().unwrap(), &**ARCHIVE_MOUNT_PATH).ok();
    }

    // 处理CLI
    match &cli.command {
        Commands::install { register } => {
            let result = isInstallDokan();
            if result.unwrap_or(false) == true {
                writeConsole(ConsoleType::Success, "driver installed");
                return;
            }

            writeConsole(ConsoleType::Info, "install driver");
            let result = installDokanDriver();
            if result.unwrap_or(false) == false {
                writeConsole(ConsoleType::Error, "Driver installation failed");
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
            let result = isInstallDokan();
            if result.unwrap_or(false) == false {
                writeConsole(ConsoleType::Error, "driver not installed");
                return;
            }
            writeConsole(ConsoleType::Info, "uninstall driver");
            let result = uninstallDokanDriver();
            if result.unwrap_or(false) == false {
                writeConsole(ConsoleType::Error, "Driver uninstall failed");
                return;
            }
            let _ = unregisterFileMenu();
            writeConsole(ConsoleType::Success, "Driver uninstall successfully");
        }
        Commands::mount { archivePath: _, mountPath: _, tempPath: _, password: _, threadCount: _, cacheSize: _, nest: _, open: _, volumeName: _, debug: _, readOnly: _ } => {
            let args: Vec<String> = env::args().skip(1).collect();
            let stdout = Command::new(&*ARCHIVE_MOUNT_PATH).creation_flags(0x08000000)
                .args(args)
                .stdout(Stdio::piped())
                .spawn().unwrap().stdout.unwrap();

            let reader = BufReader::new(stdout).lines().filter_map(|line| line.ok());
            for content in reader {
                let printType = ConsoleType::from_str(&*content.get_string_center("  ", "      ").unwrap().trim().to_string()).unwrap();
                let printMessage = content.get_string_right("      ").unwrap().trim().to_string();
                writeConsole(printType.clone(), &printMessage);
                if printType == ConsoleType::Success || printType == ConsoleType::Error {
                    return;
                }
            }
        }
        Commands::unmount { mountPath: _ } => {
            let args: Vec<String> = env::args().skip(1).collect();
            let output = Command::new(&*ARCHIVE_MOUNT_PATH).creation_flags(0x08000000).args(args).output().unwrap();
            let content = String::from_utf8_lossy(&output.stdout).to_string();
            let printType = content.get_string_center("  ", "      ").unwrap().trim().to_string();
            let printMessage = content.get_string_right("      ").unwrap().trim().to_string();
            writeConsole(ConsoleType::from_str(&printType).unwrap(), &printMessage);
        }
    }
}
