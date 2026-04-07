use std::io::Cursor;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, poll};
use mutti::args::Args;
use mutti::audio_player::AudioPlayer;
use mutti::ui::{self, AppState, Panel, PlaybackInfo, QueueItem, RepeatMode};
use mutti::visualizer::Visualizer;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;

fn main() {
    let args = Args::parse();
    let visualize = args.visualize;

    let mut player = AudioPlayer::new(&args.audio_file);

    let mut terminal = ratatui::init();
    let tick_rate = Duration::from_millis(16);
    let mut focused_panel = Panel::NowPlaying;

    let picker = Picker::halfblocks();
    let mut album_art: Option<StatefulProtocol> = make_album_art(&picker, &player.cover_art);
    let mut last_track_index = player.current_index;
    let mut visualizer = Visualizer::new();
    let term_width = terminal.size().map(|s| s.width).unwrap_or(80) as usize;
    let mut smoothed: Vec<u64> = vec![0; term_width];

    loop {
        // Refresh album art on track change
        if player.current_index != last_track_index {
            album_art = make_album_art(&picker, &player.cover_art);
            last_track_index = player.current_index;
        }
        let state = AppState {
            playback: Some(PlaybackInfo {
                title: player.title.clone(),
                artist: player.artist.clone(),
                album: player.album.clone(),
                elapsed: player.elapsed(),
                total: player.total_duration,
                is_paused: player.is_paused,
                volume: player.volume,
                shuffle: false,
                repeat: RepeatMode::Off,
            }),
            library: vec![],
            library_selected: 0,
            queue: player
                .playlist_titles()
                .into_iter()
                .enumerate()
                .map(|(i, title)| QueueItem {
                    title,
                    is_current: i == player.current_index,
                })
                .collect(),
            spectrum: if visualize {
                let num_bars = smoothed.len().max(16);
                let raw = {
                    let buf = player.sample_buffer.lock().unwrap();
                    if buf.sample_rate > 0 && !buf.samples.is_empty() {
                        visualizer.compute(
                            &buf.samples,
                            buf.sample_rate,
                            player.elapsed(),
                            num_bars,
                        )
                    } else {
                        vec![0; num_bars]
                    }
                };
                if smoothed.len() != raw.len() {
                    smoothed = vec![0; raw.len()];
                }
                for (s, r) in smoothed.iter_mut().zip(raw.iter()) {
                    // Instant attack so beats snap up; slow decay so they fall visibly.
                    if *r >= *s {
                        *s = *r;
                    } else {
                        *s = (*s as f32 * 0.85) as u64;
                    }
                }
                smoothed.clone()
            } else {
                vec![]
            },
            focused_panel,
            visualize,
        };

        terminal.draw(|frame| ui::draw(frame, &state, &mut album_art)).unwrap();

        if player.check_advance() {
            break;
        }

        if poll(tick_rate).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => player.toggle_pause(),
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        player.set_volume(player.volume.saturating_add(5).min(100));
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        player.seek_backward(5);
                    }
                    KeyCode::Right | KeyCode::Char('k') => {
                        player.seek_forward(5);
                    }
                    KeyCode::Char('-') => {
                        player.set_volume(player.volume.saturating_sub(5));
                    }
                    KeyCode::Char('.') | KeyCode::Char('>') => player.next_track(),
                    KeyCode::Char(',') | KeyCode::Char('<') => player.prev_track(),
                    KeyCode::Tab => focused_panel = focused_panel.next(),
                    KeyCode::BackTab => focused_panel = focused_panel.prev(),
                    _ => {}
                }
            }
        }
    }

    ratatui::restore();
}

fn make_album_art(picker: &Picker, cover_art: &Option<Vec<u8>>) -> Option<StatefulProtocol> {
    let data = cover_art.as_ref()?;
    let img = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    Some(picker.new_resize_protocol(img))
}
