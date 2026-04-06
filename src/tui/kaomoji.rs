use std::time::{Duration, Instant};

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const FRAME_MS: u64 = 170;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KaomojiReaction {
    SearchReady,
    SearchEmpty,
    DetailPeek,
    Copy,
    Prompt,
    Cancel,
    Error,
}

#[derive(Debug, Clone, Copy)]
struct ActiveBurst {
    reaction: KaomojiReaction,
    started_at: Instant,
}

#[derive(Debug, Default)]
pub struct KaomojiBurst {
    active: Option<ActiveBurst>,
    last_trigger: Option<Instant>,
}

impl KaomojiBurst {
    pub fn maybe_trigger(&mut self, reaction: KaomojiReaction) {
        self.maybe_trigger_at(Instant::now(), reaction);
    }

    #[cfg(test)]
    pub fn current_reaction(&self) -> Option<KaomojiReaction> {
        self.current_reaction_at(Instant::now())
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        let Some(text) = self.current_frame_at(Instant::now()) else {
            return;
        };

        let popup = popup_area(area, text);
        if popup.width == 0 || popup.height == 0 {
            return;
        }

        let bubble = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 220, 232))
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .title("nina")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(246, 186, 208))),
            );

        frame.render_widget(Clear, popup);
        frame.render_widget(bubble, popup);
    }

    fn maybe_trigger_at(&mut self, now: Instant, reaction: KaomojiReaction) {
        if self
            .last_trigger
            .is_some_and(|last| now.duration_since(last) < cooldown_for(reaction))
        {
            return;
        }

        self.active = Some(ActiveBurst {
            reaction,
            started_at: now,
        });
        self.last_trigger = Some(now);
    }

    #[cfg(test)]
    fn current_reaction_at(&self, now: Instant) -> Option<KaomojiReaction> {
        let active = self.active?;
        let elapsed = now.duration_since(active.started_at);
        if elapsed >= total_duration(active.reaction) {
            None
        } else {
            Some(active.reaction)
        }
    }

    fn current_frame_at(&self, now: Instant) -> Option<&'static str> {
        let active = self.active?;
        let elapsed = now.duration_since(active.started_at);
        frame_for(active.reaction, elapsed)
    }
}

fn popup_area(area: Rect, text: &str) -> Rect {
    let lines = text.lines().collect::<Vec<_>>();
    let height = lines.len() as u16 + 2;
    let width = lines
        .iter()
        .map(|line| line.chars().count() as u16)
        .max()
        .unwrap_or(0)
        + 4;

    if area.width <= width + 2 || area.height <= height + 1 {
        return Rect::default();
    }

    Rect {
        x: area.x + area.width.saturating_sub(width + 1),
        y: area.y + 1,
        width,
        height,
    }
}

fn cooldown_for(reaction: KaomojiReaction) -> Duration {
    match reaction {
        KaomojiReaction::SearchReady => Duration::from_millis(1200),
        KaomojiReaction::SearchEmpty => Duration::from_millis(1500),
        KaomojiReaction::DetailPeek => Duration::from_millis(650),
        KaomojiReaction::Copy => Duration::from_millis(650),
        KaomojiReaction::Prompt => Duration::from_millis(800),
        KaomojiReaction::Cancel => Duration::from_millis(850),
        KaomojiReaction::Error => Duration::from_millis(1000),
    }
}

#[cfg(test)]
fn total_duration(reaction: KaomojiReaction) -> Duration {
    Duration::from_millis(frames_for(reaction).len() as u64 * FRAME_MS)
}

fn frame_for(reaction: KaomojiReaction, elapsed: Duration) -> Option<&'static str> {
    let frames = frames_for(reaction);
    let idx = (elapsed.as_millis() / FRAME_MS as u128) as usize;
    frames.get(idx).copied()
}

