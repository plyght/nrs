use std::io;
use std::time::Duration;
use std::fs;
use std::process::{Command, ExitStatus};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use crate::notes;
use crate::commands;

#[derive(Debug, PartialEq)]
pub enum OverlayMode {
    None,
    CommandPalette,
    NoteCreation,
    Help,
}

/// TUI application state.
pub struct AppState {
    pub notes: Vec<String>,
    pub selected_idx: usize,
    pub overlay: OverlayMode,
    pub overlay_input: String,
    pub last_ai_output: Option<String>,
    pub preview: Option<String>,
    pub status_message: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        let mut s = Self {
            notes: notes::load_notes_list(),
            selected_idx: 0,
            overlay: OverlayMode::None,
            overlay_input: String::new(),
            last_ai_output: None,
            preview: None,
            status_message: None,
        };
        s.update_preview();
        s
    }

    pub fn next_note(&mut self) {
        if !self.notes.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.notes.len();
            self.update_preview();
        }
    }

    pub fn prev_note(&mut self) {
        if !self.notes.is_empty() {
            if self.selected_idx == 0 {
                self.selected_idx = self.notes.len() - 1;
            } else {
                self.selected_idx -= 1;
            }
            self.update_preview();
        }
    }

    pub fn selected_note(&self) -> Option<&String> {
        self.notes.get(self.selected_idx)
    }

    pub fn update_preview(&mut self) {
        self.preview = None;
        if let Some(st) = self.selected_note() {
            let p = notes::note_path(st);
            if let Ok(content) = fs::read_to_string(p) {
                let lines: Vec<_> = content.lines().take(10).collect();
                self.preview = Some(lines.join("\n"));
            }
        }
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
                    Event::Key(KeyEvent { code, modifiers, .. }) => {
                        if code == KeyCode::Char('q') && modifiers == KeyModifiers::NONE {
                            break;
                        }
                        match st.overlay {
                            OverlayMode::None => {
                                match code {
                                    KeyCode::Char('j') => st.next_note(),
                                    KeyCode::Char('k') => st.prev_note(),
                                    KeyCode::Char('n') => {
                                        st.overlay = OverlayMode::NoteCreation;
                                        st.overlay_input.clear();
                                    }
                                    KeyCode::Char('e') => {
                                        if let Some(sn) = st.selected_note() {
                                            let ed_result = open_in_editor(sn);
                                            match ed_result {
                                                Ok(exit_status) => {
                                                    st.status_message = Some(format!("Editor exit code: {:?}", exit_status));
                                                    execute!(io::stdout(), Clear(ClearType::All))?;
                                                    st.update_preview();
                                                }
                                                Err(e) => {
                                                    st.status_message = Some(format!("Editor error: {}", e));
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
                                    _ => {}
                                }
                            }
                            OverlayMode::CommandPalette => {
                                match code {
                                    KeyCode::Esc => st.overlay = OverlayMode::None,
                                    KeyCode::Enter => {
                                        let cmd = st.overlay_input.clone();
                                        if let Err(e) = commands::handle_cmd(cmd, &mut st) {
                                            st.last_ai_output = Some(format!("Error: {}", e));
                                        }
                                        st.overlay_input.clear();
                                        st.overlay = OverlayMode::None;
                                    }
                                    KeyCode::Backspace => { st.overlay_input.pop(); },
                                    KeyCode::Char('\t') => {
                                        let partial = st.overlay_input.trim_start_matches(':');
                                        let matches: Vec<_> = ["summarize", "keywords"].iter().filter(|x| x.starts_with(partial)).collect();
                                        if matches.len() == 1 {
                                            st.overlay_input = format!(":{}", matches[0]);
                                        }
                                    }
                                    KeyCode::Char(c) => { st.overlay_input.push(c); },
                                    _ => {}
                                }
                            }
                            OverlayMode::NoteCreation => {
                                match code {
                                    KeyCode::Esc => {
                                        st.overlay = OverlayMode::None;
                                        st.overlay_input.clear();
                                    }
                                    KeyCode::Backspace => { st.overlay_input.pop(); },
                                    KeyCode::Enter => {
                                        let title = st.overlay_input.trim().to_string();
                                        if !title.is_empty() {
                                            match notes::create_new_note(&title) {
                                                Ok(_) => {
                                                    st.notes = notes::load_notes_list();
                                                    st.selected_idx = 0;
                                                    st.update_preview();
                                                    st.status_message = Some(format!("Note created: {}", title));
                                                }
                                                Err(e) => {
                                                    st.status_message = Some(format!("Error: {}", e));
                                                }
                                            }
                                        }
                                        st.overlay = OverlayMode::None;
                                        st.overlay_input.clear();
                                    }
                                    KeyCode::Char(c) => { st.overlay_input.push(c); },
                                    _ => {}
                                }
                            }
                            OverlayMode::Help => {
                                match code {
                                    KeyCode::Esc | KeyCode::Char('h') => { st.overlay = OverlayMode::None; },
                                    _ => {}
                                }
                            }
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
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    let ed = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());
    let p = notes::note_path(stem);
    let mut child = Command::new(ed).arg(p).spawn()?;
    let exit_status = child.wait()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Ok(exit_status)
}

/// Draw the entire UI.
pub fn ui<B: ratatui::backend::Backend>(f: &mut Frame<B>, st: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.size());

    let main_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(30)].as_ref())
        .split(chunks[0]);

    let items: Vec<ListItem> = st.notes.iter().enumerate().map(|(i, text)| {
        let style = if i == st.selected_idx {
            Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(text.clone()).style(style)
    }).collect();

    let left = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("~/notes (Press 'h' for help)"));
    f.render_widget(left, main_split[0]);

    let lines = if let Some(ai) = &st.last_ai_output {
        vec![Spans::from(ai.as_str())]
    } else {
        if let Some(prev) = &st.preview {
            prev.lines().map(|l| Spans::from(l)).collect()
        } else {
            vec![Spans::from("No note selected.")]
        }
    };

    let right_par = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Preview / AI"));
    f.render_widget(right_par, main_split[1]);

    let foot_sp = Spans::from(vec![
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":Quit  "),
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(":Select  "),
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(":NewNote  "),
        Span::styled("e", Style::default().fg(Color::Yellow)),
        Span::raw(":Edit  "),
        Span::styled(":", Style::default().fg(Color::Yellow)),
        Span::raw(":Cmd  "),
        Span::styled("h", Style::default().fg(Color::Yellow)),
        Span::raw(":Help"),
    ]);

    let foot = Paragraph::new(foot_sp)
        .block(Block::default().borders(Borders::ALL).title("Footer"));
    f.render_widget(foot, chunks[1]);

    let status_txt = if let Some(msg) = &st.status_message { msg.clone() } else { " ".to_string() };
    let status_par = Paragraph::new(status_txt)
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status_par, chunks[2]);

    match st.overlay {
        OverlayMode::None => {}
        OverlayMode::CommandPalette => draw_overlay(f, "[Command]", &st.overlay_input, main_split[1]),
        OverlayMode::NoteCreation => draw_overlay(f, "[New Note Title]", &st.overlay_input, main_split[1]),
        OverlayMode::Help => draw_help_overlay(f, main_split[1]),
    }
}

