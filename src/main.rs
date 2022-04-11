// 禁用变量命名警告
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
// 禁用未使用代码警告
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

use std::{env, fs};
use std::path::PathBuf;

use rust_embed::RustEmbed;

use utils::util::{setDllDirectory, writeEmbedFile};

mod ArchiveFS;
mod sevenZip;
mod cli;
mod utils;

#[cfg(test)]
mod tests;

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

// 设置静态变量
lazy_static! {
    pub static ref TEMP_PATH: PathBuf = PathBuf::from(env::var("temp").unwrap()).join("ArchiveMount");
}

fn main() {
    // 创建临时目录
    let _ = fs::remove_dir_all(&*TEMP_PATH);
    let _ = fs::create_dir(&*TEMP_PATH);
    // 配置程序运行环境
    let dokanDll = &*TEMP_PATH.join("dokan1.dll");
    if !dokanDll.exists() { let _ = writeEmbedFile("dokan1.dll", dokanDll); }
    setDllDirectory(&*TEMP_PATH);
    // 处理CLI
    cli::cli();
    // 删除临时目录
    let _ = fs::remove_dir_all(&*TEMP_PATH);
}
