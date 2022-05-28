@echo off
cd /d %~dp0

cargo build --release

rem Add UPX
IF EXIST upx.exe upx "%cd%\target\release\ArchiveMount-CLI.exe" --best --compress-resources=0 --strip-relocs=0 --compress-icons=0 --compress-exports=0 --lzma

start "" "%cd%\target\release"
