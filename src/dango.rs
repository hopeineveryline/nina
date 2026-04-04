use std::io::{self, Write};
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::interval;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangoAnimation {
    Idle,
    Happy,
    Sad,
    Sweep,
    WalkBack,
    Wave,
    Spin,
    Dance,
}

pub struct DangoPlayer {
    stop_tx: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

pub fn frame_at(anim: DangoAnimation, idx: usize) -> &'static str {
    let frames = frames_for(anim);
    frames[idx % frames.len()]
}

pub fn position_from_pref(pref: &str, width: u16, height: u16) -> (u16, u16) {
    let right = width.saturating_sub(6).max(1);
    let bottom = height.saturating_sub(5).max(1);
    match pref {
        "off" => (0, 0),
        "top-left" => (1, 1),
        "top-right" => (right, 1),
        "bottom-left" => (1, bottom),
        "bottom-right" => (right, bottom),
        "auto" => {
            if height > 20 {
                (right, 1)
            } else {
                (right, bottom)
            }
        }
        _ => (right, 1),
    }
}

impl DangoPlayer {
    pub fn start(anim: DangoAnimation, pos: (u16, u16)) -> Self {
        let (tx, mut rx) = oneshot::channel::<()>();
        let frames = frames_for(anim);
        let mut ticker = interval(rate_for(anim));
        let handle = tokio::spawn(async move {
            let mut i = 0usize;
            loop {
                tokio::select! {
                    _ = &mut rx => break,
                    _ = ticker.tick() => {
                        draw_frame(frames[i % frames.len()], pos);
                        i += 1;
                    }
                }
            }
            clear_at(pos);
        });

        Self {
            stop_tx: Some(tx),
            handle,
        }
    }

    pub fn play_once(anim: DangoAnimation, pos: (u16, u16)) -> Self {
        let frames = frames_for(anim);
        let rate = rate_for(anim);
        let (tx, _rx) = oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            let mut ticker = interval(rate);
            for frame in frames {
                ticker.tick().await;
                draw_frame(frame, pos);
            }
            clear_at(pos);
        });
        Self {
            stop_tx: Some(tx),
            handle,
        }
    }

    pub async fn stop(mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        let _ = self.handle.await;
    }
}

fn draw_frame(frame: &str, (col, row): (u16, u16)) {
    if col == 0 || row == 0 {
        return;
    }
    print!(
        "\x1b[s\x1b[{};{}H{}\x1b[u",
        row,
        col,
        frame.replace('\n', "\n\r")
    );
    let _ = io::stdout().flush();
}

fn clear_at((col, row): (u16, u16)) {
    if col == 0 || row == 0 {
        return;
    }

    print!("\x1b[s");
    for offset in 0..4 {
        print!("\x1b[{};{}H        ", row + offset, col);
    }
    print!("\x1b[u");
    let _ = io::stdout().flush();
}

fn rate_for(anim: DangoAnimation) -> Duration {
    match anim {
        DangoAnimation::Idle => Duration::from_millis(400),
        DangoAnimation::Happy => Duration::from_millis(120),
        DangoAnimation::Sad => Duration::from_millis(600),
        DangoAnimation::Sweep => Duration::from_millis(200),
        DangoAnimation::WalkBack => Duration::from_millis(180),
        DangoAnimation::Wave => Duration::from_millis(200),
        DangoAnimation::Spin => Duration::from_millis(100),
        DangoAnimation::Dance => Duration::from_millis(150),
    }
}

