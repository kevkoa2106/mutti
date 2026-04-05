use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, poll};
use mutti::audio_player::AudioPlayer;
use mutti::ui::{self, AppState, Panel, PlaybackInfo, RepeatMode};

fn main() {
    let mut player = AudioPlayer::new("test_song.mp3");

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
            queue: vec![],
            spectrum: vec![],
            focused_panel,
        };

        terminal.draw(|frame| ui::draw(frame, &state)).unwrap();

        if player.is_finished() {
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
                    KeyCode::Tab => focused_panel = focused_panel.next(),
                    KeyCode::BackTab => focused_panel = focused_panel.prev(),
                    _ => {}
                }
            }
        }
    }

    ratatui::restore();
}
