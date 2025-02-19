use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use std::io;
use std::fs;
use regex::Regex;
use walkdir::WalkDir;
use serde::Serialize;
use crate::notes::notes_dir;

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

/// Start the web server.
pub async fn serve_notes(port: u16) -> io::Result<()> {
    println!("Web server on http://127.0.0.1:{}", port);
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

/// List all notes in an HTML page.
pub async fn index_page() -> impl Responder {
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
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>~/notes Index</title>
</head>
<body>
  <h1>All notes in ~/notes</h1>
  <ul>{}</ul>
  <a href="/graph">View Graph</a>
</body>
</html>"#, list_html);
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

/// Serve a note file.
pub async fn serve_note(stem: web::Path<String>) -> actix_web::Result<NamedFile> {
    let p = crate::notes::note_path(&stem.into_inner());
    Ok(NamedFile::open(p)?)
}

/// Serve the graph viewer HTML.
pub async fn graph_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("static/graph.html")?)
}

/// Return graph data (nodes and links) as JSON.
pub async fn graph_data() -> impl Responder {
    let (nodes, links) = build_graph();
    HttpResponse::Ok().json(serde_json::json!({ "nodes": nodes, "links": links }))
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
                let mut lines = content.lines();
                let mut in_yaml = false;
                let mut tags = Vec::new();
                let mut fm_lines = Vec::new();
                while let Some(line) = lines.next() {
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
        nodes.push(NoteNode { id: n, is_tag: false });
    }
    for t in tag_set {
        nodes.push(NoteNode { id: t, is_tag: true });
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