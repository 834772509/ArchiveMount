use crate::sevenzip::sevenZip;
use std::path::Path;
use winapi::um::{securitybaseapi};

#[test]
fn test_listArchiveFiles() {
    let zip = sevenZip::new().unwrap();
    println!("{:?}", zip.listArchiveFiles(Path::new(r"C:\Users\Administrator\Desktop\壁纸.zip")));
}

#[test]
unsafe fn test_get_file_security() {
    let file = r"C:\Users\Administrator\Desktop\notepad.exe";
    securitybaseapi::GetFileSecurityW(
        file.as_ptr(),
        winapi::um::winnt::OWNER_SECURITY_INFORMATION,
        aaa,
        bbb,
        ccc
    );
}
