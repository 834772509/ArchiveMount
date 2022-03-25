# ArchiveMount

[简体中文](README.zh.md) [English](README.md)

## introduce

`ArchiveMount` is a tool for mounting tarballs for reading.

### What is the use of `ArchiveMount`?

`ArchiveMount` can mount the compressed package format and directly access the compressed package as a folder in the resource manager.

### What are the application scenarios of `ArchiveMount`?

- Daily use (more intuitive)
- Single file program solution (faster opening, lower performance requirements)
- Portable software package (suitable for quick experience software on removable storage)
- Portable game pack (suitable for quick game experience on removable storage)

### What is the principle of `ArchiveMount`?

`ArchiveMount` uses `Dokan` to implement a user-mode file system. When a user reads a file, `ArchiveMount` automatically extracts the file to a temporary directory for reading.
`ArchiveMount` will only extract the required files, files that are not read by the user or program will not be extracted.

- Why do you need to unzip to a temporary directory?
    The zip format supports Stream streaming reading, while the 7z format does not support Stream streaming reading. In order to be more general, it is directly decompressed and read.

### Why not use compression software?

- High integration, no sense of decompression.

### What formats does `ArchiveMount` support?

`ArchiveMount` has a built-in 7-zip program that supports all formats supported by 7-zip.

- 7z, XZ, BZIP2, GZIP, TAR, ZIP, WIM
- AR, ARJ, CAB, CHM, CPIO, CramFS, DMG, EXT, FAT, GPT, HFS, IHEX, ISO, LZH, LZMA, MBR, MSI, NSIS, NTFS, QCOW2, RAR, RPM, SquashFS, UDF, UEFI , VDI, VHD, VMDK, WIM, XAR, Z

## Software Architecture

Written in `Rust`, calling the `Dokan` library to implement file filtering, and compiled with `VC-LTL`.

## Instructions for use

This program is a command line program, so it needs to be run with parameters after it. If you double-click the program directly, there will be a "flashback" phenomenon. You can run it through `cmd`, `PowerShell` and other terminals.
Note: Please run the terminal as **administrator**.

## Open Source License

`ArchiveMount` is open source using the GPL V3.0 license, please try to abide by the open source license.

## Thanks

## Participate in contribution

1. Fork this repository
2. Create a new Feat_xxx branch
3. Submit the code
4. Create a new Pull Request