fn frames_for(reaction: KaomojiReaction) -> &'static [&'static str] {
    match reaction {
        KaomojiReaction::SearchReady => &[
            "  .｡.:*♡\n (˶ᵔ ᵕ ᵔ˶)",
            "  *:･ﾟ✧\n(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧",
            "  ♡ ｡ﾟ.\n (˶ᵔ ᵕ ᵔ˶)",
        ],
        KaomojiReaction::SearchEmpty => &[
            "  ...\n(｡•́︿•̀｡)",
            "  ˚｡⋆\n( ´ . .\u{032b} . `)",
            "  ...\n(｡•́︿•̀｡)",
        ],
        KaomojiReaction::DetailPeek => {
            &["  ✦ ｡˚\n (｡☌ᴗ☌｡)", "  .⋆｡\n( ✨｡☌ᴗ☌｡)", "  ✦ ｡˚\n (｡☌ᴗ☌｡)"]
        }
        KaomojiReaction::Copy => &[
            "  ♡ ♡ ♡\n(づ｡◕‿‿◕｡)づ",
            "  *:･ﾟ✧\n ✧(｡•̀ᴗ-)✧",
            "  ♡ ♡ ♡\n(づ｡◕‿‿◕｡)づ",
        ],
        KaomojiReaction::Prompt => &[
            "  ♡ ? ♡\n(⊃｡•́‿•̀｡)⊃",
            "  .｡.:☆\n (ง ˙˘˙ )ว",
            "  ♡ ? ♡\n(⊃｡•́‿•̀｡)⊃",
        ],
        KaomojiReaction::Cancel => &[
            "  .｡.:*\n( ˘͈ ᵕ ˘͈ )",
            "  ｡ﾟ.\n ( ᵕ—ᴗ— )",
            "  .｡.:*\n( ˘͈ ᵕ ˘͈ )",
        ],
        KaomojiReaction::Error => &["  ...\n (；ω；)", "  ⊹˚.\n(っ˘̩╭╮˘̩)っ", "  ...\n (；ω；)"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use ratatui::layout::Rect;

    macro_rules! reaction_contract_tests {
        ($($name:ident => { reaction: $reaction:expr, cooldown_ms: $cooldown_ms:expr, first: $first:literal, second: $second:literal, third: $third:literal $(,)? },)+) => {
            $(
                mod $name {
                    use super::*;

                    #[test]
                    fn cooldown_matches() {
                        assert_eq!(super::cooldown_for($reaction), Duration::from_millis($cooldown_ms));
                    }

                    #[test]
                    fn frames_match_script() {
                        assert_eq!(super::frame_for($reaction, Duration::from_millis(0)), Some($first));
                        assert_eq!(super::frame_for($reaction, Duration::from_millis(170)), Some($second));
                        assert_eq!(super::frame_for($reaction, Duration::from_millis(340)), Some($third));
                    }

                    #[test]
                    fn total_duration_tracks_three_frames() {
                        assert_eq!(super::total_duration($reaction), Duration::from_millis(510));
                    }

                    #[test]
                    fn frames_expire_after_burst() {
                        assert_eq!(super::frame_for($reaction, Duration::from_millis(700)), None);
                    }
                }
            )+
        };
    }

    reaction_contract_tests! {
        search_ready => {
            reaction: KaomojiReaction::SearchReady,
            cooldown_ms: 1200,
            first: "  .｡.:*♡\n (˶ᵔ ᵕ ᵔ˶)",
            second: "  *:･ﾟ✧\n(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧",
            third: "  ♡ ｡ﾟ.\n (˶ᵔ ᵕ ᵔ˶)",
        },
        search_empty => {
            reaction: KaomojiReaction::SearchEmpty,
            cooldown_ms: 1500,
            first: "  ...\n(｡•́︿•̀｡)",
            second: "  ˚｡⋆\n( ´ . .\u{032b} . `)",
            third: "  ...\n(｡•́︿•̀｡)",
        },
        detail_peek => {
            reaction: KaomojiReaction::DetailPeek,
            cooldown_ms: 650,
            first: "  ✦ ｡˚\n (｡☌ᴗ☌｡)",
            second: "  .⋆｡\n( ✨｡☌ᴗ☌｡)",
            third: "  ✦ ｡˚\n (｡☌ᴗ☌｡)",
        },
        copy => {
            reaction: KaomojiReaction::Copy,
            cooldown_ms: 650,
            first: "  ♡ ♡ ♡\n(づ｡◕‿‿◕｡)づ",
            second: "  *:･ﾟ✧\n ✧(｡•̀ᴗ-)✧",
            third: "  ♡ ♡ ♡\n(づ｡◕‿‿◕｡)づ",
        },
        prompt => {
            reaction: KaomojiReaction::Prompt,
            cooldown_ms: 800,
            first: "  ♡ ? ♡\n(⊃｡•́‿•̀｡)⊃",
            second: "  .｡.:☆\n (ง ˙˘˙ )ว",
            third: "  ♡ ? ♡\n(⊃｡•́‿•̀｡)⊃",
        },
        cancel => {
            reaction: KaomojiReaction::Cancel,
            cooldown_ms: 850,
            first: "  .｡.:*\n( ˘͈ ᵕ ˘͈ )",
            second: "  ｡ﾟ.\n ( ᵕ—ᴗ— )",
            third: "  .｡.:*\n( ˘͈ ᵕ ˘͈ )",
        },
        error => {
            reaction: KaomojiReaction::Error,
            cooldown_ms: 1000,
            first: "  ...\n (；ω；)",
            second: "  ⊹˚.\n(っ˘̩╭╮˘̩)っ",
            third: "  ...\n (；ω；)",
        },
    }

    #[test]
    fn burst_expires_after_last_frame() {
        let now = Instant::now();
        let mut burst = KaomojiBurst::default();
        burst.maybe_trigger_at(now, KaomojiReaction::Copy);

        assert_eq!(
            burst.current_reaction_at(now + Duration::from_millis(100)),
            Some(KaomojiReaction::Copy)
        );
        assert_eq!(
            burst.current_reaction_at(now + Duration::from_millis(700)),
            None
        );
    }

    #[test]
    fn cooldown_blocks_immediate_retrigger() {
        let now = Instant::now();
        let mut burst = KaomojiBurst::default();
        burst.maybe_trigger_at(now, KaomojiReaction::SearchReady);
        burst.maybe_trigger_at(now + Duration::from_millis(200), KaomojiReaction::Error);

        assert_eq!(
            burst.current_reaction_at(now + Duration::from_millis(250)),
            Some(KaomojiReaction::SearchReady)
        );
        burst.maybe_trigger_at(now + Duration::from_millis(1400), KaomojiReaction::Error);
        assert_eq!(
            burst.current_reaction_at(now + Duration::from_millis(1450)),
            Some(KaomojiReaction::Error)
        );
    }

    #[test]
    fn popup_area_places_bubble_in_top_right_corner() {
        let area = super::popup_area(Rect::new(0, 0, 80, 24), "hi\nnina");
        assert_eq!(area, Rect::new(71, 1, 8, 4));
    }

    #[test]
    fn popup_area_returns_default_for_tiny_space() {
        assert_eq!(
            super::popup_area(Rect::new(0, 0, 8, 4), "hello"),
            Rect::default()
        );
    }
}
