# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Focus is a Tauri-based desktop application that provides a spotlight/launcher-style interface for quick access to files, applications, and system functionality through a searchable command palette.

## Development Commands

```bash
# Frontend development
pnpm dev          # Start development server (runs on port 1420)
pnpm build        # Build frontend with TypeScript checking (vue-tsc --noEmit && vite build)
pnpm preview      # Preview production build

# Tauri commands
pnpm tauri dev    # Run Tauri development build
pnpm tauri build  # Build Tauri application

# Rust development
cargo check       # Check Rust code for errors
cargo build       # Build Rust code
cargo test        # Run Rust tests

# Commit conventions
npx cz            # Use commitizen for conventional commits with emoji support
```

## Architecture

### Frontend (Vue 3 + TypeScript)
- **Framework**: Vue 3 with Composition API and `<script setup>` SFCs
- **Router**: Vue Router 4 with lazy-loaded routes: `/`, `/query`, `/settings`
- **Styling**: Tailwind CSS 4 with DaisyUI components
- **Build**: Vite 6 with strict TypeScript configuration

### Backend (Rust + Tauri)

#### Plugin System Architecture
The application uses a command tree architecture where plugins register commands with the `CommandDispatcher`. Each plugin implements the `Extension` trait from `src-tauri/src/api/extension.rs`:

```rust
pub trait Extension {
    fn OnMount(&self, command_dispatcher: &mut CommandDispatcher);
    fn OnUnmount(&self, command_dispatcher: &mut CommandDispatcher);
    fn get_meta_data(&self) -> MetaData;
}
```

#### Command Tree System
Located in `src-tauri/src/api/command_tree.rs`, the command tree supports:
- **Literal nodes**: Fixed command keywords
- **Parameter nodes**: Dynamic arguments with custom parsers
- **Truncation**: Capture remaining input as a single parameter
- **Nested commands**: Hierarchical command structure

Plugins register commands using the builder pattern:
```rust
CommandNode::new("command")
    .then(CommandNode::new("subcommand"))
    .execute(|ctx, app| { /* callback */ })
```

#### Built-in Plugins
Located in `src-tauri/src/plugins/`:
- **AppPlugin** (`app_plugin.rs`): Application management (restart, stop)
- **FilePlugin** (`file_plugin.rs`): File search using Everything SDK with icon extraction
- **LauncherPlugin** (`launcher_plugin.rs`): Application launcher with fuzzy search
- **CalculatorPlugin** (`cal_plugin.rs`): Calculator functionality
- **DemoPlugin** (`demo_plugin.rs`): Example plugin

#### Action System
Actions are registered with `ActionRunner` (singleton pattern in `src-tauri/src/api/action_runner.rs`). Each plugin can register action handlers that are triggered from the frontend. Actions are associated with search results and executed via the `run_action` Tauri command.

### Key Components

#### Frontend
- **QueryBox** (`src/components/QueryBox.vue`): Main search input with cursor position tracking
- **ResultItem** (`src/components/ResultItem.vue`): Display component for search results with icons, titles, descriptions, and actions
- **ActionsBox** (`src/components/ActionsBox.vue`): Handles result actions
- **QueryPage** (`src/pages/QueryPage.vue`): Main query interface with keyboard navigation (arrows, enter)

#### Backend Communication
- Frontend invokes Tauri commands: `query(inputText)` and `run_action(id, value)`
- Backend returns `Results` struct with `ExtensionResult` items containing actions
- Window auto-resizes based on content using `ResizeObserver`

### Configuration

#### Tauri (`src-tauri/tauri.conf.json`)
- **Window**: 800x380 transparent window with acrylic effects and no decorations
- **System Tray**: Background operation with hide/show/quit menu
- **Global Shortcuts**: Registered via `api::register_globals_shortcut()` (desktop only)
- **Security**: CSP disabled, uses Tauri filesystem and clipboard plugins

#### Build Configuration
- **Before dev**: `pnpm dev` (runs on port 1420)
- **Before build**: `pnpm build` (outputs to `../dist`)
- **Bundle resources**: Copies `libs/*` to bundle

#### Commit Configuration
Uses commitlint with cz-git for conventional commits with emoji support. Run `npx cz` or configure your IDE to use the commitizen CLI.

## Important Notes

- Everything SDK integration requires Everything search service to be running on Windows
- Fuzzy search is implemented using the `rust-fuzzy-search` crate (version 0.1.1)
- Plugins have priority levels (default: 100) for ordering search results via `MetaData`
- The command tree uses a "/" prefix by default, configurable in `CommandDispatcher::new()`
- All plugins must be manually registered in `src-tauri/src/lib.rs` setup function
- File icons are extracted using Windows API and converted to base64 for display
- Window automatically hides on blur (focus lost) via `TauriEvent.WINDOW_BLUR`