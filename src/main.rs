use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use actix_files::NamedFile;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use clap::{Parser, Subcommand};
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

// TUI imports
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::{Frame, Terminal};

// Async OpenAI integration (v0.10.0)
use async_openai::Client;
use async_openai::types::{
    ChatCompletionRequestMessage,
    CreateChatCompletionRequestArgs,
    Role,
};

const NOTES_DIR: &str = "notes";

// -------------------------
// CLI & Main Setup
// -------------------------

#[derive(Parser, Debug)]
#[command(name = "nrs", version = "0.3.0", about = "Rust-based TUI & Web for Notes")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new note
    New {
        /// Title of the note
        title: String,
    },
    /// Run the TUI interface
    Tui,
    /// Run the web server
    Serve {
        /// Port for the web server
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

/// Metadata for the graph view (web only)
#[derive(Debug, Serialize)]
struct NoteNode {
    id: String,
}

#[derive(Debug, Serialize)]
struct NoteLink {
    source: String,
    target: String,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // Ensure NOTES_DIR exists
    if !Path::new(NOTES_DIR).exists() {
        fs::create_dir(NOTES_DIR).map_err(|e| {
            eprintln!("Failed to create notes directory: {}", e);
            e
        })?;
    }

    match cli.command {
        Commands::New { title } => {
            if let Err(e) = create_new_note(&title) {
                eprintln!("Error creating note: {}", e);
            }
        }
        Commands::Tui => {
            if let Err(e) = run_tui() {
                eprintln!("Error in TUI: {}", e);
            }
        }
        Commands::Serve { port } => {
            if let Err(e) = serve_notes(port).await {
                eprintln!("Error starting server: {}", e);
            }
        }
    }
    Ok(())
}

/// Create a new Markdown note
fn create_new_note(title: &str) -> io::Result<()> {
    let filename = title.to_lowercase().replace(' ', "_").replace("/", "_").replace("\\", "_");
    let filepath = format!("{}/{}.md", NOTES_DIR, filename);
    if Path::new(&filepath).exists() {
        eprintln!("Note already exists: {}", filepath);
        return Ok(());
    }
    let mut file = fs::File::create(&filepath)?;
    let content = format!("# {}\n\nWrite your note here.\n", title);
    file.write_all(content.as_bytes())?;
    println!("Created new note: {}", filepath);
    Ok(())
}

// -------------------------
// Web Server Implementation
// -------------------------

async fn serve_notes(port: u16) -> io::Result<()> {
    println!("Starting server on http://127.0.0.1:{}", port);
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index_page))
            .route("/notes/{name}", web::get().to(serve_note))
            .route("/graph", web::get().to(graph_page))
            .route("/graph-data", web::get().to(graph_data))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

async fn index_page() -> impl Responder {
    let mut list_html = String::new();
    for entry in WalkDir::new(NOTES_DIR)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "md" {
                let fname = path.file_stem().unwrap().to_string_lossy();
                let display_name = fname.replace("_", " ");
                list_html.push_str(&format!(
                    "<li><a href=\"/notes/{0}\">{1}</a></li>",
                    fname, display_name
                ));
            }
        }
    }
    let html = format!(
r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>NRS - Notes</title>
</head>
<body>
  <h1>All Notes</h1>
  <ul>
    {list_html}
  </ul>
  <a href="/graph">View Graph</a>
</body>
</html>"##,
        list_html = list_html
    );
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

async fn serve_note(path: web::Path<String>) -> actix_web::Result<NamedFile> {
    let filename = format!("{}/{}.md", NOTES_DIR, path.into_inner());
    NamedFile::open(filename).map_err(Into::into)
}

