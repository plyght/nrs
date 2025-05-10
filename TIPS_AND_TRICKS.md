# NRS Tips and Tricks

This document contains advanced tips, workflows, and best practices for getting the most out of NRS.

## Note Organization

### Effective Tagging

Tags provide a powerful way to organize your notes:

- Use consistent tag naming conventions
- Create hierarchical tags with prefixes (e.g., `project/personal`, `project/work`)
- Limit to 3-5 tags per note for better organization
- Consider using tags for priority levels or status indicators

Example YAML front matter:
```yaml
---
title: Project Roadmap
tags: [project/acme, planning, priority/high, status/in-progress]
---
```

### Wiki-Style Linking

Create connections between notes using double-bracket links:

```markdown
Check the [[meeting_notes]] for more details on the [[project_plan]].
```

Benefits:
- Creates visual connections in the graph view
- Makes navigation between related notes easier
- Helps build a knowledge network

## TUI Workflows

### Keyboard-Only Efficiency

The TUI is designed for keyboard-driven workflows:

1. Use `/` to quickly search for notes
2. Press `Tab` to cycle between Notes, Preview, and AI tabs
3. Learn the command palette shortcuts (press `:` to activate)
4. Use `e` to quickly edit the current note in your preferred editor

### Power User Commands

Some less obvious but powerful TUI features:

- Press `r` to refresh the note list after external changes
- Use `t` to toggle tag display in the notes list
- Tab completion works in the command palette (try typing `:s` and then `Tab`)

## AI Integration

### Effective Prompting

When using the AI commands, consider these tips:

1. For `:summarize`:
   - Works best on structured notes with clear sections
   - Most effective on longer notes (250+ words)
   - Consider adding a "key points" section for better summarization

2. For `:keywords`:
   - More effective when your note has a clear topic focus
   - Use the extracted keywords to improve your tagging strategy

### Custom Commands

You can extend the AI commands by modifying the `commands.rs` and `ai.rs` files:

- Add domain-specific commands (e.g., `:analyze_code`, `:extract_action_items`)
- Customize the system prompts to fit your specific needs

## Web Interface

### Keyboard Shortcuts

The web interface supports several keyboard shortcuts:

- `Ctrl+K` (or `Cmd+K` on macOS) to open the search dialog
- Arrow keys to navigate between notes in the list
- `Esc` to close dialogs

### URL Patterns

Learn these URL patterns for direct access:

- `/notes/{note_slug}` - Direct access to a specific note
- `/graph` - Jump straight to the graph visualization

## Advanced Features

### Note Templates

Create template notes to standardize your note-taking:

```markdown
---
title: Meeting Note Template
tags: [template, meeting]
---
# Meeting: $TITLE

## Attendees
- 

## Agenda
1. 

## Action Items
- [ ] 

## Notes
```

Save this as `meeting_template.md` and adapt when creating new meeting notes.

### External Editor Integration

Set up your preferred editor with the `EDITOR` environment variable:

```bash
# For VSCode
export EDITOR="code -w"

# For Neovim
export EDITOR="nvim"

# For Sublime Text
export EDITOR="subl -w"
```

The `-w` flag is important as it makes the program wait until the file is closed.

### Graph Analysis

The graph view can reveal interesting insights about your notes:

- Highly connected notes often represent key concepts or central ideas
- Orphaned notes (no connections) might need integration into your knowledge base
- Clusters of notes suggest potential topic areas to organize or consolidate

## Data Management

### Backup Strategy

Implement a reliable backup strategy:

1. Regular backups of your `~/notes` directory
2. Consider using Git for version control of your notes:
   ```bash
   cd ~/notes
   git init
   git add .
   git commit -m "Initial notes commit"
   ```

3. For cloud sync, consider using a service like Dropbox or set up a Git remote

### Import/Export

While NRS doesn't natively support import/export, your notes are stored as standard Markdown files, making migration simple:

- To import notes from other systems, convert them to Markdown with YAML front matter
- To export, simply copy the files from your `~/notes` directory

## Troubleshooting

### Common Issues

1. **Web UI Not Loading**: Make sure you've built the web UI with `cd web-ui && bun run build`
2. **TUI Display Problems**: Try adjusting your terminal font or window size
3. **AI Commands Not Working**: Verify your `OPENAI_API_KEY` environment variable is set

### Logs

For debugging, run with the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug nrs serve
```

This will display more detailed logs to help identify issues.