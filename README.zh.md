# ArchiveMount

[简体中文](README.zh.md) [English](README.md)

## 介绍

`ArchiveMount` 是用于挂载压缩包读取的工具。

### `ArchiveMount`有什么用？

`ArchiveMount`可以将压缩包格式挂载，直接在资源管理器内以文件夹形式访问压缩包。

### `ArchiveMount`的应用场景有哪些？

- 日常使用: 更直观；
- 单文件程序方案: 加快打开速度、性能要求更低；
- 便携软件包: 适用于可移动存储上快速体验软件；
- 便携游戏包: 适用于可移动存储上快速体验游戏；

### 为什么不用压缩软件？

- 集成度高，无感解压。

### 哪种格式更适合挂载？

我们推荐使用[zstd](http://www.zstd.net)算法进行压缩。Zstd 可以以压缩速度为代价提供更强的压缩比，速度与压缩的权衡可以通过小增量进行配置。使得解压缩速度在所有设置下都保持不变。

- [7-zip ZS] (https://github.com/mcmilk/7-Zip-zstd)

## 软件架构

使用`Rust`编写，调用`Dokan`库实现文件过滤，`VC-LTL`编译。

### `ArchiveMount`的原理是什么？

`ArchiveMount`采用`Dokan`实现用户模式文件系统，当用户读取文件时，`ArchiveMount`**自动将此文件解压到临时目录**供读取。  
注意: `ArchiveMount`**仅解压所需的文件**，用户或程序未读取的文件不会进行解压。

- 为什么需要解压到临时目录？  
  zip格式支持Stream流式读取，而7z格式不支持Stream流式读取。为了更通用故直接解压出来进行读取。

### `ArchiveMount`支持那些格式？

`ArchiveMount`内置 7-zip ZS，支持 7-zip ZS 所支持的所有格式。

- 7z、XZ、BZIP2、GZIP、TAR、ZIP、WIM、ESD
-

AR、ARJ、CAB、CHM、CPIO、CramFS、DMG、EXT、FAT、GPT、HFS、IHEX、ISO、LZH、LZMA、MBR、MSI、NSIS、NTFS、QCOW2、RAR、RPM、SquashFS、UDF、UEFI、VDI、VHD、VMDK、WIM、XAR、Z

### `ArchiveMount`的缓存机制是什么？

`ArchiveMount`采用LRU算法，即最近最少使用。当缓存即将满时，自动删除最近最少使用的文件。

- 这意味着缓存大小可以小于压缩包大小（但不建议设置过小，过小的缓存将极大的增加CPU开销）。

## 使用说明

本程序为命令行程序，故需要在其后面接参数运行，如直接双击程序将会出现“闪退”现象，您可通过`cmd`、`PowerShell`等终端来运行。  
注意：请使用**管理员身份**运行终端。

### 隐藏运行

`ArchiveMount.exe -q 命令 参数`

- `ArchiveMount.exe -q install`
- `ArchiveMount.exe -q mount 压缩包路径 挂载路径 [缓存路径]`

### 安装驱动

> 温馨提示: 如之前安装过Dokan驱动需要先卸载。

**使用`ArchiveMount`前需要安装驱动，否则会提示驱动没有安装**

- 基本安装: `ArchiveMount.exe install`
- 基本安装并注册到右键菜单: `ArchiveMount.exe install -r`

### 挂载压缩包

`ArchiveMount.exe mount 压缩包路径 挂载路径 [缓存路径]`

> 温馨提示: 如路径中含有空格请使用引号进行包裹。

- 基本使用
    - `ArchiveMount.exe mount D:\Archive.7z Z:`
    - `ArchiveMount.exe mount D:\Archive.7z D:\Mount`
    - `ArchiveMount.exe mount D:\Archive.7z D:\Mount D:\Cache`
- 挂载后打开: `ArchiveMount.exe mount 压缩包路径 挂载路径 -o`
    - `ArchiveMount.exe mount D:\Archive.7z Z: -o`
- 不嵌套目录挂载: `ArchiveMount.exe mount 压缩包路径 挂载路径 -n`
    - `ArchiveMount.exe mount D:\Archive.7z Z: -n`
- 指定密码: `ArchiveMount.exe mount 压缩包路径 挂载路径 -p密码`
    - `ArchiveMount.exe mount D:\Archive.7z Z: -p123456`
- 指定线程数(默认自动): `ArchiveMount.exe mount 压缩包路径 挂载路径 -t 线程数`
    - `ArchiveMount.exe mount D:\Archive.7z Z: -t 8`
- 指定缓存大小(默认4096MB): `ArchiveMount.exe mount 压缩包路径 挂载路径 -c 缓存大小`
    - `ArchiveMount.exe mount D:\Archive.7z Z: -c 1024`
- 指定卷标(默认ArchiveMount):`ArchiveMount.exe mount 压缩包路径 挂载路径 -v 卷标名`
    - `ArchiveMount.exe mount D:\Archive.7z Z: -v ArchiveFS`
- 开启调试模式: `ArchiveMount.exe mount D:\Archive.7z Z: -d`

### 卸载压缩包

`ArchiveMount.exe unmount 挂载路径`

- `ArchiveMount.exe unmount Z:`
- `ArchiveMount.exe unmount D:\Mount`

### 卸载驱动

> 温馨提示: 卸载驱动后需要重启才能完全卸载。

- `ArchiveMount uninstall`

## 开源许可

`ArchiveMount` 使用 GPL V3.0 协议开源，请尽量遵守开源协议。

## 致谢

- Cno

## 参与贡献

1. Fork 本仓库
2. 新建 Feat_xxx 分支
3. 提交代码
4. 新建 Pull Request
