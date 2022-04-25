use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

use crate::TEMP_PATH;
use crate::utils::util::{String_utils, writeEmbedFile};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ArchiveFileInfo {
    /// 文件路径
    pub(crate) Path: String,
    /// 文件大小
    pub(crate) Size: u64,
    /// 文件大小(压缩后)
    pub(crate) PackedSize: u64,
    /// 修改时间
    pub(crate) Modified: String,
    /// 创建时间(注意：7z格式无此属性)
    pub(crate) Created: Option<String>,
    /// 是否为目录
    pub(crate) is_dir: bool,
    // 文件属性
    // pub(crate) Attributes: String,
    // 是否加密
    // pub(crate) Encrypted: bool,
    // CRC校验码
    // pub(crate) CRC: String,
    // 压缩算法
    // pub(crate) Method: String,
}

#[derive(Debug)]
pub struct sevenZip {
    zipProgram: PathBuf,
}

impl sevenZip {
    pub fn new() -> Result<sevenZip> {
        if !TEMP_PATH.exists() {
            fs::create_dir(&*TEMP_PATH)?;
        }
        let zipProgram = TEMP_PATH.join("7z.exe");
        writeEmbedFile("7z.exe", &zipProgram)?;
        writeEmbedFile("7z.dll", &TEMP_PATH.join("7z.dll"))?;
        Ok(sevenZip { zipProgram })
    }

    /// 列出压缩包文件列表
    pub fn listArchiveFiles(&self, zipFile: &Path, password: Option<&str>) -> Result<Vec<ArchiveFileInfo>> {
        let output = Command::new(&self.zipProgram).creation_flags(0x08000000)
            .arg("l")
            .arg(format!("-p{}", password.unwrap_or("")))
            // 跳过标题行
            .arg("-ba")
            // 格式化技术信息
            .arg("-slt")
            .arg("-sccUTF-8")
            .arg(zipFile.to_str().unwrap())
            .output()?;
        let content = String::from_utf8_lossy(&output.stdout);

        let arr = content.split("\r\n\r\n");

        let mut archiveFileInfoList: Vec<ArchiveFileInfo> = Vec::new();
        for item in arr {
            if item.trim().is_empty() { continue; }

            let packedSize = item.get_string_center("Packed Size = ", "\r\n").unwrap_or_else(|_| 0.to_string());
            let packedSize = if packedSize.is_empty() { 0 } else { packedSize.parse().unwrap() };

            let created = item.get_string_center("Created = ", "\r\n").unwrap_or_else(|_| "".to_string());
            let created = if created.is_empty() { None } else { Some(created) };
            archiveFileInfoList.push(ArchiveFileInfo {
                Path: item.get_string_center("Path = ", "\r\n").unwrap_or_else(|_| "".to_string()),
                Size: item.get_string_center("Size = ", "\r\n").unwrap_or_else(|_| "0".to_string()).parse().unwrap_or(0),
                PackedSize: packedSize,
                Modified: item.get_string_center("Modified = ", "\r\n").unwrap_or_else(|_| "".to_string()),
                Created: created,
                is_dir: item.get_string_center("Attributes = ", "\r\n").unwrap_or_else(|_| "".to_string()).contains('D'),
                // Attributes: item.get_string_center("Attributes = ", "\r\n").unwrap_or_else(|_| "".to_string()),
                // Encrypted: false,
                // CRC: item.get_string_center("CRC = ", "\r\n").unwrap_or_else(|_| "".to_string()),
                // Method: item.get_string_center("Method = ", "\r\n").unwrap_or_else(|_| "".to_string()),
            });
        }
        Ok(archiveFileInfoList)
    }

    /// 7-zip 解压文件
    /// 提取具有完整路径的文件（保留文件路径）
    /// # 参数
    /// 1. 压缩包路径
    /// 2. 解压路径
    /// 3. 输出路径
    pub fn extractFilesFromPath(
        &self,
        zipFile: &Path,
        password: Option<&str>,
        extractPath: &str,
        outPath: &Path,
    ) -> Result<bool> {
        let output = Command::new(&self.zipProgram).creation_flags(0x08000000)
            .arg("x")
            .arg(zipFile.to_str().unwrap())
            .arg(if !extractPath.is_empty() {
                extractPath
            } else {
                "*"
            })
            .arg("-y")
            .arg("-aos")
            .arg(format!("-p{}", password.unwrap_or("")))
            .arg(format!("-o{}", outPath.to_str().unwrap()))
            .output()?;
        let outContent = String::from_utf8_lossy(&output.stdout);
        Ok(outContent.contains("Everything is Ok"))
    }

    /// 7-zip 解压文件
    /// 提取具有完整路径的文件（递归子目录）
    /// # 参数
    /// 1. 压缩包路径
    /// 2. 解压路径
    /// 3. 输出路径
    pub fn extractFilesFromPathRecurseSubdirectories(
        &self,
        zipFile: &Path,
        extractPath: &str,
        outPath: &Path,
    ) -> Result<bool> {
        let output = Command::new(&self.zipProgram).creation_flags(0x08000000)
            .arg("x")
            .arg("-r")
            .arg(zipFile.to_str().unwrap())
            .arg(&extractPath)
            .arg("-y")
            .arg("-aos")
            .arg(format!("-o{}", outPath.to_str().unwrap()))
            .output()?;
        let content = String::from_utf8_lossy(&output.stdout);
        Ok(!content.contains("No files to process"))
    }
}
