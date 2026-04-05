use std::time::Duration;

use ratatui::prelude::*;
use ratatui::widgets::*;

/// Current state of playback for the UI to render.
pub struct PlaybackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub elapsed: Duration,
    pub total: Duration,
    pub is_paused: bool,
    pub volume: u8,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

pub struct LibraryItem {
    pub name: String,
    pub is_selected: bool,
}

pub struct QueueItem {
    pub title: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Library,
    NowPlaying,
    Visualizer,
    Queue,
}

impl Panel {
    pub fn next(self) -> Self {
        match self {
            Panel::Library => Panel::NowPlaying,
            Panel::NowPlaying => Panel::Visualizer,
            Panel::Visualizer => Panel::Queue,
            Panel::Queue => Panel::Library,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Panel::Library => Panel::Queue,
            Panel::NowPlaying => Panel::Library,
            Panel::Visualizer => Panel::NowPlaying,
            Panel::Queue => Panel::Visualizer,
        }
    }
}

pub struct AppState {
    pub playback: Option<PlaybackInfo>,
    pub library: Vec<LibraryItem>,
    pub library_selected: usize,
    pub queue: Vec<QueueItem>,
    pub spectrum: Vec<u64>,
    pub focused_panel: Panel,
}

pub fn draw(frame: &mut Frame, state: &AppState) {
    let outer = Layout::vertical([
        Constraint::Min(0),    // main panels
        Constraint::Length(3), // queue bar
    ])
    .split(frame.area());

    let panels = Layout::horizontal([
        Constraint::Percentage(25), // library browser
        Constraint::Percentage(40), // now playing
        Constraint::Percentage(35), // visualizer
    ])
    .split(outer[0]);

    draw_library(frame, panels[0], state.focused_panel == Panel::Library, state);
    draw_now_playing(frame, panels[1], state.focused_panel == Panel::NowPlaying, state);
    draw_visualizer(frame, panels[2], state.focused_panel == Panel::Visualizer, state);
    draw_queue(frame, outer[1], state.focused_panel == Panel::Queue, state);
}

fn draw_library(frame: &mut Frame, area: Rect, focused: bool, state: &AppState) {
    let block = Block::bordered()
        .title(" Library ")
        .border_style(if focused { Style::default().fg(Color::Cyan) } else { Style::default() });

    let items: Vec<ListItem> = state
        .library
        .iter()
        .map(|item| {
            let style = if item.is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(item.name.as_str()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
        .highlight_symbol("▶ ");

    frame.render_widget(list, area);
}

fn draw_now_playing(frame: &mut Frame, area: Rect, focused: bool, state: &AppState) {
    let block = Block::bordered()
        .title(" Now Playing ")
        .border_style(if focused { Style::default().fg(Color::Cyan) } else { Style::default() });

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(pb) = &state.playback else {
        let msg = Paragraph::new("No track playing")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    };

    let sections = Layout::vertical([
        Constraint::Min(0),    // track info
        Constraint::Length(1), // controls
        Constraint::Length(1), // progress bar
        Constraint::Length(1), // time
    ])
    .split(inner);

    // Track info
    let info_lines = vec![
        Line::from(pb.title.as_str())
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center),
        Line::from(pb.artist.as_str())
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center),
        Line::from(pb.album.as_str())
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
    ];
    let info = Paragraph::new(info_lines).alignment(Alignment::Center);
    frame.render_widget(info, sections[0]);

    // Controls
    let play_icon = if pb.is_paused { "▶" } else { "▐▐" };
    let shuffle_icon = if pb.shuffle { "🔀" } else { "  " };
    let repeat_icon = match pb.repeat {
        RepeatMode::Off => "  ",
        RepeatMode::One => "🔂",
        RepeatMode::All => "🔁",
    };
    let vol = format!("Vol:{}%", pb.volume);
    let controls = Line::from(vec![
        Span::raw(shuffle_icon),
        Span::raw("  ◄◄  "),
        Span::styled(play_icon, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("  ►►  "),
        Span::raw(repeat_icon),
        Span::raw("  "),
        Span::styled(vol, Style::default().fg(Color::DarkGray)),
    ]);
    let controls = Paragraph::new(controls).alignment(Alignment::Center);
    frame.render_widget(controls, sections[1]);

    // Progress bar
    let progress = if pb.total.as_secs_f64() > 0.0 {
        pb.elapsed.as_secs_f64() / pb.total.as_secs_f64()
    } else {
        0.0
    };
    let gauge = LineGauge::default()
        .filled_style(Style::default().fg(Color::Cyan))
        .unfilled_style(Style::default().fg(Color::DarkGray))
        .filled_symbol("━")
        .unfilled_symbol("─")
        .ratio(progress.clamp(0.0, 1.0));
    frame.render_widget(gauge, sections[2]);

    // Time
    let elapsed_str = format_duration(pb.elapsed);
    let total_str = format_duration(pb.total);
    let time_line = Line::from(vec![
        Span::styled(elapsed_str, Style::default().fg(Color::White)),
        Span::raw(" / "),
        Span::styled(total_str, Style::default().fg(Color::DarkGray)),
    ]);
    let time = Paragraph::new(time_line).alignment(Alignment::Center);
    frame.render_widget(time, sections[3]);
}

fn draw_visualizer(frame: &mut Frame, area: Rect, focused: bool, state: &AppState) {
    let block = Block::bordered()
        .title(" Visualizer ")
        .border_style(if focused { Style::default().fg(Color::Cyan) } else { Style::default() });
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.spectrum.is_empty() {
        let msg = Paragraph::new("⠁⠃⠇⡇⣇⣧⣷⣿")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    }

    let max_val = *state.spectrum.iter().max().unwrap_or(&1);
    let data: Vec<(&str, u64)> = state
        .spectrum
        .iter()
        .map(|&v| ("", v))
        .collect();

    let bar_chart = BarChart::default()
        .data(&data)
        .bar_gap(0)
        .bar_width(1)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .max(max_val);

    frame.render_widget(bar_chart, inner);
}

fn draw_queue(frame: &mut Frame, area: Rect, focused: bool, state: &AppState) {
    let block = Block::bordered()
        .title(" Queue ")
        .border_style(if focused { Style::default().fg(Color::Cyan) } else { Style::default() });
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.queue.is_empty() {
        let msg = Paragraph::new("Queue is empty")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    }

    let items: Vec<Span> = state
        .queue
        .iter()
        .enumerate()
        .flat_map(|(i, item)| {
            let mut spans = vec![];
            if i > 0 {
                spans.push(Span::styled(" → ", Style::default().fg(Color::DarkGray)));
            }
            if item.is_current {
                spans.push(Span::styled("▶ ", Style::default().fg(Color::Green)));
                spans.push(Span::styled(
                    item.title.as_str(),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    item.title.as_str(),
                    Style::default().fg(Color::White),
                ));
            }
            spans
        })
        .collect();

    let queue_line = Paragraph::new(Line::from(items));
    frame.render_widget(queue_line, inner);
}

fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{mins}:{secs:02}")
}
