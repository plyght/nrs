use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use actix_files::NamedFile;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use clap::{Parser, Subcommand};
use dirs::home_dir;
use futures::executor::block_on;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

// TUI
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::{Frame, Terminal};

// OpenAI
use async_openai::Client;
use async_openai::types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, Role};
use tokio::task;

type MyError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Return the path to ~/notes
fn notes_dir() -> PathBuf {
    home_dir().expect("Could not locate home directory.").join("notes")
}

/// Return a .md path in ~/notes
fn note_path(stem: &str) -> PathBuf {
    notes_dir().join(format!("{}.md", stem))
}

static KNOWN_COMMANDS: &[&str] = &["summarize", "keywords"];

// =======================================================
// CLI
// =======================================================

#[derive(Parser, Debug)]
#[command(name = "nrs", version = "0.4.0", about = "Rust-based TUI & Web for Notes")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new note
    New {
        title: String,
    },
    /// Run the TUI
    Tui,
    /// Start the web server
    Serve {
        #[arg(short, long, default_value_t = 4321)]
        port: u16,
    },
}

// =======================================================
// Graph for web
// =======================================================

#[derive(Debug, Serialize)]
struct NoteNode {
    id: String,
}

#[derive(Debug, Serialize)]
struct NoteLink {
    source: String,
    target: String,
}

// =======================================================
// Single Unified Runtime
// =======================================================

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let ndir = notes_dir();
    if !ndir.exists() {
        fs::create_dir(&ndir).map_err(|e| {
            eprintln!("Cannot create ~/notes: {}", e);
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
                eprintln!("TUI Error: {}", e);
            }
        }
        Commands::Serve { port } => {
            if let Err(e) = serve_notes(port).await {
                eprintln!("Server Error: {}", e);
            }
        }
    }

    Ok(())
}

// =======================================================
// Note creation
// =======================================================

fn create_new_note(title: &str) -> io::Result<()> {
    let file_stem = title
        .to_lowercase()
        .replace(' ', "_")
        .replace("/", "_")
        .replace("\\", "_");
    let p = note_path(&file_stem);
    if p.exists() {
        eprintln!("Note already exists: {}", p.display());
        return Ok(());
    }
    let mut f = fs::File::create(&p)?;
    let content = format!("# {}\n\nWrite your note here.\n", title);
    f.write_all(content.as_bytes())?;
    println!("Created note at: {}", p.display());
    Ok(())
}

// =======================================================
// Actix Web
// =======================================================

async fn serve_notes(port: u16) -> io::Result<()> {
    println!("Serving on http://127.0.0.1:{}", port);
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index_page))
            .route("/notes/{stem}", web::get().to(serve_note))
            .route("/graph", web::get().to(graph_page))
            .route("/graph-data", web::get().to(graph_data))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

async fn index_page() -> impl Responder {
    let mut list_html = String::new();
    for entry in WalkDir::new(notes_dir()).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(stem) = p.file_stem() {
                    let s = stem.to_string_lossy().to_string();
                    let disp = s.replace('_', " ");
                    list_html.push_str(&format!(
                        "<li><a href=\"/notes/{0}\">{1}</a></li>",
                        s, disp
                    ));
                }
            }
        }
    }

    // D3 code in a multi-hash raw string to avoid prefix errors
    let html = format!(
r###"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>~/notes index</title>
</head>
<body>
  <h1>~/notes Index</h1>
  <ul>{list_html}</ul>
  <a href="/graph">View Graph</a>
</body>
</html>
"###,
        list_html = list_html
    );

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

async fn serve_note(stem: web::Path<String>) -> actix_web::Result<NamedFile> {
    let p = note_path(&stem.into_inner());
    Ok(NamedFile::open(p)?)
}