/// Draw a generic overlay.
pub fn draw_overlay<B: ratatui::backend::Backend>(f: &mut Frame<B>, title: &str, content: &str, area: Rect) {
    let h = 3;
    let w = area.width.saturating_sub(4);
    let overlay_rect = Rect {
        x: area.x + 2,
        y: area.y + area.height.saturating_sub(h) - 2,
        width: w,
        height: h,
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let par = Paragraph::new(content.to_string()).block(block);
    f.render_widget(par, overlay_rect);
}

/// Draw the help overlay.
pub fn draw_help_overlay<B: ratatui::backend::Backend>(f: &mut Frame<B>, area: Rect) {
    let w = area.width.saturating_sub(8);
    let h = area.height.saturating_sub(4);
    let x = area.x + 4;
    let y = area.y + 2;
    let overlay_rect = Rect { x, y, width: w, height: h };
    let lines = vec![
        Spans::from("Keyboard Shortcuts:"),
        Spans::from("  j/k: Move selection"),
        Spans::from("  n: Create new note (title overlay)"),
        Spans::from("  e: Edit current note in $EDITOR; upon exit, TUI re-draws"),
        Spans::from("  : Summarize or keywords commands"),
        Spans::from("  h: Toggle this help overlay"),
        Spans::from("  q: Quit the TUI"),
        Spans::from("Press ESC or 'h' again to close this help"),
    ];
    let block = Block::default().borders(Borders::ALL).title("Help");
    let par = Paragraph::new(lines).block(block);
    f.render_widget(par, overlay_rect);
}