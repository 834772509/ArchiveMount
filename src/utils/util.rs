use std::{env, fs};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

use anyhow::Result;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

use crate::Asset;
use crate::TEMP_PATH;

/// 写到文件
pub fn writeEmbedFile(filePath: &str, outFilePath: &Path) -> Result<()> {
    let file = Asset::get(filePath).unwrap();
    File::create(outFilePath).unwrap().write_all(&file.data)?;
    Ok(())
}

/// 安装Dokan驱动
/// https://docs.microsoft.com/en-us/windows/win32/api/setupapi/nf-setupapi-setupcopyoeminfw
pub fn installDokanDriver() -> Result<bool> {
    // 1. dokan1.sys 释放至C:\windows\system32\drivers\dokan1.sys
    let dokanSysPath = Path::new(&env::var("windir")?).join(r"System32\drivers\dokan1.sys");
    if !dokanSysPath.exists() {
        writeEmbedFile("dokan1.sys", &dokanSysPath)?;
    }
    // 2. 释放 dokanctl.exe、dokan1.dll，执行 dokanctl.exe /i d
    let dokanctl = &TEMP_PATH.join("dokanctl.exe");
    if !dokanctl.exists() {
        // writeEmbedFile("dokan1.dll", &TEMP_PATH.join("dokan1.dll"))?;
        writeEmbedFile("dokanctl.exe", dokanctl)?;
    }
    let output = Command::new(dokanctl).creation_flags(0x08000000)
        .arg("/i")
        .arg("d")
        .output()?;
    let content = String::from_utf8_lossy(&output.stdout);
    Ok(content.contains("succeeded"))
}

/// 卸载Dokan驱动
pub fn uninstallDokanDriver() -> Result<bool> {
    let dokanctl = &TEMP_PATH.join("dokanctl.exe");
    if !dokanctl.exists() {
        // writeEmbedFile("dokan1.dll", &TEMP_PATH.join("dokan1.dll"))?;
        writeEmbedFile("dokanctl.exe", dokanctl)?;
    }
    let output = Command::new(&dokanctl).creation_flags(0x08000000)
        .arg("/r")
        .arg("d")
        .output()?;
    fs::remove_file(&Path::new(&env::var("windir")?).join(r"System32\drivers\dokan1.sys"))?;
    let content = String::from_utf8_lossy(&output.stdout);
    Ok(content.contains("removed"))
}

/// 注册右键菜单
pub fn registerFileMenu(programPath: &Path) -> Result<()> {
    // Reg.exe add "HKLM\SOFTWARE\Classes\*\shell\ArchiveMount" /ve /t REG_SZ /d "ArchiveMount" /f
    // Reg.exe add "HKLM\SOFTWARE\Classes\*\shell\ArchiveMount\command" /ve /t REG_SZ /d "\"ArchiveMount.exe\" mount \"%%1\" \"C:\ArchiveMount\" -o" /f
    let mountPath = env::var("SystemDrive").unwrap();
    let _ = Command::new("Reg.exe").creation_flags(0x08000000)
        .arg("add").arg(r"HKLM\SOFTWARE\Classes\*\shell\ArchiveMount")
        .arg("/ve")
        .arg("/t").arg("REG_SZ")
        .arg("/d").arg("ArchiveMount")
        .arg("/f")
        .output()?;
    let _ = Command::new("Reg.exe").creation_flags(0x08000000)
        .arg("add").arg(r"HKLM\SOFTWARE\Classes\*\shell\ArchiveMount\command")
        .arg("/ve")
        .arg("/t").arg("REG_SZ")
        .arg("/d").arg(format!("\"{}\" --quiet mount \"%1\" \"{}\\ArchiveMount\" -o", programPath.to_str().unwrap(), mountPath))
        .arg("/f")
        .output()?;
    Ok(())
}

/// 取消注册右键菜单
pub fn unregisterFileMenu() -> Result<()> {
    //reg delete HKLM\SOFTWARE\Classes\*\shell\ArchiveMount /f
    let _ = Command::new("Reg.exe").creation_flags(0x08000000)
        .arg("delete").arg(r"HKLM\SOFTWARE\Classes\*\shell\ArchiveMount")
        .arg("/f")
        .output()?;
    Ok(())
}

/// 设置DLL初始引用位置
pub fn setDllDirectory(path: &Path) -> bool {
    let path = OsStr::new(path).encode_wide().chain(once(0)).collect::<Vec<u16>>();
    let result;
    unsafe {
        result = winapi::um::winbase::SetDllDirectoryW(path.as_ptr());
    }
    result != 0
}

/// 字符串转时间
pub fn StringToSystemTime(time: &str) -> Result<SystemTime> {
    // println!("{}", time);
    let custom = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S")?;
    let date_time: DateTime<Local> = Local.from_local_datetime(&custom).unwrap();
    Ok(SystemTime::from(date_time))
}

// 增加字符串自定义方法
pub trait String_utils {
    fn get_string_left(&self, right: &str) -> Result<String, Box<dyn Error>>;
    fn get_string_center(&self, start: &str, end: &str) -> Result<String, Box<dyn Error>>;
    fn get_string_right(&self, left: &str) -> Result<String, Box<dyn Error>>;
}

impl String_utils for str {
    /// 取出字符串左边文本
    fn get_string_left(&self, right: &str) -> Result<String, Box<dyn Error>> {
        let endSize = self
            .find(right)
            .ok_or_else(|| "发生错误-查找结束位置失败".to_owned())?;
        Ok((&self[..endSize]).to_string())
    }

    /// 取出字符串中间文本
    fn get_string_center(&self, start: &str, end: &str) -> Result<String, Box<dyn Error>> {
        let startSize = self
            .find(start)
            .ok_or_else(|| "发生错误-查找起始位置失败".to_owned())?;
        let endSize = startSize
            + self[startSize..]
            .find(end)
            .ok_or_else(|| "发生错误-查找结束位置失败".to_owned())?;
        Ok((&self[startSize + start.len()..endSize]).to_string())
    }

    /// 取出字符串右边文本
    fn get_string_right(&self, left: &str) -> Result<String, Box<dyn Error>> {
        let startSize = self
            .find(left)
            .ok_or_else(|| "发生错误-查找左边位置失败".to_owned())?;
        Ok((&self[startSize + left.len()..]).to_string())
    }
}

impl String_utils for String {
    /// 取出字符串左边文本
    fn get_string_left(&self, right: &str) -> Result<String, Box<dyn Error>> {
        let endSize = self
            .find(right)
            .ok_or_else(|| "发生错误-查找结束位置失败".to_owned())?;
        Ok((&self[..endSize]).to_string())
    }

    /// 取出字符串中间文本
    fn get_string_center(&self, start: &str, end: &str) -> Result<String, Box<dyn Error>> {
        let startSize = self
            .find(start)
            .ok_or_else(|| "发生错误-查找起始位置失败".to_owned())?;
        let endSize = startSize
            + self[startSize..]
            .find(end)
            .ok_or_else(|| "发生错误-查找结束位置失败".to_owned())?;
        Ok((&self[startSize + start.len()..endSize]).to_string())
    }

    /// 取出字符串右边文本
    fn get_string_right(&self, left: &str) -> Result<String, Box<dyn Error>> {
        let startSize = self
            .find(left)
            .ok_or_else(|| "发生错误-查找左边位置失败".to_owned())?;
        Ok((&self[startSize + left.len()..]).to_string())
    }
}
