# NRS Architecture

This document provides an overview of the NRS application architecture to help with understanding and extending the codebase.

## Core Components

NRS is divided into several key components:

1. **CLI Interface** (`main.rs`) - Command-line argument parsing with Clap
2. **Note Management** (`notes.rs`) - Core functionality for note creation, storage, and retrieval
3. **Terminal UI** (`tui.rs`) - Terminal interface built with Ratatui and Crossterm
4. **Web Server** (`web.rs`) - Web API and server using Actix Web
5. **AI Integration** (`ai.rs`) - Integration with OpenAI's GPT models
6. **Command Handling** (`commands.rs`) - Processing commands in the terminal UI

## Data Flow

### Note Storage

- Notes are stored as Markdown files in the `~/notes` directory
- Each note uses YAML front matter for metadata (title, tags)
- File names are slugified versions of the note titles

### TUI Workflow

1. User navigates the list of notes in the TUI
2. When a note is selected, its content is loaded and displayed in the preview pane
3. Commands processed via the command palette trigger AI analysis

### Web Workflow

1. Web server provides APIs for fetching notes and graph data
2. React frontend consumes these APIs to render the UI
3. D3.js is used for graph visualization

## Key Interfaces

### CLI Commands

The application exposes three primary commands through its CLI:

```rust
#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new note
    New { title: String },
    /// Run TUI
    Tui,
    /// Start the web server
    Serve {
        #[arg(short, long, default_value_t = 4321)]
        port: u16,
    },
}
```

### Web API Endpoints

- `GET /api/notes` - List all notes
- `GET /api/notes/{stem}` - Get details for a specific note
- `GET /api/graph-data` - Get the graph data for visualization

## App State Management

### TUI State

The Terminal UI maintains state in the `AppState` struct:

```rust
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
```

### Web UI State

The web UI uses React hooks for state management, particularly in the `useNotes` hook.

## Extension Points

### Adding New AI Commands

To add a new AI command:

1. Add a new function to `ai.rs` similar to `openai_summarize_blocking`
2. Update the `handle_cmd` function in `commands.rs` to handle the new command
3. Add command autocompletion in `tui.rs`

### Adding New Note Features

To add new note features:

1. Update the `NoteData` struct in `web.rs` if adding new metadata
2. Modify the `extract_note_data` function to process the new data
3. Update the web UI to display the new features

## Implementation Details

### Note Creation

New notes are created with this structure:

```
---
title: Note Title
tags: []
---
# Note Title

Write your note here.
```

### AI Requests

AI requests are made using the `async-openai` crate:

```rust
let req = CreateChatCompletionRequestArgs::default()
    .model("gpt-4o-latest")
    .messages(vec![
        ChatCompletionRequestMessage {
            role: Role::System,
            content: "You are a helpful assistant that summarizes notes.".to_string(),
            name: None,
        },
        ChatCompletionRequestMessage {
            role: Role::User,
            content: format!("Summarize:\n\n{}", content),
            name: None,
        },
    ])
    .build()?;
```

### Graph Visualization

Notes are connected in the graph by:

1. [[wiki-style links]] in note content
2. Tags assigned in the YAML front matter

## Performance Considerations

- Note parsing is done on demand to minimize startup time
- Web assets are cached and served efficiently
- AI requests are processed in a blocking context to avoid UI freezes

## Future Directions

Potential areas for enhancement:

1. **Improved Search** - Add full-text search capabilities
2. **Markdown Rendering** - Enhance markdown rendering in both TUI and Web UI
3. **Rich Media Support** - Add support for images and other media in notes
4. **Sync Capabilities** - Add synchronization with cloud storage
5. **Plugin System** - Create a plugin architecture for extensions