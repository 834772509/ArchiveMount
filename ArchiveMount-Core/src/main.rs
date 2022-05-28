// 禁用变量命名警告
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
// 禁用未使用代码警告
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

use std::{env, fs, process};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use rust_embed::RustEmbed;

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
    pub static ref TEMP_PATH: PathBuf = env::temp_dir().join("ArchiveMount");
}

fn main() {
    // 程序退出调用
    let running = Arc::new(AtomicUsize::new(0));
    ctrlc::set_handler(move || {
        let prev = running.clone().fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            fs::remove_dir_all(&*TEMP_PATH).ok();
            process::exit(0x0100);
        }
    }).expect("Error setting Ctrl-C handler");

    // 创建临时目录
    let _ = fs::create_dir(&*TEMP_PATH);

    // 处理CLI
    cli::cli();

    // 删除临时目录
    let _ = fs::remove_dir_all(&*TEMP_PATH);
}
