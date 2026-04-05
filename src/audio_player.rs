use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::Accessor;
use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use walkdir::WalkDir;

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub total_duration: Duration,
}

pub struct AudioPlayer {
    player: Player,
    _sink: rodio::MixerDeviceSink,
    file_data: Vec<u8>,
    started_at: Instant,
    paused_elapsed: Duration,
    pub is_paused: bool,
    pub volume: u8,
    pub total_duration: Duration,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub playlist: Vec<PathBuf>,
    pub current_index: usize,
}

fn scan_audio_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_file()
            && path.extension().is_some_and(|ext| {
                matches!(
                    ext.to_str().map(|s| s.to_lowercase()).as_deref(),
                    Some("mp3" | "flac" | "wav" | "ogg" | "aac")
                )
            })
        {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    files
}

fn read_tags(path: &Path) -> (String, String, String) {
    let tagged = Probe::open(path).and_then(|p| p.read());
    let tag = tagged
        .as_ref()
        .ok()
        .and_then(|t| t.primary_tag().or(t.first_tag()));

    let title = tag
        .and_then(|t| t.title().map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string())
        });
    let artist = tag
        .and_then(|t| t.artist().map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown".to_string());
    let album = tag
        .and_then(|t| t.album().map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown".to_string());

    (title, artist, album)
}

impl AudioPlayer {
    fn make_decoder(data: &[u8]) -> Decoder<Cursor<Vec<u8>>> {
        let len = data.len() as u64;
        Decoder::builder()
            .with_data(Cursor::new(data.to_vec()))
            .with_byte_len(len)
            .with_seekable(true)
            .build()
            .unwrap()
    }

    pub fn new(path: &str) -> Self {
        let sink = DeviceSinkBuilder::open_default_sink().unwrap();
        let player = Player::connect_new(sink.mixer());

        let p = Path::new(path);
        let playlist = if p.is_dir() {
            scan_audio_files(p)
        } else {
            vec![p.to_path_buf()]
        };

        assert!(!playlist.is_empty(), "No audio files found in {path}");

        let file_data = std::fs::read(&playlist[0]).unwrap();
        let source = Self::make_decoder(&file_data);
        let total_duration = source.total_duration().unwrap_or_default();
        player.append(source);

        let (title, artist, album) = read_tags(&playlist[0]);

        Self {
            player,
            _sink: sink,
            file_data,
            started_at: Instant::now(),
            paused_elapsed: Duration::ZERO,
            is_paused: false,
            volume: 100,
            total_duration,
            title,
            artist,
            album,
            playlist,
            current_index: 0,
        }
    }

    fn load_track(&mut self, index: usize) {
        self.current_index = index;
        let path = &self.playlist[index];

        self.file_data = std::fs::read(path).unwrap();
        let source = Self::make_decoder(&self.file_data);
        self.total_duration = source.total_duration().unwrap_or_default();

        let (title, artist, album) = read_tags(path);
        self.title = title;
        self.artist = artist;
        self.album = album;

        self.player.skip_one();
        self.player.append(source);
        self.player.set_volume(self.volume as f32 / 100.0);
        if !self.is_paused {
            self.player.play();
        }

        self.paused_elapsed = Duration::ZERO;
        self.started_at = Instant::now();
    }

    pub fn next_track(&mut self) {
        if self.current_index + 1 < self.playlist.len() {
            self.load_track(self.current_index + 1);
        }
    }

    pub fn prev_track(&mut self) {
        if self.current_index > 0 {
            self.load_track(self.current_index - 1);
        }
    }

    pub fn elapsed(&self) -> Duration {
        if self.is_paused {
            self.paused_elapsed
        } else {
            self.paused_elapsed + self.started_at.elapsed()
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.is_paused {
            self.player.play();
            self.started_at = Instant::now();
            self.is_paused = false;
        } else {
            self.paused_elapsed += self.started_at.elapsed();
            self.player.pause();
            self.is_paused = true;
        }
    }

    pub fn set_volume(&mut self, vol: u8) {
        self.volume = vol;
        self.player.set_volume(vol as f32 / 100.0);
    }

    pub fn is_finished(&self) -> bool {
        !self.total_duration.is_zero() && self.elapsed() >= self.total_duration
    }

    /// Auto-advance to next track if current one finished.
    /// Returns true if playback is completely done (no more tracks).
    pub fn check_advance(&mut self) -> bool {
        if self.is_finished() {
            if self.current_index + 1 < self.playlist.len() {
                self.next_track();
                false
            } else {
                true // playlist ended
            }
        } else {
            false
        }
    }

    pub fn playlist_titles(&self) -> Vec<String> {
        self.playlist
            .iter()
            .map(|p| {
                p.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.to_string_lossy().to_string())
            })
            .collect()
    }

    pub fn seek_to(&mut self, target: Duration) {
        let target = target.min(self.total_duration);

        self.player.skip_one();
        let mut source = Self::make_decoder(&self.file_data);
        if !target.is_zero() {
            if let Err(e) = source.try_seek(target) {
                eprintln!("seek error: {e:?}");
            }
        }
        self.player.append(source);
        self.player.set_volume(self.volume as f32 / 100.0);
        if !self.is_paused {
            self.player.play();
        }

        self.paused_elapsed = target;
        if !self.is_paused {
            self.started_at = Instant::now();
        }
    }

    pub fn seek_backward(&mut self, secs: u64) {
        let target = self.elapsed().saturating_sub(Duration::from_secs(secs));
        self.seek_to(target);
    }

    pub fn seek_forward(&mut self, secs: u64) {
        let target = self.elapsed() + Duration::from_secs(secs);
        self.seek_to(target);
    }
}
