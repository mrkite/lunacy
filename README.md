# Lunacy

An inspector for retro platforms.  Explore the contents of old disk images.  Written
in Rust.

![Screenshot of Amiga Disk Contents](/screenshots/amiga.png)

Opening a supported disk format will show a tree view of the disk's contents.  You can
then select individual files and open them in new tabs.

![Screenshot of Basic listing](/screenshots/basic.png)

Open a supported BASIC program and it will show the source, which can then be saved
as a text file.  Special characters will be translated into their unicode
equivalents.

![Screenshot of ILBM view](/screenshots/ilbm.png)

Open a supported image format and it will show the image, with a slider that allows
for up to 10x zoom.  You can then save the image as a PNG.

![Screenshot of Hex dump](/screenshots/hex.png)

At any point, you can switch the view to a hex view of the original data.  This will
also be the default view if the file type is unknown.

## Supported File Types

- Acorn
  - Advanced Disc Filing System (ADFS)
  - Unified Emulator Format (UEF)
  - BASIC
- Amiga
  - Amiga Disk File (ADF)
  - Interleaved Bitmap (ILBM)
- Amstrad CPC
  - CPM filesystem (DSK)
- Apple II
  - DOS 3.3 filesystem
  - ProDOS filesytem
  - Super HiRes and Preferred Format Pictures
  - Applesoft BASIC
  - Integer BASIC
- Atari ST
  - ATR disk images
  - BASIC
- Commodore 64
  - D64 disk images
  - BASIC
- DOS (PC)
  - MBR disk images
  - FAT16
- Macintosh
  - MFS file system
  - HFS file system
  - Resource Forks
  - ICON/ICN# Resources
- Tandy TRS80
  - JVC file system
  - BASIC
- Vic-20
  - BASIC
- ZX-Spectrum
  - TAP tape format
  - TZX tape format
  - BASIC

## Expandable

Adding support for new formats and filesystems is simple.  It uses a "plugin-style"
structure (unfortunately not actual plugins since Rust doesn't have a standard ABI),
and provides several widgets to make showing file trees or images simple.

## Compile on unix
```sh
cargo build --release
cargo install --path . --root /usr/local 
```
