use std::path::{Path};
use std::error::Error;
use std::fs::File;
use widestring::U16CString;
use std::io::Write;
use crate::Asset;
use crate::TEMP_PATH;
use std::{env, fs};
use std::process::Command;
use std::time::SystemTime;
use chrono::{NaiveDateTime, DateTime, Local, TimeZone};

/// 写到文件
pub fn writeEmbedFile(filePath: &str, outFilePath: &Path) -> Result<(), Box<dyn Error>> {
    let file = Asset::get(filePath).unwrap();
    File::create(outFilePath).unwrap().write_all(&file.data)?;
    Ok(())
}

/// 安装Dokan驱动
pub fn installDokanDriver() -> Result<bool, Box<dyn Error>> {
    // 1. dokan1.sys释放至C:\windows\system32\drivers\dokan1.sys
    let _ = writeEmbedFile("dokan1.sys", &Path::new(&env::var("windir")?).join(r"System32\drivers\dokan1.sys"));

    // 2. 释放 dokanctl.exe、dokan1.dll，执行 dokanctl.exe /i d
    let dokanctl = &TEMP_PATH.join("dokanctl.exe");
    if !dokanctl.exists() {
        writeEmbedFile("dokan1.dll", &TEMP_PATH.join("dokan1.dll"))?;
        writeEmbedFile("dokanctl.exe", &dokanctl)?;
    }
    let output = Command::new(&dokanctl)
        .arg("/i")
        .arg("d")
        .output()?;
    let content = String::from_utf8_lossy(&output.stdout);
    Ok(content.contains("succeeded"))
}

/// 卸载Dokan驱动
pub fn uninstallDokanDriver() -> Result<bool, Box<dyn Error>> {
    let dokanctl = &TEMP_PATH.join("dokanctl.exe");
    if !dokanctl.exists() {
        writeEmbedFile("dokan1.dll", &TEMP_PATH.join("dokan1.dll"))?;
        writeEmbedFile("dokanctl.exe", &dokanctl)?;
    }
    let output = Command::new(&dokanctl)
        .arg("/r")
        .arg("d")
        .output()?;
    let _ = fs::remove_file(&Path::new(&env::var("windir")?).join(r"System32\drivers\dokan1.sys"));
    let content = String::from_utf8_lossy(&output.stdout);
    Ok(content.contains("removed"))
}

pub fn convert_str(s: impl AsRef<str>) -> U16CString {
    unsafe { U16CString::from_str_unchecked(s) }
}

pub fn StringToSystemTime(time: &str) -> SystemTime {
    let custom = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S").unwrap();
    let date_time: DateTime<Local> = Local.from_local_datetime(&custom).unwrap();
    return SystemTime::from(date_time);
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
