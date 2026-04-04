use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use futures::FutureExt;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

use crate::{
    commands::package_attr_for_config,
    dango::{frame_at, DangoAnimation},
    options::{query_options, NixOption},
    packages::{query_packages, NixPackage},
};

use super::inline_search_widget::{self, InlineSearchRenderModel};

const INLINE_HEIGHT: u16 = 18;
const RESULT_ROWS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Packages,
    Options,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineSearchOutcome {
    InstallPackage(String),
    TryPackage(String),
    Copy {
        label: String,
        text: String,
    },
    AddOption {
        option_name: String,
        snippet: String,
    },
}

#[derive(Debug)]
pub struct SearchWidget {
    mode: SearchMode,
    query: String,
    package_results: Vec<NixPackage>,
    option_results: Vec<NixOption>,
    selected: usize,
    state: WidgetState,
    animate: bool,
    dango_pos: String,
    last_edit: Option<Instant>,
    search_task: Option<JoinHandle<Result<SearchBatch>>>,
    frame_tick: usize,
}

#[derive(Debug, Clone)]
enum SearchBatch {
    Packages(Vec<NixPackage>),
    Options(Vec<NixOption>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WidgetState {
    Loading,
    Browsing,
    Typing,
    Acting,
    Exiting,
    Error(String),
}

enum SearchControl {
    Continue,
    Finish(Option<InlineSearchOutcome>, WidgetState),
}

impl SearchWidget {
    pub fn new(
        mode: SearchMode,
        initial_query: impl Into<String>,
        animate: bool,
        dango_pos: impl Into<String>,
    ) -> Self {
        let query = initial_query.into();
        Self {
            mode,
            query: query.clone(),
            package_results: Vec::new(),
            option_results: Vec::new(),
            selected: 0,
            state: WidgetState::Browsing,
            animate,
            dango_pos: dango_pos.into(),
            last_edit: (!query.trim().is_empty())
                .then_some(Instant::now() - Duration::from_millis(250)),
            search_task: None,
            frame_tick: 0,
        }
    }

    pub async fn run(mut self) -> Result<Option<InlineSearchOutcome>> {
        let mut terminal = super::init_inline_terminal(INLINE_HEIGHT)?;
        let result = self.run_loop(&mut terminal).await;
        super::restore_inline_terminal(&mut terminal)?;
        result
    }

    async fn run_loop(
        &mut self,
        terminal: &mut super::TuiTerminal,
    ) -> Result<Option<InlineSearchOutcome>> {
        loop {
            if self.should_refresh() {
                self.start_search();
            }
            self.poll_search();
            self.frame_tick = self.frame_tick.wrapping_add(1);

            terminal.draw(|frame| {
                let model = self.render_model();
                inline_search_widget::render(frame, frame.size(), &model);
            })?;

            if event::poll(Duration::from_millis(50))? {
                let evt = event::read()?;
                if let Event::Key(key) = evt {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match self.handle_key(key.code)? {
                        SearchControl::Continue => {}
                        SearchControl::Finish(outcome, exit_state) => {
                            self.state = exit_state;
                            self.play_exit_frames(terminal).await?;
                            return Ok(outcome);
                        }
                    }
                }
            }
        }
    }

    async fn play_exit_frames(&mut self, terminal: &mut super::TuiTerminal) -> Result<()> {
        for _ in 0..4 {
            self.frame_tick = self.frame_tick.wrapping_add(1);
            terminal.draw(|frame| {
                let model = self.render_model();
                inline_search_widget::render(frame, frame.size(), &model);
            })?;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode) -> Result<SearchControl> {
        Ok(match code {
            KeyCode::Esc => SearchControl::Finish(None, WidgetState::Exiting),
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                SearchControl::Continue
            }
            KeyCode::Down => {
                if self.selected + 1 < self.result_count() {
                    self.selected += 1;
                }
                SearchControl::Continue
            }
            KeyCode::Char('i') => match self.mode {
                SearchMode::Packages => self
                    .selected_package()
                    .map(|pkg| {
                        SearchControl::Finish(
                            Some(InlineSearchOutcome::InstallPackage(package_ref(pkg))),
                            WidgetState::Acting,
                        )
                    })
                    .unwrap_or(SearchControl::Continue),
                SearchMode::Options => self
                    .selected_option()
                    .map(|option| {
                        SearchControl::Finish(
                            Some(InlineSearchOutcome::AddOption {
                                option_name: option.name.clone(),
                                snippet: snippet_for(option),
                            }),
                            WidgetState::Acting,
                        )
                    })
                    .unwrap_or(SearchControl::Continue),
            },
            KeyCode::Char('t') if self.mode == SearchMode::Packages => self
                .selected_package()
                .map(|pkg| {
                    SearchControl::Finish(
                        Some(InlineSearchOutcome::TryPackage(package_ref(pkg))),
                        WidgetState::Acting,
                    )
                })
                .unwrap_or(SearchControl::Continue),
            KeyCode::Char('c') => match self.mode {
                SearchMode::Packages => self
                    .selected_package()
                    .map(|pkg| {
                        let label = package_attr_for_config(&package_ref(pkg));
                        SearchControl::Finish(
                            Some(InlineSearchOutcome::Copy {
                                text: label.clone(),
                                label,
                            }),
                            WidgetState::Acting,
                        )
                    })
                    .unwrap_or(SearchControl::Continue),
                SearchMode::Options => self
                    .selected_option()
                    .map(|option| {
                        SearchControl::Finish(
                            Some(InlineSearchOutcome::Copy {
                                label: option.name.clone(),
                                text: snippet_for(option),
                            }),
                            WidgetState::Acting,
                        )
                    })
                    .unwrap_or(SearchControl::Continue),
            },
            KeyCode::Backspace => {
                self.query.pop();
                self.last_edit = Some(Instant::now());
                self.state = WidgetState::Typing;
                SearchControl::Continue
            }
            KeyCode::Char(ch) => {
                self.query.push(ch);
                self.last_edit = Some(Instant::now());
                self.state = WidgetState::Typing;
                SearchControl::Continue
            }
            _ => SearchControl::Continue,
        })
    }

    fn should_refresh(&self) -> bool {
        self.last_edit
            .is_some_and(|last| last.elapsed() >= Duration::from_millis(200))
    }

    fn start_search(&mut self) {
        if let Some(task) = &self.search_task {
            task.abort();
        }
        let query = self.query.trim().to_string();
        if query.is_empty() {
            self.package_results.clear();
            self.option_results.clear();
            self.selected = 0;
            self.state = WidgetState::Browsing;
            self.last_edit = None;
            self.search_task = None;
            return;
        }

        self.state = WidgetState::Loading;
        self.search_task = Some(tokio::spawn({
            let mode = self.mode;
            async move {
                match mode {
                    SearchMode::Packages => query_packages(&query).await.map(SearchBatch::Packages),
                    SearchMode::Options => query_options(&query).await.map(SearchBatch::Options),
                }
            }
        }));
        self.last_edit = None;
    }

    fn poll_search(&mut self) {
        let is_ready = self
            .search_task
            .as_ref()
            .is_some_and(|task| task.is_finished());
        if !is_ready {
            return;
        }

        let task = self.search_task.take().expect("search task present");
        let results = task.now_or_never().expect("finished task");
        match results {
            Ok(Ok(SearchBatch::Packages(items))) => {
                self.package_results = items;
                self.option_results.clear();
                self.selected = 0;
                self.state = if self.package_results.is_empty() {
                    WidgetState::Error(format!("nothing matched '{}'", self.query.trim()))
                } else {
                    WidgetState::Browsing
                };
            }
            Ok(Ok(SearchBatch::Options(items))) => {
                self.option_results = items;
                self.package_results.clear();
                self.selected = 0;
                self.state = if self.option_results.is_empty() {
                    WidgetState::Error(format!("nothing matched '{}'", self.query.trim()))
                } else {
                    WidgetState::Browsing
                };
            }
            Ok(Err(err)) => {
                self.package_results.clear();
                self.option_results.clear();
                self.selected = 0;
                self.state = WidgetState::Error(err.to_string());
            }
            Err(err) => {
                self.package_results.clear();
                self.option_results.clear();
                self.selected = 0;
                self.state = WidgetState::Error(err.to_string());
            }
        }
    }

    fn render_model(&self) -> InlineSearchRenderModel {
        InlineSearchRenderModel {
            header_left: format!("🔍 {}", self.query_with_cursor()),
            header_right: format!("{} results", self.result_count()),
            result_lines: self.render_result_lines(),
            detail_lines: self.render_detail_lines(),
            hints: self.hints().to_string(),
            dango: self
                .show_dango()
                .then_some(frame_at(self.dango_animation(), self.frame_tick)),
        }
    }

    fn render_result_lines(&self) -> Vec<String> {
        match &self.state {
            WidgetState::Loading => pad_lines(vec!["  searching...".to_string()], RESULT_ROWS),
            WidgetState::Error(_) if self.result_count() == 0 => {
                let line = match self.mode {
                    SearchMode::Packages => "  no packages found",
                    SearchMode::Options => "  no options found",
                };
                pad_lines(vec![line.to_string()], RESULT_ROWS)
            }
            _ => {
                let rows = match self.mode {
                    SearchMode::Packages => visible_slice(&self.package_results, self.selected)
                        .into_iter()
                        .enumerate()
                        .map(|(offset, pkg)| {
                            let absolute =
                                visible_offset(self.package_results.len(), self.selected) + offset;
                            let marker = if absolute == self.selected {
                                "▸"
                            } else {
                                " "
                            };
                            format!(
                                "  {marker} {}  {}  {}",
                                truncate(&display_name(pkg), 16),
                                truncate(pkg.version.as_deref().unwrap_or("?"), 8),
                                truncate(pkg.license.as_deref().unwrap_or("unknown"), 10)
                            )
                        })
                        .collect::<Vec<_>>(),
                    SearchMode::Options => visible_slice(&self.option_results, self.selected)
                        .into_iter()
                        .enumerate()
                        .map(|(offset, option)| {
                            let absolute =
                                visible_offset(self.option_results.len(), self.selected) + offset;
                            let marker = if absolute == self.selected {
                                "▸"
                            } else {
                                " "
                            };
                            format!("  {marker} {}", truncate(&option.name, 34))
                        })
                        .collect::<Vec<_>>(),
                };
                pad_lines(rows, RESULT_ROWS)
            }
        }
    }

    fn render_detail_lines(&self) -> Vec<String> {
        match self.mode {
            SearchMode::Packages => {
                if let Some(pkg) = self.selected_package() {
                    pad_lines(render_package_detail(pkg), 5)
                } else {
                    detail_for_state(&self.state, self.query.trim(), "package")
                }
            }
            SearchMode::Options => {
                if let Some(option) = self.selected_option() {
                    pad_lines(render_option_detail(option), 5)
                } else {
                    detail_for_state(&self.state, self.query.trim(), "option")
                }
            }
        }
    }

    fn query_with_cursor(&self) -> String {
        let cursor_on = (self.frame_tick / 10).is_multiple_of(2);
        let base = if self.query.is_empty() {
            String::new()
        } else {
            self.query.clone()
        };
        if cursor_on {
            format!("{base}█")
        } else {
            format!("{base} ")
        }
    }

    fn selected_package(&self) -> Option<&NixPackage> {
        self.package_results.get(self.selected)
    }

    fn selected_option(&self) -> Option<&NixOption> {
        self.option_results.get(self.selected)
    }

    fn result_count(&self) -> usize {
        match self.mode {
            SearchMode::Packages => self.package_results.len(),
            SearchMode::Options => self.option_results.len(),
        }
    }

    fn hints(&self) -> &'static str {
        match self.mode {
            SearchMode::Packages => "[i] install  [t] try  [c] copy  [↑↓] navigate  [esc] exit",
            SearchMode::Options => "[i] add to config  [c] copy snippet  [↑↓] navigate  [esc] exit",
        }
    }

    fn dango_animation(&self) -> DangoAnimation {
        match self.state {
            WidgetState::Loading => DangoAnimation::Spin,
            WidgetState::Acting => DangoAnimation::Happy,
            WidgetState::Exiting => DangoAnimation::Wave,
            WidgetState::Error(_) => DangoAnimation::Sad,
            WidgetState::Browsing | WidgetState::Typing => DangoAnimation::Idle,
        }
    }

    fn show_dango(&self) -> bool {
        self.animate && self.dango_pos != "off"
    }
}

fn render_package_detail(pkg: &NixPackage) -> Vec<String> {
    vec![
        format!(
            "{}  ·  {}  ·  {}  ·  {}",
            display_name(pkg),
            pkg.version.as_deref().unwrap_or("unknown"),
            pkg.license.as_deref().unwrap_or("unknown"),
            pkg.size.as_deref().unwrap_or("size ?")
        ),
        truncate(
            pkg.description
                .as_deref()
                .unwrap_or("no description from nix search yet"),
            68,
        ),
        truncate(pkg.long_description.as_deref().unwrap_or(""), 68),
        format!(
            "homepage   {}",
            truncate(pkg.homepage.as_deref().unwrap_or("—"), 57)
        ),
        format!("install    {}", package_attr_for_config(&package_ref(pkg))),
    ]
}

fn render_option_detail(option: &NixOption) -> Vec<String> {
    vec![
        truncate(&option.name, 72),
        format!(
            "type: {}  ·  default: {}",
            option.option_type.as_deref().unwrap_or("unknown"),
            truncate(option.default_value.as_deref().unwrap_or("—"), 32)
        ),
        truncate(
            option
                .description
                .as_deref()
                .unwrap_or("no description yet"),
            72,
        ),
        format!("example: {}", truncate(&snippet_for(option), 63)),
        format!(
            "source: {}",
            truncate(option.source.as_deref().unwrap_or("search.nixos.org"), 64)
        ),
    ]
}

fn detail_for_state(state: &WidgetState, query: &str, label: &str) -> Vec<String> {
    match state {
        WidgetState::Loading => pad_lines(vec![format!("searching {label}s...")], 5),
        WidgetState::Error(message) => pad_lines(
            vec![message.clone(), "try a shorter search  ♡".to_string()],
            5,
        ),
        _ => pad_lines(
            vec![format!("type to search for a {label}, like '{query}'")],
            5,
        ),
    }
}

fn snippet_for(option: &NixOption) -> String {
    if let Some(example) = &option.example {
        example.trim().trim_end_matches(';').to_string() + ";"
    } else if option.option_type.as_deref() == Some("boolean") {
        format!("{} = true;", option.name)
    } else {
        format!("{} = <value>;", option.name)
    }
}

fn visible_offset(total: usize, selected: usize) -> usize {
    if total <= RESULT_ROWS {
        0
    } else if selected >= total.saturating_sub(RESULT_ROWS / 2) {
        total - RESULT_ROWS
    } else {
        selected.saturating_sub(RESULT_ROWS / 2)
    }
}

fn visible_slice<T>(items: &[T], selected: usize) -> &[T] {
    let start = visible_offset(items.len(), selected);
    let end = (start + RESULT_ROWS).min(items.len());
    &items[start..end]
}

fn pad_lines(mut lines: Vec<String>, count: usize) -> Vec<String> {
    while lines.len() < count {
        lines.push(String::new());
    }
    lines.truncate(count);
    lines
}

fn display_name(pkg: &NixPackage) -> String {
    if pkg.name.is_empty() {
        package_ref(pkg)
    } else {
        pkg.name.clone()
    }
}

fn package_ref(pkg: &NixPackage) -> String {
    if !pkg.attribute.is_empty() {
        pkg.attribute.clone()
    } else {
        pkg.name.clone()
    }
}

fn truncate(text: &str, width: usize) -> String {
    let text = text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(text)
        .trim();
    if text.chars().count() <= width {
        return text.to_string();
    }
    text.chars()
        .take(width.saturating_sub(1))
        .collect::<String>()
        + "…"
}

#[cfg(test)]
mod tests {
    use super::{snippet_for, visible_offset, SearchMode, SearchWidget, WidgetState};
    use crate::{options::NixOption, packages::NixPackage};

    #[test]
    fn visible_offset_keeps_selection_in_window() {
        assert_eq!(visible_offset(10, 0), 0);
        assert_eq!(visible_offset(10, 3), 1);
        assert_eq!(visible_offset(10, 9), 5);
    }

    #[test]
    fn option_snippet_prefers_example() {
        let option = NixOption {
            name: "services.ollama.enable".to_string(),
            example: Some("services.ollama.enable = true;".to_string()),
            ..NixOption::default()
        };
        assert_eq!(snippet_for(&option), "services.ollama.enable = true;");
    }

    #[test]
    fn render_model_uses_query_results_count() {
        let mut widget = SearchWidget::new(SearchMode::Packages, "rip", true, "auto");
        widget.package_results = vec![NixPackage {
            attribute: "ripgrep".to_string(),
            ..NixPackage::default()
        }];
        widget.state = WidgetState::Browsing;
        let model = widget.render_model();
        assert!(model.header_right.contains("1 results"));
    }
}
