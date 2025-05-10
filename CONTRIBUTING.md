# Contributing to NRS

Thank you for your interest in contributing to NRS! This document will guide you through the contribution process.

## Code of Conduct

Please be respectful and considerate in all interactions related to this project.

## Getting Started

### Prerequisites

- Rust (latest stable version recommended)
- Bun.js (for web UI development)
- A working OpenAI API key (for testing AI features)

### Development Environment Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/nrs.git
   cd nrs
   ```
3. Add the original repository as an upstream remote:
   ```bash
   git remote add upstream https://github.com/original-owner/nrs.git
   ```
4. Install dependencies:
   ```bash
   # For the Rust application
   cargo build
   
   # For the web UI
   cd web-ui
   bun install
   ```

## Development Workflow

### Branching Strategy

- `main` - The primary development branch
- Create feature branches from `main` for your work

### Making Changes

1. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   ```
   or
   ```bash
   git checkout -b fix/your-bugfix-name
   ```

2. Make your changes
3. Run tests and linters:
   ```bash
   # Rust
   cargo test
   cargo fmt -- --check
   cargo clippy
   
   # Web UI
   cd web-ui
   bun run lint
   bun run format
   ```

4. Commit your changes with descriptive commit messages
5. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

6. Create a pull request against the `main` branch of the original repository

### Pull Request Guidelines

- Keep PRs focused on a single feature or bugfix
- Write clear descriptions of the changes and their purpose
- Include any relevant documentation updates
- Ensure all tests and linters pass
- Be responsive to feedback and questions

## Project Structure

### Code Organization

- `src/` - Rust source code
  - `main.rs` - CLI entry point
  - `notes.rs` - Note functionality core
  - `tui.rs` - Terminal UI
  - `web.rs` - Web server and API
  - `ai.rs` - AI integration
  - `commands.rs` - TUI command handling
- `web-ui/` - React web interface
  - `src/` - Frontend source code
  - `public/` - Static assets

### Guidelines for Different Types of Contributions

#### Adding a New Feature

1. Discuss the feature in an issue first to gather feedback
2. Consider how the feature fits into the existing architecture
3. Update relevant documentation
4. Add tests for the new functionality

#### Fixing Bugs

1. Create an issue describing the bug if one doesn't exist
2. Reference the issue in your pull request
3. Add a test that reproduces the bug if possible
4. Explain your fix approach in the PR description

#### Documentation Improvements

1. Check for consistency with existing documentation
2. Use clear, concise language
3. Add examples where appropriate

#### Performance Improvements

1. Include benchmarks showing the improvement
2. Describe the approach and technical details
3. Consider any trade-offs (e.g., memory vs. speed)

## Architectural Considerations

### Components and Extension Points

The application is designed to be extended in these key areas:

1. **Note Features** - `notes.rs` handles core note functionality
2. **TUI Commands** - `commands.rs` processes terminal commands
3. **AI Integration** - `ai.rs` contains AI-related functionality
4. **Web API** - `web.rs` defines the web API endpoints
5. **Web UI Components** - React components in `web-ui/src/components`

### Design Principles

When contributing, keep these principles in mind:

1. **Simplicity** - Keep implementations as simple as possible
2. **Modularity** - Maintain clear component boundaries
3. **Performance** - Consider the impact on performance, especially for large note collections
4. **User Experience** - Prioritize intuitive interfaces and workflows

## Testing

### Running Tests

```bash
# Run all Rust tests
cargo test

# Run specific tests
cargo test test_name

# Run web UI tests
cd web-ui
bun test
```

### Writing Tests

- Unit tests for Rust code should be in the same file as the code they test
- Integration tests should go in the `tests/` directory
- React component tests should be co-located with the components

## Documentation

### Code Documentation

- Document all public functions, structs, and traits with rustdoc comments
- Use examples in documentation where helpful

### Project Documentation

- Update relevant markdown files when adding or changing features
- Keep the README.md up to date with installation and usage instructions

## Release Process

NRS follows semantic versioning:

- MAJOR version for incompatible API changes
- MINOR version for backwards-compatible functionality
- PATCH version for backwards-compatible bug fixes

## Getting Help

If you have questions about contributing, please:

1. Check existing issues and documentation
2. Create a new issue if you can't find an answer

We appreciate your contributions to making NRS better!