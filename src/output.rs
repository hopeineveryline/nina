//! Warm, colorful output for nina — matching the personality of nina.asha.software
//!
//! Color palette (from the website):
//!   pink     #ff9ac7  — accent, highlights
//!   lavender #bfa5ff  — accent-2, secondary accent
//!   mint     #9dffce  — success, positive states
//!   gold     #ffd166  — warnings, attention
//!   warm     #f8efff  — primary text
//!   muted    #c9bad6  — secondary text
//!   border   #3a2f48  — separators, structural elements

use std::io::{self, IsTerminal};

#[derive(Debug, Clone)]
pub struct Output {
    pub color: bool,
    pub teach: bool,
}

impl Output {
    pub fn new(color: bool, teach: bool) -> Self {
        Self {
            color: color && io::stdout().is_terminal(),
            teach,
        }
    }

    // ─── primary message types ─────────────────────────────────────────────

    /// General info — pink ✦
    pub fn info(&self, message: &str) {
        println!("{} {}", self.symbol("✦", PINK), message);
    }

    /// Positive outcome — mint ✓
    pub fn success(&self, message: &str) {
        println!("{} {}", self.symbol("✓", MINT), message);
    }

    /// Warning or attention — gold ⚠
    pub fn warn(&self, message: &str) {
        println!("{} {}", self.symbol("⚠", GOLD), message);
    }

    /// Error — soft rose ✗
    pub fn error(&self, message: &str) {
        eprintln!("{} {}", self.symbol("✗", ROSE), message);
    }

    /// Rollback indicator — lavender ↩
    pub fn rollback(&self, message: &str) {
        println!("{} {}", self.symbol("↩", LAVENDER), message);
    }

    /// Step or continuation — muted →
    pub fn step(&self, message: &str) {
        println!("{} {}", self.symbol("→", MUTED), message);
    }

    // ─── kaomoji reactions ─────────────────────────────────────────────────

    /// Warm nina greeting (˶ᵔ ᵕ ᵔ˶)
    pub fn face(&self, message: &str) {
        self.kao(Kaomoji::Nina, message);
    }

    /// Soft happy reaction (⑅˘꒳˘)♡ — used for successes
    pub fn happy(&self, message: &str) {
        self.kao(Kaomoji::SoftBlush, message);
    }

    /// Excited reaction (ง ˙˘˙ )ว — used for discoveries or enthusiasm
    pub fn excited(&self, message: &str) {
        self.kao(Kaomoji::Excited, message);
    }

    /// Sad reaction (｡•́︿•̀｡) — used for empty states or disappointments
    pub fn sad(&self, message: &str) {
        self.kao(Kaomoji::Sad, message);
    }

    /// Curious reaction (˘ω˘) — used for prompts or questions
    pub fn curious(&self, message: &str) {
        self.kao(Kaomoji::Curious, message);
    }

    /// Cozy reaction (˶ᵔ ᵕ ᵔ˶)♡ — used for mood or status warmth
    pub fn cozy(&self, message: &str) {
        self.kao(Kaomoji::Cozy, message);
    }

    fn kao(&self, k: Kaomoji, message: &str) {
        println!("{} {}", k.symbol(self), message);
    }

    // ─── structured output ─────────────────────────────────────────────────

    /// Key-value pair — label in muted lavender, value in warm white
    /// e.g. "  generation    142"
    pub fn kv(&self, label: &str, value: &str) {
        println!(
            "  {}  {}",
            self.paint(label, MUTED),
            self.paint(value, WARM_WHITE)
        );
    }

    /// Label with a colored value (value inherits label's color)
    pub fn kv_succ(&self, label: &str, value: &str) {
        println!("  {}  {}", self.dim(label), self.paint(value, MINT));
    }

    pub fn kv_warn(&self, label: &str, value: &str) {
        println!("  {}  {}", self.dim(label), self.paint(value, GOLD));
    }

    pub fn kv_err(&self, label: &str, value: &str) {
        println!("  {}  {}", self.dim(label), self.paint(value, ROSE));
    }

    /// Section label — used for grouped output headers
    pub fn section(&self, label: &str) {
        println!("\n  {}", self.paint(label, PINK));
    }

    /// Small framed header for prompt/session moments.
    pub fn hero(&self, title: &str, subtitle: &str) {
        let content_width = title
            .chars()
            .count()
            .max(subtitle.chars().count())
            .max(24);
        let border = format!("╭{}╮", "─".repeat(content_width + 2));
        let footer = format!("╰{}╯", "─".repeat(content_width + 2));
        println!(
            "  {}",
            self.paint(&border, BORDER)
        );
        println!(
            "  {} {} {}",
            self.paint("│", BORDER),
            self.paint(&pad_right(title, content_width), PINK),
            self.paint("│", BORDER)
        );
        println!(
            "  {} {} {}",
            self.paint("│", BORDER),
            self.paint(&pad_right(subtitle, content_width), WARM_WHITE),
            self.paint("│", BORDER)
        );
        println!("  {}", self.paint(&footer, BORDER));
    }

