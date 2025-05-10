use crate::commands;
use crate::notes;
use crossterm::cursor;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Tabs, Wrap};
use ratatui::Frame;
use ratatui::Terminal;
use regex::Regex;
use std::fs;
use std::io;
use std::process::{Command, ExitStatus};
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum OverlayMode {
    None,
    CommandPalette,
    NoteCreation,
    Help,
    Search,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TabState {
    Notes,
    Preview,
    AI,
}

/// TUI application state.
pub struct AppState {
    pub notes: Vec<String>,
    pub filtered_notes: Vec<String>,
    pub selected_idx: usize,
    pub overlay: OverlayMode,
    pub overlay_input: String,
    pub last_ai_output: Option<String>,
    pub preview: Option<String>,
    pub status_message: Option<String>,
    pub active_tab: TabState,
    pub search_query: Option<String>,
    pub show_tags: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let notes_list = notes::load_notes_list();
        let mut s = Self {
            notes: notes_list.clone(),
            filtered_notes: notes_list,
            selected_idx: 0,
            overlay: OverlayMode::None,
            overlay_input: String::new(),
            last_ai_output: None,
            preview: None,
            status_message: None,
            active_tab: TabState::Notes,
            search_query: None,
            show_tags: false,
        };
        s.update_preview();
        s
    }

    pub fn next_note(&mut self) {
        let notes = if self.search_query.is_some() {
            &self.filtered_notes
        } else {
            &self.notes
        };

        if !notes.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % notes.len();
            self.update_preview();
        }
    }

    pub fn prev_note(&mut self) {
        let notes = if self.search_query.is_some() {
            &self.filtered_notes
        } else {
            &self.notes
        };

        if !notes.is_empty() {
            if self.selected_idx == 0 {
                self.selected_idx = notes.len() - 1;
            } else {
                self.selected_idx -= 1;
            }
            self.update_preview();
        }
    }

    pub fn selected_note(&self) -> Option<&String> {
        if self.search_query.is_some() {
            self.filtered_notes.get(self.selected_idx)
        } else {
            self.notes.get(self.selected_idx)
        }
    }

    pub fn update_preview(&mut self) {
        self.preview = None;
        if let Some(st) = self.selected_note() {
            let p = notes::note_path(st);
            if let Ok(content) = fs::read_to_string(p) {
                // Extract and parse markdown content for better preview
                let yaml_delim = Regex::new(r"^---\s*$").unwrap();
                let mut lines: Vec<&str> = content.lines().collect();

                // Remove YAML front matter if present
                if lines.len() > 2 && yaml_delim.is_match(lines[0]) {
                    if let Some(end_idx) = lines
                        .iter()
                        .skip(1)
                        .position(|line| yaml_delim.is_match(line))
                    {
                        lines = lines[end_idx + 2..].to_vec();
                    }
                }

                // Take more lines for a richer preview
                let preview_lines: Vec<_> = lines.into_iter().take(20).collect();
                self.preview = Some(preview_lines.join("\n"));
            }
        }
    }

    pub fn apply_search(&mut self) {
        if let Some(query) = &self.search_query {
            let query_lowercase = query.to_lowercase();
            self.filtered_notes = self
                .notes
                .iter()
                .filter(|note| {
                    // Match note name
                    let name_match = note.to_lowercase().contains(&query_lowercase);

                    // Match note content
                    let content_match = match fs::read_to_string(notes::note_path(note)) {
                        Ok(content) => content.to_lowercase().contains(&query_lowercase),
                        Err(_) => false,
                    };

                    name_match || content_match
                })
                .cloned()
                .collect();

            self.selected_idx = 0;
            self.update_preview();
        } else {
            self.filtered_notes = self.notes.clone();
        }
    }

    pub fn toggle_tab(&mut self) {
        self.active_tab = match self.active_tab {
            TabState::Notes => TabState::Preview,
            TabState::Preview => TabState::AI,
            TabState::AI => TabState::Notes,
        };
    }
}

