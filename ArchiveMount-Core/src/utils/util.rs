use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr::null;
use std::time::SystemTime;

use anyhow::Result;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

use crate::Asset;

/// 写到文件
pub fn writeEmbedFile(filePath: &str, outFilePath: &Path) -> Result<()> {
    let file = Asset::get(filePath).unwrap();
    File::create(outFilePath).unwrap().write_all(&file.data)?;
    Ok(())
}

/// 创建虚拟盘符
/// 参数1: 目标路径
/// 参数2: 挂载盘符
pub fn createVirtualDrive(targetPath: &Path, mountPath: &Path) -> bool {
    let path = OsStr::new(&format!(r"\??\{}", targetPath.to_str().unwrap())).encode_wide().chain(once(0)).collect::<Vec<u16>>();
    let mount_point = mountPath.as_os_str().encode_wide().chain(once(0)).collect::<Vec<u16>>();
    let result = unsafe { winapi::um::fileapi::DefineDosDeviceW(1, mount_point.as_ptr(), path.as_ptr()) };
    result != 0
}

/// 卸载虚拟盘符
/// 参数1: 挂载盘符
pub fn removeVirtualDrive(mountPath: &Path) -> bool {
    let mountPath = mountPath.as_os_str().encode_wide().chain(once(0)).collect::<Vec<u16>>();
    let result = unsafe { winapi::um::fileapi::DefineDosDeviceW(2, mountPath.as_ptr(), null()) };
    result != 0
}

/// 字符串转时间
pub fn StringToSystemTime(time: &str) -> Result<SystemTime> {
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
