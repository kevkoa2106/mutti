# mutti

A terminal music player built with Rust.

## Features

- Audio playback with play/pause, seek, and volume control
- TUI interface built with [ratatui](https://github.com/ratatui/ratatui)
- Reads metadata (title, artist, album) via [lofty](https://github.com/Serial-ATA/lofty-rs)
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

## Usage

```sh
cargo run -- <file>
```

## License

[GPLv3](LICENSE)