/// Run the TUI.
pub fn run_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = (|| {
        let mut st = AppState::new();
        loop {
            terminal.draw(|f| ui(f, &st))?;

            if event::poll(Duration::from_millis(50))? {
                let ev = event::read()?;
                match ev {
                    Event::Key(KeyEvent {
                        code, modifiers, ..
                    }) => {
                        if code == KeyCode::Char('q')
                            && modifiers == KeyModifiers::NONE
                            && st.overlay == OverlayMode::None
                        {
                            break;
                        }
                        match st.overlay {
                            OverlayMode::None => match code {
                                KeyCode::Char('j') | KeyCode::Down => st.next_note(),
                                KeyCode::Char('k') | KeyCode::Up => st.prev_note(),
                                KeyCode::Char('n') => {
                                    st.overlay = OverlayMode::NoteCreation;
                                    st.overlay_input.clear();
                                }
                                KeyCode::Char('e') => {
                                    if let Some(sn) = st.selected_note() {
                                        let ed_result = open_in_editor(sn);
                                        match ed_result {
                                            Ok(_exit_status) => {
                                                st.status_message =
                                                    Some(format!("Edited note: {}", sn));
                                                execute!(io::stdout(), Clear(ClearType::All))?;
                                                st.update_preview();
                                            }
                                            Err(e) => {
                                                st.status_message =
                                                    Some(format!("Editor error: {}", e));
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char(':') => {
                                    st.overlay = OverlayMode::CommandPalette;
                                    st.overlay_input.clear();
                                }
                                KeyCode::Char('h') => {
                                    st.overlay = OverlayMode::Help;
                                }
                                KeyCode::Char('/') => {
                                    st.overlay = OverlayMode::Search;
                                    st.overlay_input.clear();
                                }
                                KeyCode::Tab => st.toggle_tab(),
                                KeyCode::Char('t') => {
                                    st.show_tags = !st.show_tags;
                                    st.status_message = Some(format!(
                                        "Tags display: {}",
                                        if st.show_tags { "on" } else { "off" }
                                    ));
                                }
                                KeyCode::Char('r') => {
                                    st.notes = notes::load_notes_list();
                                    if let Some(_query) = &st.search_query {
                                        st.apply_search();
                                    } else {
                                        st.filtered_notes = st.notes.clone();
                                    }
                                    st.update_preview();
                                    st.status_message = Some("Notes refreshed".to_string());
                                }
                                _ => {}
                            },
                            OverlayMode::CommandPalette => match code {
                                KeyCode::Esc => st.overlay = OverlayMode::None,
                                KeyCode::Enter => {
                                    let cmd = st.overlay_input.clone();
                                    if let Err(e) = commands::handle_cmd(cmd, &mut st) {
                                        st.last_ai_output = Some(format!("Error: {}", e));
                                    }
                                    st.overlay_input.clear();
                                    st.overlay = OverlayMode::None;
                                }
                                KeyCode::Backspace => {
                                    st.overlay_input.pop();
                                }
                                KeyCode::Char('\t') => {
                                    let partial = st.overlay_input.trim_start_matches(':');
                                    let matches: Vec<_> = ["summarize", "keywords"]
                                        .iter()
                                        .filter(|x| x.starts_with(partial))
                                        .collect();
                                    if matches.len() == 1 {
                                        st.overlay_input = format!(":{}", matches[0]);
                                    }
                                }
                                KeyCode::Char(c) => {
                                    st.overlay_input.push(c);
                                }
                                _ => {}
                            },
                            OverlayMode::NoteCreation => match code {
                                KeyCode::Esc => {
                                    st.overlay = OverlayMode::None;
                                    st.overlay_input.clear();
                                }
                                KeyCode::Backspace => {
                                    st.overlay_input.pop();
                                }
                                KeyCode::Enter => {
                                    let title = st.overlay_input.trim().to_string();
                                    if !title.is_empty() {
                                        match notes::create_new_note(&title) {
                                            Ok(_) => {
                                                st.notes = notes::load_notes_list();
                                                st.filtered_notes = st.notes.clone();
                                                st.selected_idx = 0;
                                                st.update_preview();
                                                st.status_message =
                                                    Some(format!("Note created: {}", title));
                                            }
                                            Err(e) => {
                                                st.status_message = Some(format!("Error: {}", e));
                                            }
                                        }
                                    }
                                    st.overlay = OverlayMode::None;
                                    st.overlay_input.clear();
                                }
                                KeyCode::Char(c) => {
                                    st.overlay_input.push(c);
                                }
                                _ => {}
                            },
                            OverlayMode::Search => match code {
                                KeyCode::Esc => {
                                    st.overlay = OverlayMode::None;
                                    st.overlay_input.clear();
                                }
                                KeyCode::Enter => {
                                    let query = st.overlay_input.trim().to_string();
                                    if query.is_empty() {
                                        st.search_query = None;
                                        st.filtered_notes = st.notes.clone();
                                    } else {
                                        st.search_query = Some(query);
                                        st.apply_search();
                                    }
                                    st.selected_idx = 0;
                                    st.overlay = OverlayMode::None;
                                    st.overlay_input.clear();
                                    st.update_preview();
                                }
                                KeyCode::Backspace => {
                                    st.overlay_input.pop();
                                }
                                KeyCode::Char(c) => {
                                    st.overlay_input.push(c);
                                }
                                _ => {}
                            },
                            OverlayMode::Help => match code {
                                KeyCode::Esc | KeyCode::Char('h') => {
                                    st.overlay = OverlayMode::None;
                                }
                                _ => {}
                            },
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

/// Open the note in an external editor.
pub fn open_in_editor(stem: &str) -> io::Result<ExitStatus> {
    // Save the terminal state
    let mut stdout = io::stdout();
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    // Clear the screen to avoid leftovers
    execute!(stdout, Clear(ClearType::All))?;

    // Launch the editor
    let ed = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());
    let p = notes::note_path(stem);
    let mut child = Command::new(ed).arg(p).spawn()?;
    let exit_status = child.wait()?;

    // Properly restore the terminal state
    execute!(stdout, Clear(ClearType::All))?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    // Ensure cursor visibility is reset
    execute!(stdout, cursor::Show)?;

    Ok(exit_status)
}

/// Draw the entire UI.
pub fn ui<B: ratatui::backend::Backend>(f: &mut Frame<B>, st: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top bar with tabs
            Constraint::Min(5),    // Main content
            Constraint::Length(3), // Footer
            Constraint::Length(1), // Status bar
        ])
        .split(f.size());

    // Render top tabs
    let tab_titles = vec![
        Spans::from("Notes"),
        Spans::from("Preview"),
        Spans::from("AI"),
    ];
    let tabs = Tabs::new(tab_titles)
        .select(match st.active_tab {
            TabState::Notes => 0,
            TabState::Preview => 1,
            TabState::AI => 2,
        })
        .style(Style::default())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )
        .divider(Span::raw("|"))
        .block(Block::default().borders(Borders::ALL).title("Tabs"));
    f.render_widget(tabs, chunks[0]);

    // Main content layout
    let main_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[1]);

    // Render note list (always visible)
    let notes_to_display = if st.search_query.is_some() {
        &st.filtered_notes
    } else {
        &st.notes
    };

    let items: Vec<ListItem> = notes_to_display
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let display_text = text.replace('_', " ");
            let style = if i == st.selected_idx {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Show search status if applicable

            ListItem::new(display_text).style(style)
        })
        .collect();

    let list_title = if let Some(query) = &st.search_query {
        format!("Notes (filtered: {})", query)
    } else {
        "Notes".to_string()
    };

    let left = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(list_title),
    );
    f.render_widget(left, main_split[0]);

    // Render right panel based on active tab
    match st.active_tab {
        TabState::Notes => {
            // If in Notes tab, show a more detailed view of the selected note
            if let Some(note) = st.selected_note() {
                let note_path = notes::note_path(note);
                if let Ok(content) = fs::read_to_string(note_path) {
                    // Create a nicer display with YAML front matter parsed
                    let yaml_delim = Regex::new(r"^---\s*$").unwrap();
                    let mut lines: Vec<&str> = content.lines().collect();

                    let mut title = note.replace('_', " ");
                    let mut tags = Vec::new();

                    // Extract title and tags from YAML if present
                    if lines.len() > 2 && yaml_delim.is_match(lines[0]) {
                        if let Some(end_idx) = lines
                            .iter()
                            .skip(1)
                            .position(|line| yaml_delim.is_match(line))
                        {
                            let yaml_section = &lines[1..end_idx + 1];
                            for line in yaml_section {
                                if line.starts_with("title:") {
                                    title = line.trim_start_matches("title:").trim().to_string();
                                }
                                if line.starts_with("tags:") {
                                    let tags_str = line.trim_start_matches("tags:").trim();
                                    let tags_str =
                                        tags_str.trim_start_matches('[').trim_end_matches(']');
                                    tags =
                                        tags_str.split(',').map(|s| s.trim().to_string()).collect();
                                }
                            }
                            lines = lines[end_idx + 2..].to_vec();
                        }
                    }

                    // Create spans with title and tags highlighted
                    let mut text_spans = Vec::new();

                    // Add title
                    text_spans.push(Spans::from(vec![
                        Span::styled("Title: ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                            title,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));

                    // Add tags if any
                    if !tags.is_empty() && st.show_tags {
                        text_spans.push(Spans::from(vec![
                            Span::styled("Tags: ", Style::default().fg(Color::Yellow)),
                            Span::styled(tags.join(", "), Style::default().fg(Color::Cyan)),
                        ]));
                    }

                    // Add a separator
                    text_spans.push(Spans::from(""));

                    // Add the note content
                    for line in lines {
                        text_spans.push(Spans::from(line));
                    }

                    let note_details = Paragraph::new(text_spans)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_type(BorderType::Rounded)
                                .title("Note Details"),
                        )
                        .wrap(Wrap { trim: false });

                    f.render_widget(note_details, main_split[1]);
                }
            } else {
                let no_note = Paragraph::new("No note selected.").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Note Details"),
                );
                f.render_widget(no_note, main_split[1]);
            }
        }
        TabState::Preview => {
            // Show preview of the note content
            let lines = if let Some(prev) = &st.preview {
                let text_lines: Vec<Spans> = prev.lines().map(Spans::from).collect();
                text_lines
            } else {
                vec![Spans::from("No note selected.")]
            };

            let preview = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Preview"),
                )
                .wrap(Wrap { trim: false });

            f.render_widget(preview, main_split[1]);
        }
        TabState::AI => {
            // Show AI output or guidance
            let ai_content = if let Some(ai) = &st.last_ai_output {
                ai.clone()
            } else {
                "No AI output. Use :summarize or :keywords command.".to_string()
            };

            let ai_para = Paragraph::new(ai_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("AI Analysis"),
                )
                .wrap(Wrap { trim: false });

            f.render_widget(ai_para, main_split[1]);
        }
    }

    // Enhanced footer with more keyboard shortcuts
    let foot_sp = Spans::from(vec![
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":Quit  "),
        Span::styled("↑/↓/j/k", Style::default().fg(Color::Yellow)),
        Span::raw(":Select  "),
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(":NewNote  "),
        Span::styled("e", Style::default().fg(Color::Yellow)),
        Span::raw(":Edit  "),
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::raw(":Search  "),
        Span::styled(":", Style::default().fg(Color::Yellow)),
        Span::raw(":Cmd  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(":SwitchTab  "),
        Span::styled("t", Style::default().fg(Color::Yellow)),
        Span::raw(":Tags  "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(":Refresh  "),
        Span::styled("h", Style::default().fg(Color::Yellow)),
        Span::raw(":Help"),
    ]);

    let foot = Paragraph::new(foot_sp).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Keyboard Shortcuts"),
    );
    f.render_widget(foot, chunks[2]);

    // Status bar with search/filter indicator
    let mut status_parts = Vec::new();

    // Add note count info
    let note_count_text = if st.search_query.is_some() {
        format!("{}/{} notes", st.filtered_notes.len(), st.notes.len())
    } else {
        format!("{} notes", st.notes.len())
    };

    status_parts.push(Span::raw(note_count_text));
    status_parts.push(Span::raw(" | "));

    // Add status message if any
    if let Some(msg) = &st.status_message {
        status_parts.push(Span::styled(msg, Style::default().fg(Color::Green)));
    } else {
        status_parts.push(Span::raw("Ready"));
    }

    let status_txt = Spans::from(status_parts);
    let status_par = Paragraph::new(status_txt)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title(""),
        )
        .alignment(Alignment::Left);
    f.render_widget(status_par, chunks[3]);

    // Draw overlays last so they appear on top
    match st.overlay {
        OverlayMode::None => {}
        OverlayMode::CommandPalette => {
            draw_overlay(f, "[Command]", &st.overlay_input, main_split[1])
        }
        OverlayMode::NoteCreation => {
            draw_overlay(f, "[New Note Title]", &st.overlay_input, main_split[1])
        }
        OverlayMode::Search => draw_overlay(f, "[Search]", &st.overlay_input, main_split[1]),
        OverlayMode::Help => draw_help_overlay(f, main_split[1]),
    }
}

/// Draw a generic overlay.
pub fn draw_overlay<B: ratatui::backend::Backend>(
    f: &mut Frame<B>,
    title: &str,
    content: &str,
    area: Rect,
) {
    let h = 3;
    let w = area.width.saturating_sub(4);
    let overlay_rect = Rect {
        x: area.x + 2,
        y: area.y + area.height.saturating_sub(h) - 2,
        width: w,
        height: h,
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title);
    let par = Paragraph::new(content.to_string())
        .block(block)
        .style(Style::default().fg(Color::White));
    f.render_widget(par, overlay_rect);
}

/// Draw the help overlay.
pub fn draw_help_overlay<B: ratatui::backend::Backend>(f: &mut Frame<B>, area: Rect) {
    // Center the help modal in the content area
    let w = area.width.saturating_sub(10).min(80); // Max width of 80
    let h = area.height.saturating_sub(6).min(20); // Max height of 20

    // Center the help box in the area
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;

    let overlay_rect = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    // Create visually enhanced help content
    let lines = vec![
        Spans::from(vec![Span::styled(
            "⌨️  Keyboard Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(""),
        Spans::from(vec![Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
        )]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("↑/↓", Style::default().fg(Color::Green)),
            Span::raw(" or "),
            Span::styled("j/k", Style::default().fg(Color::Green)),
            Span::raw(": Move between notes"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("Tab", Style::default().fg(Color::Green)),
            Span::raw(": Switch between Notes/Preview/AI tabs"),
        ]),
        Spans::from(""),
        Spans::from(vec![Span::styled(
            "Note Management",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
        )]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("n", Style::default().fg(Color::Green)),
            Span::raw(": Create new note"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("e", Style::default().fg(Color::Green)),
            Span::raw(": Edit current note in $EDITOR"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("r", Style::default().fg(Color::Green)),
            Span::raw(": Refresh note list"),
        ]),
        Spans::from(""),
        Spans::from(vec![Span::styled(
            "Features",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
        )]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("/", Style::default().fg(Color::Green)),
            Span::raw(": Search notes (by title and content)"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled(":", Style::default().fg(Color::Green)),
            Span::raw(": Command palette (AI commands: "),
            Span::styled(":summarize", Style::default().fg(Color::Magenta)),
            Span::raw(", "),
            Span::styled(":keywords", Style::default().fg(Color::Magenta)),
            Span::raw(")"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("t", Style::default().fg(Color::Green)),
            Span::raw(": Toggle tag display"),
        ]),
        Spans::from(""),
        Spans::from(vec![Span::styled(
            "General",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
        )]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("h", Style::default().fg(Color::Green)),
            Span::raw(": Toggle this help (press "),
            Span::styled("Esc", Style::default().fg(Color::Green)),
            Span::raw(" or "),
            Span::styled("h", Style::default().fg(Color::Green)),
            Span::raw(" to close)"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("q", Style::default().fg(Color::Green)),
            Span::raw(": Quit the TUI"),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default())
        .title(Span::styled(
            " Help ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let par = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    // Create a slightly transparent background
    f.render_widget(par, overlay_rect);
}
