# NRS - Hackable CLI Note-Taking App

A hackable CLI note-taking app built in Rust with a web server, TUI interface, command palette, AI integration, and graph visualization.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)
![Vite](https://img.shields.io/badge/vite-%23646CFF.svg?style=for-the-badge&logo=vite&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/tailwindcss-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)

## Features

- **Command-line Interface**: Create and organize markdown notes from your terminal
- **Terminal UI (TUI)**: Browse and edit notes with a nice terminal interface
- **Web Interface**: Access your notes through a modern React web application
- **AI Integration**: Summarize notes and extract keywords with GPT-4
- **Graph Visualization**: See connections between notes and tags
- **Hackable Architecture**: Designed to be easily extended and customized

## Installation

### Prerequisites

- Rust (latest stable version recommended)
- Bun.js (for web UI development)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-username/nrs.git
cd nrs

# Build the Rust application
cargo build --release

# Install the binary
cargo install --path .

# Build the web UI
cd web-ui
bun install
bun run build
```

## Usage

### CLI Commands

```bash
# Create a new note
nrs new "My New Note"

# Launch the TUI interface
nrs tui

# Start the web server
nrs serve
# Or specify a custom port
nrs serve --port 8080
```

### TUI Keyboard Shortcuts

| Key | Function |
|-----|----------|
| `↑/↓` or `j/k` | Navigate between notes |
| `Tab` | Switch between Notes/Preview/AI tabs |
| `n` | Create a new note |
| `e` | Edit current note in $EDITOR |
| `/` | Search notes |
| `:` | Command palette |
| `t` | Toggle tag display |
| `r` | Refresh note list |
| `h` | Show help |
| `q` | Quit |

### AI Commands

To use AI features, set your OpenAI API key:

```bash
export OPENAI_API_KEY=your-api-key-here
```

In the TUI, use these commands:
- `:summarize` - Generate a summary of the current note
- `:keywords` - Extract keywords from the current note

### Web Interface

Access the web interface at `http://localhost:4321` after starting the server with `nrs serve`.

## Development

### Project Structure

- `src/` - Rust source code
  - `main.rs` - Entry point and CLI definition
  - `notes.rs` - Core note functionality
  - `tui.rs` - Terminal UI implementation
  - `web.rs` - Web server and API
  - `ai.rs` - AI integration with OpenAI
  - `commands.rs` - Command handlers
- `web-ui/` - React web interface
  - `src/` - TypeScript and React components
  - `public/` - Static assets

### Building the Web UI

```bash
cd web-ui
bun install
bun run dev   # Development server
bun run build # Production build
```

## License

MIT

---

**Note**: This project is a learning exercise in Rust development. Expect occasional bugs and high CPU usage during development.