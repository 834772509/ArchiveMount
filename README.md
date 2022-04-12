# ArchiveMount

[简体中文](README.zh.md) [English](README.md)

## introduce

`ArchiveMount` is a tool for mounting tarballs for reading.

### What is the use of `ArchiveMount`?

`ArchiveMount` can mount the compressed package format and directly access the compressed package as a folder in the
resource manager.

### What are the application scenarios of `ArchiveMount`?

- Daily use: more intuitive;
- Single-file program solution: faster opening and lower performance requirements;
- Portable software package: suitable for quick experience software on removable storage;
- Portable game pack: suitable for fast game experience on removable storage;

### Why not use compression software?

- High integration, no sense of decompression.

## Software Architecture

Written in `Rust`, calling the `Dokan` library to implement file filtering, and compiled with `VC-LTL`.

### What is the principle of `ArchiveMount`?

`ArchiveMount` uses `Dokan` to implement a user-mode file system. When a user reads a file, `ArchiveMount`**
automatically extracts the file to a temporary directory** for reading. Note: `ArchiveMount`**Only extracts the required
files**, files not read by the user or program will not be extracted.

- Why do you need to unzip to a temporary directory? The zip format supports Stream streaming reading, while the 7z
  format does not support Stream streaming reading. In order to be more general, it is directly decompressed and read.

### What formats does `ArchiveMount` support?

`ArchiveMount` has 7-zip built in and supports all formats supported by 7-zip.

- 7z, XZ, BZIP2, GZIP, TAR, ZIP, WIM
-

AR, ARJ, CAB, CHM, CPIO, CramFS, DMG, EXT, FAT, GPT, HFS, IHEX, ISO, LZH, LZMA, MBR, MSI, NSIS, NTFS, QCOW2, RAR, RPM,
SquashFS, UDF, UEFI, VDI, VHD, VMDK, WIM, XAR, Z

### What is the caching mechanism of `ArchiveMount`?

`ArchiveMount` uses the LRU algorithm, i.e. Least Recently Used. When the cache is about to be full, the least recently
used files are automatically deleted.

- This means that the cache size can be smaller than the compressed package size (but it is not recommended to set too
  small, too small cache will greatly increase the CPU overhead).

## Instructions for use

This program is a command line program, so it needs to be run with parameters after it. If you double-click the program
directly, there will be a "flashback" phenomenon. You can run it through `cmd`, `PowerShell` and other terminals. Note:
Please run the terminal as **administrator**.

### install driver

Drivers need to be installed before using `ArchiveMount`:

- `ArchiveMount.exe install`

### Mount the compressed package

`ArchiveMount.exe mout ArchivePath MountPath [CachePath]`

- basic use
    - `ArchiveMount.exe mount D:\Archive.7z D:\Mount`
    - `ArchiveMount.exe mount D:\Archive.7z D:\Mount D:\Cache`
- Specify password: `ArchiveMount.exe mount ArchivePath MountPath -p password`
    - `ArchiveMount.exe mount ArchivePath MountPath -p 123456`
- Specify the number of threads: `ArchiveMount.exe mount ArchivePath MountPath -t threadNumber`
    - `ArchiveMount.exe mount ArchivePath MountPath path -t8`
- Specify cache size: `ArchiveMount.exe mount ArchivePath MountPath -c CacheSize`
    - `ArchiveMount.exe mount ArchivePath MountPath path -c1024`
- Specify the cache directory: `ArchiveMount.exe mount ArchivePath MountPath CacheDirectory`
    - `ArchiveMount.exe mount ArchivePath MountPath D:\Cache`

### Uninstall the driver

- `ArchiveMount uninstall`

## Open Source License

`ArchiveMount` is open source using the GPL V3.0 license, please try to abide by the open source license.

## Thanks

- Cno

## Participate in contribution

1. Fork this repository
2. Create a new Feat_xxx branch
3. Submit the code
4. Create a new Pull Request
