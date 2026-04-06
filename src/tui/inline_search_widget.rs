use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
pub struct InlineSearchRenderModel {
    pub header_left: String,
    pub header_right: String,
    pub result_lines: Vec<String>,
    pub detail_lines: Vec<String>,
    pub hints: String,
    pub kaomoji: Option<&'static str>,
}

pub fn render(frame: &mut Frame<'_>, area: Rect, model: &InlineSearchRenderModel) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(58, 46, 80)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let kaomoji_width = if model.kaomoji.is_some() { 14 } else { 0 };
    let main_width = inner.width.saturating_sub(kaomoji_width);
    let main_area = Rect {
        x: inner.x,
        y: inner.y,
        width: main_width,
        height: inner.height,
    };

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(main_area);

    let header = Paragraph::new(format!(
        "{}\n{}",
        truncate(&model.header_left, main_area.width as usize),
        pad_lr("", &model.header_right, main_area.width as usize)
    ))
    .style(
        Style::default()
            .fg(Color::Rgb(232, 223, 252))
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(header, sections[0]);

    let results = Paragraph::new(model.result_lines.join("\n"))
        .style(Style::default().fg(Color::Rgb(232, 223, 252)));
    frame.render_widget(results, sections[1]);

    frame.render_widget(separator(main_area.width), sections[2]);

    let detail = Paragraph::new(model.detail_lines.join("\n"))
        .style(Style::default().fg(Color::Rgb(244, 239, 252)));
    frame.render_widget(detail, sections[3]);

    frame.render_widget(separator(main_area.width), sections[4]);

    let hints = Paragraph::new(truncate(&model.hints, main_area.width as usize))
        .style(Style::default().fg(Color::Rgb(190, 178, 209)));
    frame.render_widget(hints, sections[5]);

    if let Some(kaomoji) = model.kaomoji {
        let kaomoji_area = Rect {
            x: inner.x + inner.width.saturating_sub(14),
            y: inner.y,
            width: 14,
            height: 3,
        };
        let widget =
            Paragraph::new(kaomoji).style(Style::default().fg(Color::Rgb(255, 220, 230)));
        frame.render_widget(widget, kaomoji_area);
    }
}

fn separator(width: u16) -> Paragraph<'static> {
    Paragraph::new("─".repeat(width as usize)).style(Style::default().fg(Color::Rgb(58, 46, 80)))
}

fn pad_lr(left: &str, right: &str, width: usize) -> String {
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    if left_len + right_len >= width {
        return truncate(&format!("{left} {right}"), width);
    }
    format!(
        "{left}{}{}",
        " ".repeat(width - left_len - right_len),
        right
    )
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }

    text.chars()
        .take(width.saturating_sub(1))
        .collect::<String>()
        + "…"
}