    /// Separator line for visual grouping
    pub fn sep(&self) {
        println!(
            "{}",
            self.paint("  ────────────────────────────────────────", BORDER)
        );
    }

    /// Tip or hint in muted style
    pub fn tip(&self, message: &str) {
        println!("  {} {}", self.dim("→"), self.dim(message));
    }

    // ─── diff output ──────────────────────────────────────────────────────

    /// Print a diff preview with context
    pub fn diff(&self, content: &str) {
        println!("\n{}\n", content);
    }

    // ─── raw text helpers ────────────────────────────────────────────────

    /// Plain println for when you just need to print
    pub fn print(&self, message: &str) {
        println!("{}", message);
    }

    pub fn print_muted(&self, message: &str) {
        println!("{}", self.dim(message));
    }

    pub fn blank(&self) {
        println!();
    }

    /// Plain print without color — use for help listings where terminal wrapping
    /// would split ANSI escape sequences across lines and corrupt the output.
    pub fn print_plain(&self, message: &str) {
        println!("{message}");
    }

    pub fn command_echo(&self, command: &str) {
        println!(
            "  {} {}",
            self.paint("nina>", PINK),
            self.paint(command, WARM_WHITE)
        );
    }

    pub fn status_line(&self, tag: &str, message: &str, color: RgbColor) {
        println!(
            "  {} {}",
            self.colored(&format!("[{tag}]"), color),
            self.paint(message, WARM_WHITE)
        );
    }

    pub fn prompt(&self, label: &str) -> String {
        format!("{} {} ", self.paint(label, PINK), self.paint("›", LAVENDER))
    }

    // ─── tech/teach mode ──────────────────────────────────────────────────

    pub fn teach_command(&self, command: &str) {
        if self.teach {
            println!("  ╌ {}", self.dim(&format!("ran: {command}")));
        }
    }

    // ─── internals ────────────────────────────────────────────────────────

    fn symbol(&self, text: &str, color: Color) -> String {
        self.paint(text, color)
    }

    fn paint(&self, text: &str, color: Color) -> String {
        if self.color {
            format!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                color.0, color.1, color.2, text
            )
        } else {
            text.to_string()
        }
    }

    fn dim(&self, text: &str) -> String {
        if self.color {
            format!("\x1b[38;2;{}m{}\x1b[0m", MUTED.rgb(), text)
        } else {
            text.to_string()
        }
    }

    /// Apply a color to arbitrary text
    pub fn colored(&self, text: &str, color: RgbColor) -> String {
        if self.color {
            format!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                color.0, color.1, color.2, text
            )
        } else {
            text.to_string()
        }
    }
}

fn pad_right(text: &str, width: usize) -> String {
    let pad = width.saturating_sub(text.chars().count());
    format!("{text}{}", " ".repeat(pad))
}

/// RGB color triple for terminal output — import with `use crate::output::RgbColor`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor(pub u8, pub u8, pub u8);

impl RgbColor {
    pub const PINK: RgbColor = RgbColor(255, 154, 199);
    pub const LAVENDER: RgbColor = RgbColor(191, 165, 255);
    pub const MINT: RgbColor = RgbColor(157, 255, 206);
    pub const GOLD: RgbColor = RgbColor(255, 209, 102);
    pub const ROSE: RgbColor = RgbColor(255, 182, 193);
}

// ─── color definitions ───────────────────────────────────────────────────────

/// Website palette — see index.html CSS variables
struct Color(u8, u8, u8);

impl Color {
    fn rgb(&self) -> String {
        format!("{};{};{}", self.0, self.1, self.2)
    }
}

const PINK: Color = Color(255, 154, 199);
const LAVENDER: Color = Color(191, 165, 255);
const MINT: Color = Color(157, 255, 206);
const GOLD: Color = Color(255, 209, 102);
const ROSE: Color = Color(255, 182, 193);
const WARM_WHITE: Color = Color(248, 239, 255);
const MUTED: Color = Color(201, 186, 214);
const BORDER: Color = Color(58, 47, 72);

// ─── kaomoji definitions ───────────────────────────────────────────────────

enum Kaomoji {
    Nina,      // ˶ᵔ ᵕ ᵔ˶ — warm general greeting
    SoftBlush, // ⑅˘꒳˘)♡ — happy success
    Excited,   // (ง ˙˘˙ )ว — enthusiasm / discovery
    Sad,       // (｡•́︿•̀｡) — disappointment / empty
    Curious,   // (˘ω˘) — prompt / question
    Cozy,      // ˶ᵔ ᵕ ᵔ˶)♡ — mood warmth
}

