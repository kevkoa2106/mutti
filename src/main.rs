use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, poll};
use mutti::args::Args;
use mutti::audio_player::AudioPlayer;
use mutti::ui::{self, AppState, Panel, PlaybackInfo, QueueItem, RepeatMode};

fn main() {
    let args = Args::parse();

    let mut player = AudioPlayer::new(&args.audio_file);

    let mut terminal = ratatui::init();
    let tick_rate = Duration::from_millis(16);
    let mut focused_panel = Panel::NowPlaying;

    loop {
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
            spectrum: vec![],
            focused_panel,
        };

        terminal.draw(|frame| ui::draw(frame, &state)).unwrap();

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
