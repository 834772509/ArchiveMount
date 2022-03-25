# Free mount

An open source software to mount .rar, .7z, .zip and more, works like rar2fs but for Windows

## Warning

This still needs more testing, but the software is currently able to mount a multipart .7z file and play video, so I decided to release it.

## Why use this instead of, lets say, extracting .rar file

I ran out of disk spaces... I need to handle 300GB+ multipart .rar file

## Used components/libraries in this software

7z.dll [license](https://www.7-zip.org/license.txt) - LGPL v2.1  
Dokany, releases includes user-mode library [license-mit](https://github.com/dokan-dev/dokany/blob/master/license.mit.txt) [license-lgpl](https://github.com/dokan-dev/dokany/blob/master/license.lgpl.txt) - Part of it is MIT. another part is LGPL v3  
SevenZipSharp (maintained by squid-box) [license-lgpl](https://github.com/squid-box/SevenZipSharp/blob/dev/license) - LGPL v3

### Honorable mention

This project is inspired by [Shaman.Dokan.Archive](https://github.com/antiufo/Shaman.Dokan.Archive), but it's not updated, at first I tried to update it myself but the library it used is also not maintained, so I made my own version.