async fn graph_page() -> impl Responder {
    let html = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>NRS - Graph</title>
  <script src="https://d3js.org/d3.v7.min.js"></script>
</head>
<body>
  <h1>Notes Graph</h1>
  <div id="graph"></div>
  <script>
    fetch('/graph-data')
      .then(response => response.json())
      .then(data => {
        const width = 800, height = 600;
        const svg = d3.select("#graph")
          .append("svg")
          .attr("width", width)
          .attr("height", height);
        const simulation = d3.forceSimulation(data.nodes)
          .force("link", d3.forceLink(data.links).id(d => d.id).distance(100))
          .force("charge", d3.forceManyBody().strength(-200))
          .force("center", d3.forceCenter(width / 2, height / 2));
        const link = svg.selectAll("line")
          .data(data.links)
          .enter().append("line")
          .style("stroke", "#999")
          .style("stroke-width", 2);
        const node = svg.selectAll("circle")
          .data(data.nodes)
          .enter().append("circle")
          .attr("r", 8)
          .attr("fill", "steelblue")
          .call(d3.drag()
            .on("start", (event, d) => {
              if (!event.active) simulation.alphaTarget(0.3).restart();
              d.fx = d.x; d.fy = d.y;
            })
            .on("drag", (event, d) => {
              d.fx = event.x; d.fy = event.y;
            })
            .on("end", (event, d) => {
              if (!event.active) simulation.alphaTarget(0);
              d.fx = null; d.fy = null;
            }));
        const label = svg.selectAll("text")
          .data(data.nodes)
          .enter().append("text")
          .text(d => d.id)
          .attr("font-size", "12px")
          .attr("dx", 12)
          .attr("dy", "0.35em");
        node.on("click", (event, d) => {
          const noteName = d.id.replace(' ', '_');
          window.location.href = "/notes/" + noteName;
        });
        simulation.on("tick", () => {
          link.attr("x1", d => d.source.x)
              .attr("y1", d => d.source.y)
              .attr("x2", d => d.target.x)
              .attr("y2", d => d.target.y);
          node.attr("cx", d => d.x)
              .attr("cy", d => d.y);
          label.attr("x", d => d.x)
               .attr("y", d => d.y);
        });
      });
  </script>
</body>
</html>"##;
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

async fn graph_data() -> impl Responder {
    let (nodes, links) = build_graph_data();
    let json_data = serde_json::json!({ "nodes": nodes, "links": links });
    HttpResponse::Ok().json(json_data)
}

fn build_graph_data() -> (Vec<NoteNode>, Vec<NoteLink>) {
    let link_pattern = Regex::new(r"\[\[(.+?)\]\]").unwrap();
    let mut adjacency = Vec::new();
    for entry in WalkDir::new(NOTES_DIR)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            let content = fs::read_to_string(path).unwrap_or_default();
            let mut found_links = Vec::new();
            for caps in link_pattern.captures_iter(&content) {
                if let Some(linked) = caps.get(1) {
                    let cleaned = linked.as_str().to_lowercase().replace(' ', "_");
                    found_links.push(cleaned);
                }
            }
            adjacency.push((stem, found_links));
        }
    }
    let mut node_set = std::collections::HashSet::new();
    let mut links = Vec::new();
    for (src, link_list) in &adjacency {
        node_set.insert(src.clone());
        for l in link_list {
            node_set.insert(l.clone());
        }
    }
    let nodes = node_set.into_iter().map(|id| NoteNode { id }).collect();
    for (src, link_list) in adjacency {
        for l in link_list {
            links.push(NoteLink { source: src.clone(), target: l });
        }
    }
    (nodes, links)
}

// -------------------------
// TUI Implementation
// -------------------------

struct AppState {
    notes: Vec<String>,
    selected_idx: usize,
    show_command_palette: bool,
    command_input: String,
    last_ai_output: Option<String>,
}

impl AppState {
    fn new() -> Self {
        Self {
            notes: load_notes_list(),
            selected_idx: 0,
            show_command_palette: false,
            command_input: String::new(),
            last_ai_output: None,
        }
    }
    fn next_note(&mut self) {
        if !self.notes.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.notes.len();
        }
    }
    fn prev_note(&mut self) {
        if !self.notes.is_empty() {
            if self.selected_idx == 0 {
                self.selected_idx = self.notes.len() - 1;
            } else {
                self.selected_idx -= 1;
            }
        }
    }
    fn selected_note(&self) -> Option<&String> {
        self.notes.get(self.selected_idx)
    }
}