async fn graph_page() -> impl Responder {
    let html = r###"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>~/notes Graph</title>
  <script src="https://d3js.org/d3.v7.min.js"></script>
</head>
<body>
  <h1>~/notes Graph</h1>
  <div id="graph"></div>
  <script>
  const width = 800, height=600;
  fetch('/graph-data')
    .then(resp=>resp.json())
    .then(data=>{
      const svg = d3.select("#graph")
        .append("svg")
        .attr("width", width)
        .attr("height", height);

      const sim = d3.forceSimulation(data.nodes)
        .force("link", d3.forceLink(data.links).id(d => d.id).distance(100))
        .force("charge", d3.forceManyBody().strength(-200))
        .force("center", d3.forceCenter(width/2, height/2));

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
          .on("start", (e,d)=>{
            if(!e.active) sim.alphaTarget(0.3).restart();
            d.fx = d.x; d.fy = d.y;
          })
          .on("drag", (e,d)=>{
            d.fx=e.x; d.fy=e.y;
          })
          .on("end", (e,d)=>{
            if(!e.active) sim.alphaTarget(0);
            d.fx=null; d.fy=null;
          }));

      const label = svg.selectAll("text")
        .data(data.nodes)
        .enter().append("text")
        .text(d=>d.id)
        .attr("font-size","12px")
        .attr("dx",12)
        .attr("dy","0.35em");

      node.on("click",(_,d)=>{
        const rep = d.id.replace(' ','_');
        window.location.href="/notes/"+rep;
      });

      sim.on("tick", ()=>{
        link
          .attr("x1", d=>d.source.x)
          .attr("y1", d=>d.source.y)
          .attr("x2", d=>d.target.x)
          .attr("y2", d=>d.target.y);

        node
          .attr("cx", d=>d.x)
          .attr("cy", d=>d.y);

        label
          .attr("x", d=>d.x)
          .attr("y", d=>d.y);
      });
    });
  </script>
</body>
</html>
"###;
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

async fn graph_data() -> impl Responder {
    let (nodes, links) = build_graph();
    HttpResponse::Ok().json(serde_json::json!({ "nodes": nodes, "links": links }))
}

fn build_graph() -> (Vec<NoteNode>, Vec<NoteLink>) {
    let pat = Regex::new(r"\[\[(.+?)\]\]").unwrap();
    let mut adjacency = Vec::new();

    for entry in WalkDir::new(notes_dir()).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("md") {
                let stem = p.file_stem().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(p).unwrap_or_default();
                let mut found = Vec::new();
                for caps in pat.captures_iter(&content) {
                    if let Some(m) = caps.get(1) {
                        let c = m.as_str().to_lowercase().replace(' ', "_");
                        found.push(c);
                    }
                }
                adjacency.push((stem, found));
            }
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
    for (src, list) in adjacency {
        for l in list {
            links.push(NoteLink {
                source: src.clone(),
                target: l,
            });
        }
    }
    (nodes, links)
}

// =======================================================
// TUI
// =======================================================

struct AppState {
    notes: Vec<String>,
    selected_idx: usize,
    show_cmd: bool,
    cmd_input: String,
    last_ai_output: Option<String>,
    preview: Option<String>,
}

impl AppState {
    fn new() -> Self {
        let mut s = Self {
            notes: load_notes_list(),
            selected_idx: 0,
            show_cmd: false,
            cmd_input: String::new(),
            last_ai_output: None,
            preview: None,
        };
        s.update_preview();
        s
    }

    fn next_note(&mut self) {
        if !self.notes.is_empty() {
            self.selected_idx = (self.selected_idx + 1) % self.notes.len();
            self.update_preview();
        }
    }

    fn prev_note(&mut self) {
        if !self.notes.is_empty() {
            if self.selected_idx == 0 {
                self.selected_idx = self.notes.len() - 1;
            } else {
                self.selected_idx -= 1;
            }
            self.update_preview();
        }
    }

    fn selected_note(&self) -> Option<&String> {
        self.notes.get(self.selected_idx)
    }

    fn update_preview(&mut self) {
        self.preview = None;
        if let Some(stem) = self.selected_note() {
            let p = note_path(stem);
            if let Ok(content) = fs::read_to_string(p) {
                let lines: Vec<_> = content.lines().take(10).collect();
                self.preview = Some(lines.join("\n"));
            }
        }
    }
}