fn frames_for(anim: DangoAnimation) -> &'static [&'static str] {
    match anim {
        DangoAnimation::Idle => &["(●)\n(●)\n(●)\n | ", "(●)\n(●.)\n(●)\n | "],
        DangoAnimation::Happy => &["(●)\n(●)\n(●)\n | ", "/(●)\\\n(●)\n(●)\n | "],
        DangoAnimation::Sad => &["(●)\n(;_;)\n(●)\n | ", "(●)\n(●)\n(●)\n | "],
        DangoAnimation::Sweep => &["(●)\n(●)\n(●)\n |/~", "(●)\n(●)\n(●)\n |/~~~"],
        DangoAnimation::WalkBack => &["(●)\n(●)\n(●)\n\\| ", "(●)\n(●)\n(●)\n |/"],
        DangoAnimation::Wave => &["(●)\n(●)\n(●)\n | ", "(●)/\n(●)\n(●)\n | "],
        DangoAnimation::Spin => &["(●)\n(◉)\n(○)\n | ", "(◉)\n(○)\n(◌)\n | "],
        DangoAnimation::Dance => &["(●)\n(●)\n(●)\n | ", "\\(●)/\n(●)\n(●)\n/|\\"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! animation_contract_tests {
        ($($name:ident => { anim: $anim:expr, rate_ms: $rate_ms:expr, frames: [$first:literal, $second:literal $(,)?] },)+) => {
            $(
                mod $name {
                    use super::*;

                    #[test]
                    fn frame_zero_matches() {
                        assert_eq!(frame_at($anim, 0), $first);
                    }

                    #[test]
                    fn frame_cycle_matches() {
                        assert_eq!(frame_at($anim, 1), $second);
                        assert_eq!(frame_at($anim, 2), $first);
                    }

                    #[test]
                    fn rate_matches() {
                        assert_eq!(rate_for($anim), Duration::from_millis($rate_ms));
                    }

                    #[test]
                    fn frames_stay_multiline() {
                        for frame in frames_for($anim) {
                            assert_eq!(frame.lines().count(), 4, "unexpected frame height: {frame}");
                        }
                    }
                }
            )+
        };
    }

    animation_contract_tests! {
        idle => { anim: DangoAnimation::Idle, rate_ms: 400, frames: ["(●)\n(●)\n(●)\n | ", "(●)\n(●.)\n(●)\n | "] },
        happy => { anim: DangoAnimation::Happy, rate_ms: 120, frames: ["(●)\n(●)\n(●)\n | ", "/(●)\\\n(●)\n(●)\n | "] },
        sad => { anim: DangoAnimation::Sad, rate_ms: 600, frames: ["(●)\n(;_;)\n(●)\n | ", "(●)\n(●)\n(●)\n | "] },
        sweep => { anim: DangoAnimation::Sweep, rate_ms: 200, frames: ["(●)\n(●)\n(●)\n |/~", "(●)\n(●)\n(●)\n |/~~~"] },
        walk_back => { anim: DangoAnimation::WalkBack, rate_ms: 180, frames: ["(●)\n(●)\n(●)\n\\| ", "(●)\n(●)\n(●)\n |/"] },
        wave => { anim: DangoAnimation::Wave, rate_ms: 200, frames: ["(●)\n(●)\n(●)\n | ", "(●)/\n(●)\n(●)\n | "] },
        spin => { anim: DangoAnimation::Spin, rate_ms: 100, frames: ["(●)\n(◉)\n(○)\n | ", "(◉)\n(○)\n(◌)\n | "] },
        dance => { anim: DangoAnimation::Dance, rate_ms: 150, frames: ["(●)\n(●)\n(●)\n | ", "\\(●)/\n(●)\n(●)\n/|\\"] },
    }

    #[test]
    fn top_left_pref_anchors_origin() {
        assert_eq!(position_from_pref("top-left", 80, 24), (1, 1));
    }

    #[test]
    fn top_right_pref_uses_safe_margin() {
        assert_eq!(position_from_pref("top-right", 80, 24), (74, 1));
    }

    #[test]
    fn bottom_left_pref_uses_safe_margin() {
        assert_eq!(position_from_pref("bottom-left", 80, 24), (1, 19));
    }

    #[test]
    fn bottom_right_pref_uses_safe_margin() {
        assert_eq!(position_from_pref("bottom-right", 80, 24), (74, 19));
    }

    #[test]
    fn auto_pref_prefers_top_right_on_tall_terminals() {
        assert_eq!(position_from_pref("auto", 80, 24), (74, 1));
    }

    #[test]
    fn auto_pref_prefers_bottom_right_on_short_terminals() {
        assert_eq!(position_from_pref("auto", 80, 20), (74, 15));
    }

    #[test]
    fn off_pref_disables_drawing() {
        assert_eq!(position_from_pref("off", 80, 24), (0, 0));
    }

    #[test]
    fn unknown_pref_falls_back_to_top_right() {
        assert_eq!(position_from_pref("mystery", 80, 24), (74, 1));
    }

    #[test]
    fn safe_margin_saturates_on_small_terminals() {
        assert_eq!(position_from_pref("bottom-right", 4, 3), (1, 1));
    }
}
