# WinAVFS
A high-performance Windows virtual filesystem for mounting archives

## What is WinAVFS
WinAVFS is a user-mode Windows filesystem implementation based on Dokan driver. It mounts an archive file (supports all formats supported by 7-Zip) as a read-only volume. To achieve high overall performance, WinAVFS uses unmanaged memory as cached buffer and extracts a file only when it is being read.

## Dependencies
- [Dokany](https://github.com/dokan-dev/dokany) file system driver [LGPL License]
- [DokanNet](https://github.com/dokan-dev/dokan-dotnet) [MIT License]
- [SevenZipSharp](https://github.com/squid-box/SevenZipSharp) [LGPL License]
- 7z.dll from [7-Zip](https://www.7-zip.org) [LGPL + unRAR + BSD 3-clause License]

## Usage
To mount an archive, run the following command:
```
> WinAVFS.CLI.exe <path to archive> <mount point>
```
While the application is running, press Ctrl-C to unmount the volume.

### Mount an archive as a volume with drive letter
```
> WinAVFS.CLI.exe D:\test.zip Z:\
```

### Mount an archive in an NTFS mountpoint
```
> WinAVFS.CLI.exe D:\test.zip D:\mount
```

## License
MIT License

Copyright (c) 2020 DeepAQ

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
