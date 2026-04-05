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
cargo build --release
```

## Usage

```sh
cargo run -- <file>
```

## License

[GPLv3](LICENSE)