/// Temporarily leave raw mode to prompt for new note
fn prompt_create_note_with_restore() -> io::Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;
    let r = prompt_create_note();
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    r
}

fn run_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let res = (|| {
        let mut st = AppState::new();
        loop {
            term.draw(|f| ui(f, &st))?;

            if event::poll(Duration::from_millis(50))? {
                let ev = event::read()?;
                match ev {
                    Event::Key(KeyEvent { code, modifiers, .. }) => {
                        // always let 'q' exit
                        if code == KeyCode::Char('q') && modifiers == KeyModifiers::NONE {
                            break;
                        }
                        if st.show_cmd {
                            match code {
                                KeyCode::Esc => st.show_cmd = false,
                                KeyCode::Enter => {
                                    let cmd = st.cmd_input.clone();
                                    if let Err(e) = handle_cmd(cmd, &mut st) {
                                        st.last_ai_output = Some(format!("Error: {}", e));
                                    }
                                    st.cmd_input.clear();
                                    st.show_cmd = false;
                                }
                                KeyCode::Backspace => {
                                    st.cmd_input.pop();
                                }
                                KeyCode::Char('\t') => {
                                    // naive tab complete
                                    let partial = st.cmd_input.trim_start_matches(':');
                                    let matches: Vec<_> = KNOWN_COMMANDS
                                        .iter()
                                        .filter(|c| c.starts_with(partial))
                                        .collect();
                                    if matches.len() == 1 {
                                        st.cmd_input = format!(":{}", matches[0]);
                                    }
                                }
                                KeyCode::Char(c) => {
                                    st.cmd_input.push(c);
                                }
                                _ => {}
                            }
                        } else {
                            match code {
                                KeyCode::Char('j') => st.next_note(),
                                KeyCode::Char('k') => st.prev_note(),
                                KeyCode::Char('n') => {
                                    if let Err(e) = prompt_create_note_with_restore() {
                                        eprintln!("Error: {}", e);
                                    }
                                    st.notes = load_notes_list();
                                    st.selected_idx = 0;
                                    st.update_preview();
                                }
                                KeyCode::Char('e') => {
                                    if let Some(sm) = st.selected_note() {
                                        if let Err(e) = open_in_editor(sm) {
                                            eprintln!("Editor error: {}", e);
                                        }
                                        st.update_preview();
                                    }
                                }
                                KeyCode::Char(':') => {
                                    st.show_cmd = true;
                                    st.cmd_input.clear();
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();

    // restore
    disable_raw_mode()?;
    crossterm::execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;

    res
}

fn ui<B: ratatui::backend::Backend>(f: &mut Frame<B>, s: &AppState) {
    let top_bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
        .split(f.size());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(30)].as_ref())
        .split(top_bottom[0]);

    // Left: notes
    let items: Vec<ListItem> = s.notes.iter().enumerate().map(|(i, nm)| {
        let style = if i == s.selected_idx {
            Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(nm.clone()).style(style)
    }).collect();
    let left = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("~/notes"));
    f.render_widget(left, main_chunks[0]);

    // Right: AI or preview
    let lines = if let Some(ai) = &s.last_ai_output {
        vec![Spans::from(ai.as_str())]
    } else {
        if let Some(prev) = &s.preview {
            prev.lines().map(|l| Spans::from(l)).collect()
        } else {
            vec![Spans::from("No note selected.")]
        }
    };
    let right = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Preview / AI Output"));
    f.render_widget(right, main_chunks[1]);

    // footer
    let foot = Spans::from(vec![
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":Quit "),
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(":Select "),
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(":New "),
        Span::styled("e", Style::default().fg(Color::Yellow)),
        Span::raw(":Edit "),
        Span::styled(":", Style::default().fg(Color::Yellow)),
        Span::raw(":Command"),
    ]);
    let foot_par = Paragraph::new(foot)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(foot_par, top_bottom[1]);

    if s.show_cmd {
        draw_cmd_palette(f, s, main_chunks[1]);
    }
}

fn draw_cmd_palette<B: ratatui::backend::Backend>(f: &mut Frame<B>, s: &AppState, right: Rect) {
    let h = 3;
    let w = right.width.saturating_sub(4);
    let rect = Rect {
        x: right.x + 2,
        y: right.y + right.height.saturating_sub(h) - 2,
        width: w,
        height: h,
    };
    let text = format!("{}", s.cmd_input);
    let block = Block::default().borders(Borders::ALL).title("Command");
    let par = Paragraph::new(text).block(block);
    f.render_widget(par, rect);
}

// =======================================================
// Helpers
// =======================================================

fn load_notes_list() -> Vec<String> {
    let mut out = Vec::new();
    for entry in WalkDir::new(notes_dir()).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(st) = p.file_stem() {
                    out.push(st.to_string_lossy().to_string());
                }
            }
        }
    }
    out.sort();
    out
}

fn prompt_create_note() -> io::Result<()> {
    print!("Enter title for new note: ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let t = buf.trim();
    if !t.is_empty() {
        create_new_note(t)?;
    }
    Ok(())
}

fn open_in_editor(stem: &str) -> io::Result<()> {
    // leave raw
    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    let ed = env::var("EDITOR").or_else(|_| env::var("VISUAL")).unwrap_or_else(|_| "vi".to_string());
    let path = note_path(stem);
    let mut child = Command::new(ed).arg(path).spawn()?;
    let _ = child.wait();

    // re-enter raw
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Ok(())
}

// =======================================================
// Commands
// =======================================================

fn handle_cmd(cmd: String, st: &mut AppState) -> Result<(), MyError> {
    // remove leading colon
    let trimmed = cmd.trim_start_matches(':').trim();
    match trimmed {
        "summarize" => {
            if let Some(sn) = st.selected_note() {
                let content = fs::read_to_string(note_path(sn))?;
                // spawn_blocking, then block_on
                let handle = task::spawn_blocking(move || openai_summarize_blocking(content));
                let text = block_on(handle)??; // block_on + ?? to unwrap the handle and the result
                st.last_ai_output = Some(text);
            }
        }
        "keywords" => {
            if let Some(sn) = st.selected_note() {
                let content = fs::read_to_string(note_path(sn))?;
                let handle = task::spawn_blocking(move || openai_keywords_blocking(content));
                let text = block_on(handle)??;
                st.last_ai_output = Some(text);
            }
        }
        other => {
            st.last_ai_output = Some(format!("Unknown command: '{}'", other));
        }
    }
    Ok(())
}

// =======================================================
// AI
// =======================================================

fn openai_summarize_blocking(content: String) -> Result<String, MyError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let key = env::var("OPENAI_API_KEY")?;
        let cli = Client::new().with_api_key(key);
        let req = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o")
            .messages(vec![
                ChatCompletionRequestMessage {
                    role: Role::System,
                    content: "You are a helpful assistant that summarizes notes.".into(),
                    name: None,
                },
                ChatCompletionRequestMessage {
                    role: Role::User,
                    content: format!("Summarize:\n\n{}", content),
                    name: None,
                },
            ])
            .build()?;
        let resp = cli.chat().create(req).await?;
        let text = resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "No summary received.".to_string());
        Ok(text)
    })
}

fn openai_keywords_blocking(content: String) -> Result<String, MyError> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let key = env::var("OPENAI_API_KEY")?;
        let cli = Client::new().with_api_key(key);
        let req = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .messages(vec![
                ChatCompletionRequestMessage {
                    role: Role::System,
                    content: "You are a helpful assistant that extracts keywords.".into(),
                    name: None,
                },
                ChatCompletionRequestMessage {
                    role: Role::User,
                    content: format!("Extract keywords:\n\n{}", content),
                    name: None,
                },
            ])
            .build()?;
        let resp = cli.chat().create(req).await?;
        let text = resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "No keywords received.".to_string());
        Ok(text)
    })
}