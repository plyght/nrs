use crate::notes::{note_path, notes_dir};
use actix_files::{Files, NamedFile};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

/// A note node in the graph.
#[derive(Debug, Serialize)]
pub struct NoteNode {
    pub id: String,
    pub is_tag: bool,
}

/// A note link in the graph.
#[derive(Debug, Serialize)]
pub struct NoteLink {
    pub source: String,
    pub target: String,
}

/// Note data for API
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteData {
    pub title: String,
    pub slug: String,
    pub preview: String,
    pub tags: Vec<String>,
    pub last_modified: u64, // Unix timestamp
}

/// Start the web server.
pub async fn serve_notes(port: u16) -> io::Result<()> {
    println!("Web server on http://127.0.0.1:{}", port);

    // Ensure static directory exists
    if !Path::new("static").exists() {
        eprintln!("Warning: 'static' directory not found, web interface may not work correctly");
    }

    HttpServer::new(|| {
        App::new()
            // Enable logger middleware with more verbose output
            .wrap(middleware::Logger::new("%a %r %s %b %T"))
            // Serve API Routes first - high priority
            .route("/api/notes", web::get().to(notes_list_api))
            .route("/api/notes/{stem}", web::get().to(note_detail_api))
            .route("/api/graph-data", web::get().to(graph_data))
            // Serve specific app.js and app.css files with proper MIME types
            .route("/assets/{filename:.*}", web::get().to(serve_assets))
            // Serve source files for development builds
            .service(Files::new("/src", "web-ui/src").show_files_listing())
            // Serve public files (polyfills, etc.)
            .service(Files::new("/public", "web-ui/public").show_files_listing())
            // Legacy static files - must come after API routes
            .service(Files::new("/static", "static").show_files_listing())
            // Web Routes
            .route("/", web::get().to(index_page))
            .route("/notes/{stem}", web::get().to(serve_note))
            .route("/graph", web::get().to(graph_page))
            .route("/graph-data", web::get().to(graph_data)) // Keep for backward compatibility
            // SPA fallback - handle all React Router paths
            .default_service(web::get().to(index_page))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

/// Return a list of notes as JSON for the API.
pub async fn notes_list_api() -> impl Responder {
    let notes = collect_notes_data();
    HttpResponse::Ok().json(notes)
}

/// Return details of a specific note as JSON.
pub async fn note_detail_api(stem: web::Path<String>) -> impl Responder {
    let p = note_path(&stem);
    if !p.exists() {
        return HttpResponse::NotFound().body("Note not found");
    }

    match extract_note_data(&stem) {
        Some(note) => HttpResponse::Ok().json(note),
        None => HttpResponse::InternalServerError().body("Failed to extract note data"),
    }
}

/// Extract note data from a file.
fn extract_note_data(slug: &str) -> Option<NoteData> {
    let p = note_path(slug);
    let metadata = fs::metadata(&p).ok()?;
    let content = fs::read_to_string(&p).ok()?;

    let modified = metadata.modified().ok()?;
    let last_modified = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    let yaml_delim = Regex::new(r"^---\s*$").unwrap();
    let re_title = Regex::new(r"^\s*title:\s*(.+)$").unwrap();
    let re_tags = Regex::new(r"^\s*tags:\s*\[([^\]]*)\]").unwrap();

    let mut lines = content.lines();
    let mut title = slug.replace('_', " ");
    let mut tags = Vec::new();
    let mut in_yaml = false;

    // Parse YAML front matter
    for line in lines.by_ref() {
        if yaml_delim.is_match(line) {
            if !in_yaml {
                in_yaml = true;
                continue;
            } else {
                break; // End of front matter
            }
        }

        if in_yaml {
            if let Some(caps) = re_title.captures(line) {
                if let Some(m) = caps.get(1) {
                    title = m.as_str().trim().to_string();
                }
            }

            if let Some(caps) = re_tags.captures(line) {
                if let Some(m) = caps.get(1) {
                    tags = m
                        .as_str()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
    }

    // Extract preview (first few lines after front matter)
    let preview_lines: Vec<&str> = content
        .lines()
        .skip_while(|line| !yaml_delim.is_match(line)) // Skip until first delimiter
        .skip(1) // Skip the delimiter
        .skip_while(|line| !yaml_delim.is_match(line)) // Skip until second delimiter
        .skip(1) // Skip the delimiter
        .take(3) // Take first 3 lines
        .collect();

    let preview = if preview_lines.is_empty() {
        "No content".to_string()
    } else {
        preview_lines.join("\n")
    };

    Some(NoteData {
        title,
        slug: slug.to_string(),
        preview,
        tags,
        last_modified,
    })
}

/// Collect all notes data for API.
fn collect_notes_data() -> Vec<NoteData> {
    let mut notes_data = Vec::new();

    for entry in WalkDir::new(notes_dir()).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(stem) = p.file_stem() {
                    let slug = stem.to_string_lossy().to_string();
                    if let Some(note_data) = extract_note_data(&slug) {
                        notes_data.push(note_data);
                    }
                }
            }
        }
    }

    // Sort by last modified (newest first)
    notes_data.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    notes_data
}

/// Serve the main index HTML page.
pub async fn index_page() -> impl Responder {
    match fs::read_to_string("static/web/index.html") {
        Ok(mut content) => {
            // Fix asset paths for proper serving from our handler
            content = content.replace("src=\"/assets/", "src=\"/assets/");
            content = content.replace("href=\"/assets/", "href=\"/assets/");

            // Make sure crossorigin attributes are removed if present
            content = content.replace(" crossorigin", "");

            // Inject a simple inline require() polyfill at the very beginning of head
            // This is simpler and more reliable than loading an external script
            if !content.contains("window.require = window.require") {
                content = content.replace(
                    "<head>",
                    "<head>\n  <script>window.require = window.require || function(id) { console.log('[REQUIRE POLYFILL]', id); return {}; };</script>"
                );
            }

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(content)
        }
        Err(_) => {
            // Fallback for development
            if let Ok(_file) = NamedFile::open("web-ui/index.html") {
                // Convert to HttpResponse manually
                let mut response = HttpResponse::Ok();
                response.content_type("text/html; charset=utf-8");
                if let Ok(content) = fs::read_to_string("web-ui/index.html") {
                    return response.body(content);
                }
            }

            // Last resort fallback
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body("<html><body><h1>UltraThink Notes</h1><p>Index file not found. Please run 'bun run build' in web-ui directory.</p></body></html>")
        }
    }
}

/// Serve a note file.
pub async fn serve_note(stem: web::Path<String>) -> impl Responder {
    let p = note_path(&stem);
    if !p.exists() {
        return HttpResponse::NotFound().body("Note not found");
    }

    // Read the file content and convert it to HTML for display
    match fs::read_to_string(&p) {
        Ok(content) => {
            // Simple conversion: just replace newlines with <br> for now
            // In a real app, you'd use a proper Markdown to HTML converter
            let html_content = content.replace('\n', "<br>").replace("  ", "&nbsp;&nbsp;");

            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{}</title>
  <style>
    :root {{
      --bg-gradient: linear-gradient(135deg, #e0d7f7, #ffffff);
      --text-color: #333;
      --accent-color: #8a70d6;
      --card-bg: #fff;
      --border-color: #d0d4d8;
    }}
    body {{
      margin: 0;
      padding: 20px;
      background: var(--bg-gradient);
      color: var(--text-color);
      font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
      line-height: 1.6;
    }}
    .note-container {{
      max-width: 800px;
      margin: 0 auto;
      background: white;
      padding: 20px;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }}
    h1, h2, h3 {{
      color: var(--accent-color);
    }}
    a {{
      color: var(--accent-color);
      text-decoration: none;
    }}
    a:hover {{
      text-decoration: underline;
    }}
    .header {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;
      padding-bottom: 10px;
      border-bottom: 1px solid var(--border-color);
    }}
  </style>
</head>
<body>
  <div class="note-container">
    <div class="header">
      <h1>{}</h1>
      <a href="/">Back to Index</a>
    </div>
    <div class="content">
      {}
    </div>
  </div>
</body>
</html>"#,
                stem.replace('_', " "), // title
                stem.replace('_', " "), // h1
                html_content            // content
            );

            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to read note content"),
    }
}

/// Serve the graph viewer HTML.
pub async fn graph_page() -> impl Responder {
    // Try the new React-based graph view first
    if let Ok(mut content) = fs::read_to_string("static/web/graph.html") {
        // Fix asset paths for proper serving from our handler
        content = content.replace("src=\"/assets/", "src=\"/assets/");
        content = content.replace("href=\"/assets/", "href=\"/assets/");

        // Make sure crossorigin attributes are removed if present
        content = content.replace(" crossorigin", "");

        // Inject a simple inline require() polyfill
        if !content.contains("window.require = window.require") {
            content = content.replace(
                "<head>",
                "<head>\n  <script>window.require = window.require || function(id) { console.log('[REQUIRE POLYFILL]', id); return {}; };</script>"
            );
        }

        return HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(content);
    }

    // Fallback to the legacy graph viewer
    if let Ok(content) = fs::read_to_string("static/graph.html") {
        return HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(content);
    }

    // Redirect to main page if no graph viewer found
    HttpResponse::TemporaryRedirect()
        .append_header(("Location", "/"))
        .body("Redirecting to home page")
}

/// Return graph data (nodes and links) as JSON.
pub async fn graph_data() -> impl Responder {
    let (nodes, links) = build_graph();
    HttpResponse::Ok().json(serde_json::json!({ "nodes": nodes, "links": links }))
}

/// Serve static assets with proper MIME types
pub async fn serve_assets(path: web::Path<String>) -> impl Responder {
    let filename = path.into_inner();
    let filepath = format!("static/web/assets/{}", filename);

    // Make sure path doesn't contain any directory traversal attempts
    if filename.contains("..") {
        return HttpResponse::BadRequest().body("Invalid path");
    }

    // Attempt to read the file
    match fs::read(&filepath) {
        Ok(content) => {
            // Determine content type based on extension
            let content_type = if filename.ends_with(".js") {
                "application/javascript"
            } else if filename.ends_with(".css") {
                "text/css"
            } else if filename.ends_with(".png") {
                "image/png"
            } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                "image/jpeg"
            } else if filename.ends_with(".svg") {
                "image/svg+xml"
            } else if filename.ends_with(".woff2") {
                "font/woff2"
            } else if filename.ends_with(".woff") {
                "font/woff"
            } else {
                "application/octet-stream"
            };

            HttpResponse::Ok().content_type(content_type).body(content)
        }
        Err(_) => {
            println!("Asset not found: {}", filepath);
            HttpResponse::NotFound().body(format!("Asset not found: {}", filename))
        }
    }
}

/// Build the graph from note content.
pub fn build_graph() -> (Vec<NoteNode>, Vec<NoteLink>) {
    let re_links = Regex::new(r"\[\[(.+?)\]\]").unwrap();
    let yaml_delim = Regex::new(r"^---\s*$").unwrap();
    let re_tags = Regex::new(r"^\s*tags:\s*\[([^\]]*)\]").unwrap();

    let mut adjacency = Vec::new(); // (this_stem, [linked_stems])
    let mut note_tags = Vec::new(); // (this_stem, [tags])

    for entry in WalkDir::new(notes_dir()).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("md") {
                let stem = p.file_stem().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(p).unwrap_or_default();

                // Find [[links]] in content.
                let mut found_links = Vec::new();
                for caps in re_links.captures_iter(&content) {
                    if let Some(m) = caps.get(1) {
                        let c = m.as_str().to_lowercase().replace(' ', "_");
                        found_links.push(c);
                    }
                }

                // Parse YAML front matter for tags.
                let lines = content.lines();
                let mut in_yaml = false;
                let mut tags = Vec::new();
                let mut fm_lines = Vec::new();
                for line in lines {
                    if yaml_delim.is_match(line) {
                        if !in_yaml {
                            in_yaml = true;
                            continue;
                        } else {
                            break;
                        }
                    }
                    if in_yaml {
                        fm_lines.push(line);
                    }
                }
                for l in fm_lines {
                    if let Some(caps) = re_tags.captures(l) {
                        let inside = caps.get(1).unwrap().as_str();
                        let raw_tags: Vec<&str> = inside.split(',').map(|s| s.trim()).collect();
                        for t in raw_tags {
                            if !t.is_empty() {
                                tags.push(t.to_lowercase().replace(' ', "_"));
                            }
                        }
                    }
                }

                adjacency.push((stem.clone(), found_links));
                note_tags.push((stem, tags));
            }
        }
    }

    // Build sets of nodes and links.
    let mut note_set = std::collections::HashSet::new();
    let mut tag_set = std::collections::HashSet::new();
    let mut links = Vec::new();

    for (src, link_list) in &adjacency {
        note_set.insert(src.clone());
        for l in link_list {
            note_set.insert(l.clone());
        }
    }
    for (st, tags) in &note_tags {
        note_set.insert(st.clone());
        for t in tags {
            tag_set.insert(t.clone());
        }
    }

    let mut nodes = Vec::new();
    for n in note_set {
        nodes.push(NoteNode {
            id: n,
            is_tag: false,
        });
    }
    for t in tag_set {
        nodes.push(NoteNode {
            id: t,
            is_tag: true,
        });
    }

    for (src, link_list) in adjacency {
        for l in link_list {
            links.push(NoteLink {
                source: src.clone(),
                target: l,
            });
        }
    }
    for (st, tags) in note_tags {
        for t in tags {
            links.push(NoteLink {
                source: st.clone(),
                target: t,
            });
        }
    }

    (nodes, links)
}
