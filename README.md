# converty

Automatically converts media files when their extension is renamed. Powered by inotify — no manual commands needed.

Rename `video.mp4` → `video.gif` and converty silently converts the content in the background.

## How it works

1. Watches your home directory recursively via inotify
2. Detects when a media file's extension changes
3. Converts the file content to match the new format using ffmpeg or ImageMagick
4. The original rename is transparent — the file just appears in the new format

## Requirements

- `ffmpeg`
- `imagemagick` (provides the `magick` command)

```bash
sudo pacman -S ffmpeg imagemagick
```

## Installation

```bash
git clone https://github.com/Oztturk/converty.git
cd converty
cargo build --release
sudo cp target/release/converty /usr/local/bin/
```

## Usage

```bash
converty [OPTIONS]
```

Simply run `converty` and start renaming files. Press `Ctrl+C` to stop.

### Options

| Flag | Description |
|------|-------------|
| `--save-origin` | Keep the original file after conversion |
| `--save-path <DIR>` | Save originals to a specific directory (implies `--save-origin`) |
| `--temp-path <DIR>` | Use a specific directory for temporary files during conversion |

### Examples

```bash
# Basic — originals are deleted after conversion
converty

# Keep originals in the same directory
converty --save-origin

# Keep originals in a dedicated folder
converty --save-path ~/originals

# Use /tmp for intermediate files (faster on tmpfs)
converty --temp-path /tmp/converty

# All options combined
converty --save-path ~/originals --temp-path /tmp/converty
```

## Supported formats

| Category | Formats | Tool |
|----------|---------|------|
| Photo | `jpg` `jpeg` `png` `webp` `bmp` `tiff` `ico` `avif` `heic` | ImageMagick |
| Video | `mp4` `avi` `mkv` `mov` `webm` `flv` `wmv` | ffmpeg |
| GIF | `gif` | ImageMagick / ffmpeg |

### Conversion rules

| From → To | Tool | Notes |
|-----------|------|-------|
| Photo → Photo | ImageMagick | |
| Photo → GIF | ImageMagick | Static GIF |
| GIF → Photo | ImageMagick | Falls back to ffmpeg if GIF contains video |
| Video → Video | ffmpeg | |
| Video → GIF | ffmpeg | Animated GIF |
| Video → Photo | ffmpeg | Extracts first frame |
| GIF → Video | ffmpeg | No audio |
| Photo → Video | ffmpeg | Single-frame video |

## Project structure

```
src/
├── main.rs              # Entry point, dependency checks
├── cli.rs               # CLI arguments (clap)
├── formats.rs           # Supported formats and categories
├── converter/
│   ├── mod.rs           # Conversion orchestration, temp file management
│   └── tools.rs         # ffmpeg and ImageMagick command wrappers
└── watcher/
    ├── mod.rs           # inotify watcher setup, directory traversal
    └── handler.rs       # Rename event dispatch (From/To/Both)
```

## Notes

- Symlinks are not followed to avoid watching restricted paths (e.g. Steam Proton's `/root` symlink)
- A 300ms delay is applied before conversion starts to let file managers finish their post-rename operations
- Temporary files are prefixed with `.converty_tmp_` and hidden when created in the same directory as the source file
- If conversion fails, the file is restored to its original name
