// 禁用变量命名警告
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
// 禁用未使用代码警告
#![allow(dead_code)]

use std::iter::zip;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::ArchiveFS::ArchiveFS;
use crate::sevenZip::{ArchiveFileInfo, sevenZip};
use crate::TEMP_PATH;

#[test]
fn test_listArchiveFiles() {
    let zip = sevenZip::new().unwrap();
    let list = zip.listArchiveFiles(Path::new(r"./test/test.7z"), None).unwrap();
    for item in list.iter() {
        println!("{}", item.Path);
    }
}

#[test]
fn test_mountArchive() {
    let zip = sevenZip::new().unwrap();
    let archivePath = PathBuf::from(r"./test/test.7z");
    let archiveFileInfoList = zip.listArchiveFiles(&archivePath, None).unwrap();
    let extractPath = TEMP_PATH.join("ArchiveTemp").join(&archivePath.file_name().unwrap());
    let archiveFS = ArchiveFS::new(&*archivePath, &extractPath, None, 1024, false, archiveFileInfoList, true, "ArchiveMount", true);
    archiveFS.mount("Z:".as_ref(), 0);
}

/// 文件结构
struct FileInfo {
    /// 文件路径
    pub(crate) Path: PathBuf,
    /// 文件大小
    pub(crate) Size: u64,
    /// 创建时间
    pub(crate) Created: SystemTime,
    /// 修改时间
    pub(crate) Modified: SystemTime,
    /// 是否为目录
    pub(crate) is_dir: bool,
}

// [
//   文件:[文件1信息, [] ],
//   目录:[文件1信息, [内目录[]] ],
// ]
/// 目录树结构
#[derive(Debug)]
struct FileTree {
    // name: String,
    // path: PathBuf,
    // is_dir: bool,

    info: ArchiveFileInfo,
    children: Option<Vec<FileTree>>,
}

#[test]
fn test_fileTree() {
    let zip = sevenZip::new().unwrap();
    let list = zip.listArchiveFiles(Path::new(r"./test/test.7z"), None).unwrap();
    // println!("{:#?}", list);

    let tree = getFileTree(&list);
    println!("===========================");
    println!("{:#?}", tree);
}

fn getFileTree(list: &Vec<ArchiveFileInfo>) -> Vec<FileTree> {
    let mut FileTree: Vec<FileTree> = Vec::new();

    for item in list.iter() {
        // if !item.Path.contains("嵌套目录测试") { continue; }
        // println!("=============={}=========", item.Path);
        if item.is_dir {
            let dirList: Vec<ArchiveFileInfo> = list.into_iter()
                .filter(|info| info.Path.starts_with(&item.Path) && Path::new(&info.Path).parent().unwrap().to_str().unwrap() == item.Path && info.Path != item.Path).cloned().collect();
            // println!("{:#?}", dirList);
            // let children = getFileTree(&dirList);
            // println!("{:?}", children);
            // FileTree.push(FileTree { info: item.clone(), children: Some(children) });
        } else if Path::new(&item.Path).parent().unwrap().to_str().unwrap() == "" {
            // 根目录
            FileTree.push(FileTree { info: item.clone(), children: None });
        }
    }
    return FileTree;
}