impl Kaomoji {
    fn symbol(&self, o: &Output) -> String {
        let face = match self {
            Kaomoji::Nina => "(˶ᵔ ᵕ ᵔ˶)",
            Kaomoji::SoftBlush => "(⑅˘꒳˘)♡",
            Kaomoji::Excited => "(ง ˙˘˙ )ว",
            Kaomoji::Sad => "(｡•́︿•̀｡)",
            Kaomoji::Curious => "(˘ω˘)",
            Kaomoji::Cozy => "(˶ᵔ ᵕ ᵔ˶)♡",
        };
        o.paint(face, PINK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_rgb_formats_triplet() {
        assert_eq!(Color(1, 2, 3).rgb(), "1;2;3");
    }

    #[test]
    fn paint_leaves_plain_text_when_color_is_off() {
        let output = Output {
            color: false,
            teach: false,
        };
        assert_eq!(output.paint("nina", PINK), "nina");
    }

    #[test]
    fn paint_wraps_escape_codes_when_color_is_on() {
        let output = Output {
            color: true,
            teach: false,
        };
        assert_eq!(
            output.paint("nina", PINK),
            "\x1b[38;2;255;154;199mnina\x1b[0m"
        );
    }

    #[test]
    fn dim_leaves_plain_text_when_color_is_off() {
        let output = Output {
            color: false,
            teach: false,
        };
        assert_eq!(output.dim("soft"), "soft");
    }

    #[test]
    fn dim_wraps_escape_codes_when_color_is_on() {
        let output = Output {
            color: true,
            teach: false,
        };
        assert_eq!(output.dim("soft"), "\x1b[38;2;201;186;214msoft\x1b[0m");
    }

    #[test]
    fn colored_uses_rgb_color_when_color_is_on() {
        let output = Output {
            color: true,
            teach: false,
        };
        assert_eq!(
            output.colored("tag", RgbColor::LAVENDER),
            "\x1b[38;2;191;165;255mtag\x1b[0m"
        );
    }

    #[test]
    fn colored_leaves_text_plain_when_color_is_off() {
        let output = Output {
            color: false,
            teach: false,
        };
        assert_eq!(output.colored("tag", RgbColor::LAVENDER), "tag");
    }

    #[test]
    fn prompt_stays_readable_without_color() {
        let output = Output {
            color: false,
            teach: false,
        };
        assert_eq!(output.prompt("nina"), "nina › ");
    }

    #[test]
    fn prompt_colors_label_and_chevron_when_enabled() {
        let output = Output {
            color: true,
            teach: false,
        };
        assert_eq!(
            output.prompt("nina"),
            "\x1b[38;2;255;154;199mnina\x1b[0m \x1b[38;2;191;165;255m›\x1b[0m "
        );
    }

    #[test]
    fn rgb_constants_match_palette() {
        assert_eq!(RgbColor::PINK, RgbColor(255, 154, 199));
        assert_eq!(RgbColor::LAVENDER, RgbColor(191, 165, 255));
        assert_eq!(RgbColor::MINT, RgbColor(157, 255, 206));
        assert_eq!(RgbColor::GOLD, RgbColor(255, 209, 102));
        assert_eq!(RgbColor::ROSE, RgbColor(255, 182, 193));
    }

    macro_rules! kaomoji_symbol_tests {
        ($($name:ident => $variant:expr, $face:literal,)+) => {
            $(
                #[test]
                fn $name() {
                    let output = Output {
                        color: false,
                        teach: false,
                    };
                    assert_eq!($variant.symbol(&output), $face);
                }
            )+
        };
    }

    kaomoji_symbol_tests! {
        nina_face_symbol => Kaomoji::Nina, "(˶ᵔ ᵕ ᵔ˶)",
        soft_blush_symbol => Kaomoji::SoftBlush, "(⑅˘꒳˘)♡",
        excited_symbol => Kaomoji::Excited, "(ง ˙˘˙ )ว",
        sad_symbol => Kaomoji::Sad, "(｡•́︿•̀｡)",
        curious_symbol => Kaomoji::Curious, "(˘ω˘)",
        cozy_symbol => Kaomoji::Cozy, "(˶ᵔ ᵕ ᵔ˶)♡",
    }

    macro_rules! colored_kaomoji_symbol_tests {
        ($($name:ident => $variant:expr,)+) => {
            $(
                #[test]
                fn $name() {
                    let output = Output {
                        color: true,
                        teach: false,
                    };
                    let rendered = $variant.symbol(&output);
                    assert!(rendered.starts_with("\x1b[38;2;255;154;199m"));
                    assert!(rendered.ends_with("\x1b[0m"));
                }
            )+
        };
    }

    colored_kaomoji_symbol_tests! {
        nina_face_symbol_is_colored => Kaomoji::Nina,
        soft_blush_symbol_is_colored => Kaomoji::SoftBlush,
        excited_symbol_is_colored => Kaomoji::Excited,
        sad_symbol_is_colored => Kaomoji::Sad,
        curious_symbol_is_colored => Kaomoji::Curious,
        cozy_symbol_is_colored => Kaomoji::Cozy,
    }
}
