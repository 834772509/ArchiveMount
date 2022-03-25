use crate::sevenzip::sevenZip;
use std::path::Path;

#[test]
fn test_listArchiveFiles() {
    let zip = sevenZip::new().unwrap();
    println!("{:?}", zip.listArchiveFiles(Path::new(r"C:\Users\Administrator\Desktop\壁纸.zip")));
}
