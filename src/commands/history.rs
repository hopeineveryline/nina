use anyhow::{Context, Result};
use clap::Args;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::time::Duration;

use crate::commands::AppContext;

#[derive(Debug, Clone, Args)]
pub struct HistoryArgs {
    #[arg(long)]
    pub on: Option<String>,
    #[arg(long, default_value_t = false)]
    pub tui: bool,
}

#[derive(Debug, Clone)]
struct GenerationEntry {
    generation: u32,
    summary: String,
    current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HistoryOutcome {
    Switch(u32),
    Diff(u32, u32),
}

pub async fn run(ctx: &AppContext, args: HistoryArgs) -> Result<()> {
    let machine = ctx.machine(&args.on)?;
    let output = crate::exec::run(
        &machine,
        "nix-env --list-generations -p /nix/var/nix/profiles/system",
    )
    .await
    .context("couldn't load generation history")?;

    if !output.success() {
        anyhow::bail!("couldn't load generation history: {}", output.stderr);
    }

    let generations = parse_generations(&output.stdout);
    if generations.is_empty() {
        ctx.output
            .warn("i couldn't find any system generations yet.");
        return Ok(());
    }

    let mut app = HistoryApp::new(
        machine.name.clone(),
        generations,
        ctx.config.animate,
        ctx.config.dango_pos.clone(),
    );
    match app.run().await? {
        Some(HistoryOutcome::Switch(generation)) => {
            crate::commands::go::run(
                ctx,
                crate::commands::go::GoArgs {
                    generation,
                    on: args.on,
                },
            )
            .await
        }
        Some(HistoryOutcome::Diff(from, to)) => {
            crate::commands::diff::run(
                ctx,
                crate::commands::diff::DiffArgs {
                    from: Some(from),
                    to: Some(to),
                    on: args.on,
                },
            )
            .await
        }
        None => Ok(()),
    }
}

struct HistoryApp {
    machine: String,
    generations: Vec<GenerationEntry>,
    selected: usize,
    confirm: Option<HistoryOutcome>,
    animate: bool,
    dango_pos: String,
}

impl HistoryApp {
    fn new(
        machine: String,
        generations: Vec<GenerationEntry>,
        animate: bool,
        dango_pos: String,
    ) -> Self {
        let selected = generations.iter().position(|row| row.current).unwrap_or(0);
        Self {
            machine,
            generations,
            selected,
            confirm: None,
            animate,
            dango_pos,
        }
    }

    async fn run(&mut self) -> Result<Option<HistoryOutcome>> {
        let mut terminal = crate::tui::init_terminal()?;
        let result = self.run_loop(&mut terminal);
        crate::tui::restore_terminal(&mut terminal)?;
        result
    }

    fn run_loop(
        &mut self,
        terminal: &mut crate::tui::TuiTerminal,
    ) -> Result<Option<HistoryOutcome>> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if event::poll(Duration::from_millis(50))? {
                let evt = event::read()?;
                if let Event::Key(key) = evt {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match self.handle_key(key.code) {
                        HistoryControl::Continue => {}
                        HistoryControl::Exit => return Ok(None),
                        HistoryControl::Action(outcome) => return Ok(Some(outcome)),
                    }
                }
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode) -> HistoryControl {
        if self.confirm.is_some() {
            return match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    HistoryControl::Action(self.confirm.take().expect("confirm present"))
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirm = None;
                    HistoryControl::Continue
                }
                _ => HistoryControl::Continue,
            };
        }

        match code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                HistoryControl::Continue
            }
            KeyCode::Down => {
                if self.selected + 1 < self.generations.len() {
                    self.selected += 1;
                }
                HistoryControl::Continue
            }
            KeyCode::Enter => {
                let generation = self.generations[self.selected].generation;
                self.confirm = Some(HistoryOutcome::Switch(generation));
                HistoryControl::Continue
            }
            KeyCode::Char('d') => {
                if let Some(current) = self.current_generation() {
                    let generation = self.generations[self.selected].generation;
                    if generation != current {
                        self.confirm = Some(HistoryOutcome::Diff(generation, current));
                    }
                }
                HistoryControl::Continue
            }
            KeyCode::Esc => HistoryControl::Exit,
            _ => HistoryControl::Continue,
        }
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.size();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(2)])
            .split(area);
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(if self.animate && self.dango_pos != "off" {
                [Constraint::Min(20), Constraint::Length(6)]
            } else {
                [Constraint::Min(20), Constraint::Length(0)]
            })
            .split(layout[0]);

        let items = self
            .generations
            .iter()
            .enumerate()
            .map(|(idx, row)| {
                let marker = if idx == self.selected { "▸" } else { " " };
                let current = if row.current { "  ← current" } else { "" };
                let style = if idx == self.selected {
                    Style::default()
                        .fg(Color::Rgb(255, 220, 230))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!(
                    "{marker} {:>4}  {}{}",
                    row.generation, row.summary, current
                ))
                .style(style)
            })
            .collect::<Vec<_>>();

        let history = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("nina · generation history · {}", self.machine)),
        );
        frame.render_widget(history, body[0]);

        let footer = Paragraph::new(
            "↑↓ navigate  ·  [enter] switch  ·  [d] diff from current  ·  [esc] exit",
        )
        .block(Block::default().borders(Borders::ALL).title("keys"));
        frame.render_widget(footer, layout[1]);

        if self.animate && self.dango_pos != "off" && body[1].width >= 6 && body[1].height >= 5 {
            let dango_area = Rect {
                x: body[1].x,
                y: body[1].y + body[1].height.saturating_sub(5),
                width: 6,
                height: 5,
            };
            let dango =
                crate::dango::frame_at(crate::dango::DangoAnimation::WalkBack, self.selected);
            let widget = Paragraph::new(dango);
            frame.render_widget(widget, dango_area);
        }

        if let Some(confirm) = &self.confirm {
            let popup = centered_rect(60, 20, area);
            frame.render_widget(Clear, popup);
            let message = match confirm {
                HistoryOutcome::Switch(generation) => {
                    format!("switch to generation {generation}? (y/n)")
                }
                HistoryOutcome::Diff(generation, current) => {
                    format!("diff generation {generation} against current {current}? (y/n)")
                }
            };
            let widget = Paragraph::new(message).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("confirm history action"),
            );
            frame.render_widget(widget, popup);
        }
    }

    fn current_generation(&self) -> Option<u32> {
        self.generations
            .iter()
            .find(|row| row.current)
            .map(|row| row.generation)
    }
}

enum HistoryControl {
    Continue,
    Exit,
    Action(HistoryOutcome),
}

fn parse_generations(stdout: &str) -> Vec<GenerationEntry> {
    stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }

            let mut parts = trimmed.split_whitespace();
            let generation: u32 = parts.next()?.parse().ok()?;
            let current = trimmed.contains("(current)");
            let summary = trimmed
                .strip_prefix(&generation.to_string())
                .unwrap_or(trimmed)
                .trim()
                .replace("(current)", "")
                .trim()
                .to_string();

            Some(GenerationEntry {
                generation,
                summary,
                current,
            })
        })
        .collect()
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::parse_generations;

    #[test]
    fn parses_generation_listing() {
        let rows = parse_generations(
            "  13   2026-03-28 14:22:01   (current)\n  12   2026-03-27 09:30:00\n",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].generation, 13);
        assert!(rows[0].current);
        assert_eq!(rows[1].generation, 12);
        assert!(!rows[1].current);
    }
}
