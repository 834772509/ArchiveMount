# ArchiveMount

[简体中文](README.zh.md) [English](README.md)

## 介绍

`ArchiveMount` 是用于挂载压缩包读取的工具。

### `ArchiveMount`有什么用？

`ArchiveMount`可以将压缩包格式挂载，直接在资源管理器内以文件夹形式访问压缩包。

### `ArchiveMount`的应用场景有哪些？

- 日常使用: 更直观
- 单文件程序方案: 加快打开速度、性能要求更低
- 便携软件包: 适用于可移动存储上快速体验软件
- 便携游戏包: 适用于可移动存储上快速体验游戏

### `ArchiveMount`的原理是什么？

`ArchiveMount`采用`Dokan`实现用户模式文件系统，当用户读取文件时，`ArchiveMount`自动将此文件**自动解压到临时目录**供读取。  
`ArchiveMount`仅会解压所需的文件，用户或程序不读取的文件不会进行解压。

- 为什么需要解压到临时目录？  
    zip格式支持Stream流式读取，而7z格式不支持Stream流式读取。为了更通用故直接解压出来读取。

### 为什么不用压缩软件？

- 集成度高，无感解压。

### `ArchiveMount`支持那些格式？

`ArchiveMount`内置7-zip程序，支持7-zip所支持的所有格式。

- 7z、XZ、BZIP2、GZIP、TAR、ZIP、WIM
- AR、ARJ、CAB、CHM、CPIO、CramFS、DMG、EXT、FAT、GPT、HFS、IHEX、ISO、LZH、LZMA、MBR、MSI、NSIS、NTFS、QCOW2、RAR、RPM、SquashFS、UDF、UEFI、VDI、VHD、VMDK、WIM、XAR、Z

## 软件架构

使用`Rust`编写，调用`Dokan`库实现文件过滤，`VC-LTL`编译。

## 使用说明

本程序为命令行程序，故需要在其后面接参数运行，如直接双击程序将会出现“闪退”现象，您可通过`cmd`、`PowerShell`等终端来运行。  
注意：请使用**管理员身份**运行终端。

### 挂载压缩包

`ArchiveMount.exe mount 压缩包路径`

## 开源许可

`ArchiveMount` 使用 GPL V3.0 协议开源，请尽量遵守开源协议。

## 致谢

## 参与贡献

1.  Fork 本仓库
2.  新建 Feat_xxx 分支
3.  提交代码
4.  新建 Pull Request
