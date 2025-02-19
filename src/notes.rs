use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};
use dirs::home_dir;
use walkdir::WalkDir;

/// Return the path to `~/notes`.
pub fn notes_dir() -> PathBuf {
    home_dir().expect("Could not locate home directory.").join("notes")
}

/// Build a `.md` path for the given stem in `~/notes`.
pub fn note_path(stem: &str) -> PathBuf {
    notes_dir().join(format!("{}.md", stem))
}

/// Create a new note with YAML front matter.
pub fn create_new_note(title: &str) -> io::Result<()> {
    let slug = title
        .to_lowercase()
        .replace(' ', "_")
        .replace("/", "_")
        .replace("\\", "_");
    let path = note_path(&slug);
    if path.exists() {
        eprintln!("Note already exists: {}", path.display());
        return Ok(());
    }
    let mut f = fs::File::create(&path)?;
    let content = format!(
r#"---
title: {t}
tags: []
---
# {t}

Write your note here.
"#,
        t = title
    );
    f.write_all(content.as_bytes())?;
    println!("Created note at: {}", path.display());
    Ok(())
}

/// Load and return a sorted list of note stems.
pub fn load_notes_list() -> Vec<String> {
    let mut out = Vec::new();
    for entry in WalkDir::new(notes_dir()).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(stem) = p.file_stem() {
                    out.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    out.sort();
    out
}