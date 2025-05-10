# NRS Web UI

The web interface for NRS, built with React, TypeScript, and Tailwind CSS.

## Overview

This web UI provides a modern, responsive interface for viewing and managing your notes. Features include:

- Note listing and viewing
- Note content rendering with Markdown support
- Graph visualization of note connections
- Responsive design for desktop and mobile

## Getting Started

### Prerequisites

- [Bun](https://bun.sh/) (JavaScript runtime and package manager)
- NRS backend running (`nrs serve`)

### Installation

```bash
# Install dependencies
bun install
```

### Development

```bash
# Start the development server
bun run dev
```

This will start a development server with hot reloading at `http://localhost:5173`. The dev server will proxy API requests to the NRS backend server running at `http://localhost:4321`.

### Production Build

```bash
# Build for production
bun run build
```

This will create optimized production files in the `dist` directory, which are automatically served by the NRS backend when you run `nrs serve`.

## Project Structure

```
web-ui/
├── src/                  # Source code
│   ├── components/       # Reusable UI components
│   │   ├── Header.tsx    # Navigation header
│   │   ├── NoteCard.tsx  # Note card component
│   │   └── ...
│   ├── hooks/            # Custom React hooks
│   │   └── useNotes.ts   # Hook for notes data fetching
│   ├── pages/            # Page components
│   │   ├── HomePage.tsx  # Main notes listing page
│   │   ├── NotePage.tsx  # Single note view
│   │   ├── GraphPage.tsx # Graph visualization
│   │   └── ...
│   ├── types/            # TypeScript type definitions
│   ├── utils/            # Utility functions
│   ├── App.tsx           # Main App component
│   ├── index.css         # Global styles
│   └── main.tsx          # Application entry point
├── public/               # Static assets
├── index.html            # HTML template
├── vite.config.ts        # Vite configuration
├── tailwind.config.js    # Tailwind CSS configuration
├── tsconfig.json         # TypeScript configuration
└── package.json          # Project dependencies and scripts
```

## Key Components

### Note Management

Notes are fetched and managed through the `useNotes` hook, which provides:

- List of all notes
- Functions to fetch individual notes
- Caching for improved performance

### Markdown Rendering

Note content is rendered as Markdown using the `MarkdownRenderer` component, which supports:

- Headings, lists, and basic formatting
- Code blocks with syntax highlighting
- Internal links between notes

### Graph Visualization

The graph visualization is implemented in `GraphPage.tsx` using D3.js:

- Notes are represented as nodes
- Connections are based on internal links and shared tags
- Interactive navigation and zooming

## Customization

### Styling

This project uses Tailwind CSS for styling. To customize the design:

1. Modify `tailwind.config.js` to adjust colors, fonts, and other theme settings
2. Edit component files to change classes and layouts
3. Update `index.css` for global style overrides

### Adding New Features

To add new features to the web UI:

1. Create any necessary components in `src/components/`
2. Add new pages in `src/pages/` if needed
3. Update routing in `App.tsx` to include new pages
4. Extend API client functions in `useNotes.ts` if additional backend data is required

## API Endpoints

The web UI interacts with these NRS backend API endpoints:

- `GET /api/notes` - Retrieve list of all notes
- `GET /api/notes/{slug}` - Get a specific note by slug
- `GET /api/graph-data` - Get graph visualization data

## Deployment

The built web UI is automatically served by the NRS backend when you run `nrs serve`. The production build process:

1. Compiles TypeScript to JavaScript
2. Optimizes and bundles assets
3. Generates production-ready files in the `dist` directory
4. These files are served from the `static/web` directory by the NRS backend

## Troubleshooting

### Common Issues

- **API Errors**: Ensure the NRS backend is running on the expected port
- **Build Failures**: Check for TypeScript errors and dependency issues
- **Styling Problems**: Verify Tailwind is configured correctly

### Development Tips

- Use the browser console for debugging
- Check the Network tab for API request issues
- Use React DevTools for component inspection

## Contributing

See the main [CONTRIBUTING.md](../CONTRIBUTING.md) file for guidelines on contributing to the project.