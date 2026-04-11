# mutti

A terminal music player built with Rust.

## Features

- Audio playback with play/pause, seek, and volume control
- TUI interface built with [ratatui](https://github.com/ratatui/ratatui)
- Library management with SQLite-backed persistent track storage
- Reads metadata (title, artist, album, cover art) via [lofty](https://github.com/Serial-ATA/lofty-rs)
- Album art display in the terminal
- Optional FFT-based audio visualizer (`--visualize`)
- Supports formats provided by [Symphonia](https://github.com/pdeljanov/Symphonia) (MP3, FLAC, WAV, AAC, OGG, etc.)

## Keybindings

| Key | Action |
|-----|--------|
| `Space` | Play / Pause |
| `h` / `Left` | Seek backward 5s |
| `k` / `Right` | Seek forward 5s |
| `+` / `=` | Volume up |
| `-` | Volume down |
| `>` / `.` | Next song |
| `<` / `,` | Previous song |
| `Up` / `Down` | Navigate library |
| `Enter` | Play selected track |
| `Tab` | Next panel |
| `Shift+Tab` | Previous panel |
| `q` | Quit |

## Building

```sh
git clone --depth=1 https://github.com/kevkoa2106/mutti
cargo build --release
```

## Installation

### Prebuilt binaries

You can install it from the [releases](https://github.com/kevkoa2106/mutti/releases/tag/v0.1.0) page and unzip the tar file.

### Homebrew

```sh
brew tap kevkoa2106/tap
brew update
brew install mutti
```

## Dependencies

| Name | Purpose |
|-----|--------|
| ratatui | Display terminal UI |
| ratatui-image | Display images in ratatui |
| crossterm | Terminal event handling |
| clap | Parse command-line arguments |
| lofty | Get music files' metadata |
| image | Decode images |
| rusqlite | Read/write SQLite |
| walkdir | Recursively get all files in directory |
| rodio | Play/decode music file |
| rustfft | FFT algorithm for audio visualizer |

## Usage

```sh
mutti [file_or_directory] [--visualize]
```

## License

[GPLv3](LICENSE)