fn run_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app_state = AppState::new();

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || loop {
        if event::poll(Duration::from_millis(200)).unwrap() {
            if let Ok(ev) = event::read() {
                tx.send(ev).unwrap();
            }
        }
    });

    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app_state))?;
        let timeout = Duration::from_millis(250).checked_sub(last_tick.elapsed()).unwrap_or(Duration::from_secs(0));
        if let Ok(ev) = rx.recv_timeout(timeout) {
            if let Event::Key(KeyEvent { code, modifiers: KeyModifiers::NONE, .. }) = ev {
                if app_state.show_command_palette {
                    match code {
                        KeyCode::Esc => app_state.show_command_palette = false,
                        KeyCode::Enter => {
                            let cmd = app_state.command_input.clone();
                            if let Err(e) = handle_command_palette(cmd, &mut app_state) {
                                app_state.last_ai_output = Some(format!("Error: {}", e));
                            }
                            app_state.command_input.clear();
                            app_state.show_command_palette = false;
                        }
                        KeyCode::Backspace => { app_state.command_input.pop(); }
                        KeyCode::Char(c) => { app_state.command_input.push(c); }
                        _ => {}
                    }
                } else {
                    match code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') => app_state.next_note(),
                        KeyCode::Char('k') => app_state.prev_note(),
                        KeyCode::Char('n') => {
                            if let Err(e) = prompt_create_note() {
                                eprintln!("Error creating note: {}", e);
                            }
                            app_state.notes = load_notes_list();
                            app_state.selected_idx = 0;
                        }
                        KeyCode::Char('e') => {
                            if let Some(note) = app_state.selected_note() {
                                if let Err(e) = open_in_editor(note) {
                                    eprintln!("Editor error: {}", e);
                                }
                            }
                        }
                        KeyCode::Char(':') => {
                            app_state.show_command_palette = true;
                            app_state.command_input.clear();
                        }
                        _ => {}
                    }
                }
            }
        }
        if last_tick.elapsed() >= Duration::from_millis(250) {
            last_tick = Instant::now();
        }
    }
    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui<B: ratatui::backend::Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
        .split(f.size());
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(30)].as_ref())
        .split(chunks[0]);
    // Left: Notes List
    let notes_items: Vec<ListItem> = app_state.notes.iter().enumerate().map(|(i, note)| {
        let style = if i == app_state.selected_idx {
            Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(note.clone()).style(style)
    }).collect();
    let notes_list = List::new(notes_items)
        .block(Block::default().borders(Borders::ALL).title("Notes"));
    f.render_widget(notes_list, main_chunks[0]);
    // Right: Info / AI Output
    let info = if let Some(ai_out) = &app_state.last_ai_output {
        vec![Spans::from(Span::raw(ai_out))]
    } else {
        vec![
            Spans::from("Use j/k to navigate, n: new, e: edit, : to command"),
            Spans::from("AI commands: ':ai summarize' or ':ai keywords'"),
        ]
    };
    let info_block = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title("Info / AI Output"));
    f.render_widget(info_block, main_chunks[1]);
    // Footer
    let footer = Paragraph::new(Spans::from(vec![
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(": Quit   "),
        Span::styled("e", Style::default().fg(Color::Yellow)),
        Span::raw(": Edit   "),
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(": New Note   "),
        Span::styled(":", Style::default().fg(Color::Yellow)),
        Span::raw(": Command Palette"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[1]);
    if app_state.show_command_palette {
        draw_command_palette(f, app_state, chunks[0]);
    }
}

fn draw_command_palette<B: ratatui::backend::Backend>(f: &mut Frame<B>, app_state: &AppState, area: Rect) {
    let height = 3;
    let width = area.width.saturating_sub(4);
    let palette_area = Rect {
        x: area.x + 2,
        y: area.y + area.height.saturating_sub(height) - 2,
        width,
        height,
    };
    let text = format!(":{}", app_state.command_input);
    let block = Block::default().borders(Borders::ALL).title("Command");
    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, palette_area);
}

fn load_notes_list() -> Vec<String> {
    let mut result = Vec::new();
    for entry in WalkDir::new(NOTES_DIR)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(stem) = path.file_stem() {
                result.push(stem.to_string_lossy().to_string());
            }
        }
    }
    result.sort();
    result
}

fn prompt_create_note() -> io::Result<()> {
    print!("Enter title for new note: ");
    io::stdout().flush()?;
    let mut title = String::new();
    io::stdin().read_line(&mut title)?;
    let title = title.trim();
    if !title.is_empty() {
        create_new_note(title)?;
    }
    Ok(())
}

fn open_in_editor(note_stem: &str) -> io::Result<()> {
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());
    let filepath = format!("{}/{}.md", NOTES_DIR, note_stem);
    Command::new(editor).arg(filepath).spawn()?.wait()?;
    Ok(())
}

/// Handle command palette input and trigger AI calls.
fn handle_command_palette(cmd: String, app_state: &mut AppState) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }
    match parts.as_slice() {
        ["ai", "summarize"] => {
            if let Some(note) = app_state.selected_note() {
                let path = format!("{}/{}.md", NOTES_DIR, note);
                let content = fs::read_to_string(&path)?;
                let summary = run_openai_summarize(&content)?;
                app_state.last_ai_output = Some(summary);
            }
        }
        ["ai", "keywords"] => {
            if let Some(note) = app_state.selected_note() {
                let path = format!("{}/{}.md", NOTES_DIR, note);
                let content = fs::read_to_string(&path)?;
                let keywords = run_openai_keywords(&content)?;
                app_state.last_ai_output = Some(keywords);
            }
        }
        _ => {
            app_state.last_ai_output = Some(format!("Unknown command: {}", cmd));
        }
    }
    Ok(())
}

// -------------------------
// OpenAI Integration
// -------------------------

fn run_openai_summarize(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_openai_summarize(content))
}

fn run_openai_keywords(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_openai_keywords(content))
}

async fn async_openai_summarize(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY not set. Please set it to use AI features.")?;
    let client = Client::new().with_api_key(api_key);
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .messages(vec![
            ChatCompletionRequestMessage {
                role: Role::System,
                content: "You are a helpful assistant summarizing notes.".to_string(),
                name: None,
            },
            ChatCompletionRequestMessage {
                role: Role::User,
                content: format!("Summarize this note:\n\n{}", content),
                name: None,
            },
        ])
        .build()?;
    let response = client.chat().create(request).await?;
    let summary = response.choices.first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No summary received.".to_string());
    Ok(summary)
}

async fn async_openai_keywords(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY not set. Please set it to use AI features.")?;
    let client = Client::new().with_api_key(api_key);
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .messages(vec![
            ChatCompletionRequestMessage {
                role: Role::System,
                content: "You are a helpful assistant that extracts keywords from notes.".to_string(),
                name: None,
            },
            ChatCompletionRequestMessage {
                role: Role::User,
                content: format!("Extract keywords from this note:\n\n{}", content),
                name: None,
            },
        ])
        .build()?;
    let response = client.chat().create(request).await?;
    let keywords = response.choices.first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No keywords received.".to_string());
    Ok(keywords)
}