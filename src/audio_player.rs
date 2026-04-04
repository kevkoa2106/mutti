use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use std::io::Cursor;
use std::time::{Duration, Instant};

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

    pub fn new(file_path: &str) -> Self {
        let sink = DeviceSinkBuilder::open_default_sink().unwrap();
        let player = Player::connect_new(sink.mixer());

        let file_data = std::fs::read(file_path).unwrap();
        let source = Self::make_decoder(&file_data);
        let total_duration = source.total_duration().unwrap_or_default();

        player.append(source);

        Self {
            player,
            _sink: sink,
            file_data,
            started_at: Instant::now(),
            paused_elapsed: Duration::ZERO,
            is_paused: false,
            volume: 100,
            total_duration,
            title: file_path.to_string(),
            artist: "Unknown".to_string(),
            album: "Unknown".to_string(),
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
