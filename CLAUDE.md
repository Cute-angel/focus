# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Tauri desktop application built with Vue 3, TypeScript, and Rust. The app appears to be a focus/productivity tool that integrates with Everything search for file finding capabilities.

## Key Technologies

- **Frontend**: Vue 3 with Composition API, TypeScript, Tailwind CSS, DaisyUI
- **Backend**: Rust with Tauri v2
- **Build Tool**: Vite
- **File Search**: Everything SDK integration via `everything-rs` crate

## Development Commands

### Frontend Development

```bash
npm run dev          # Start development server (Vite on port 1420)
npm run build        # Build for production (runs TypeScript check then Vite build)
npm run preview      # Preview production build
```

### Tauri Development

```bash
npm run tauri dev    # Run Tauri app in development mode
npm run tauri build  # Build Tauri app for production
```

### TypeScript Checking

The build process includes `vue-tsc --noEmit` for type checking before building.

## Architecture

### Frontend Structure

- **Entry Point**: `src/main.ts` - Sets up Vue Router and app initialization
- **Main Component**: `src/App.vue` - Contains the primary application interface
- **API Layer**: `src/api/` - Contains Tauri API bindings
  - `window.ts` - Window management functions for creating new windows/webviews
- **Routing**: Vue Router with `/settings` route pointing to `SettingPage.vue`

### Backend Structure

- **Entry Point**: `src-tauri/src/main.rs` - Calls `focus_lib::run()`
- **Core Logic**: `src-tauri/src/lib.rs` - Tauri app setup with command handlers
- **Commands**: `src-tauri/src/commands.rs` - Rust command implementations
  - `greet()` - Simple greeting command
  - `set_text()` - Text processing command
  - `get_file_finder_result()` - Everything SDK integration for file search
- **Dependencies**: Uses Everything SDK for file system search capabilities

### Key Features

- **Multi-window Support**: Can create additional windows/webviews via `window_start()` function
- **File Search**: Integration with Everything search tool for finding files and folders
- **Real-time Communication**: Watch-based communication between frontend input and backend processing

## Development Notes

- The Vite dev server runs on a fixed port 1420 (required by Tauri)
- Frontend HMR runs on port 1421 when `TAURI_DEV_HOST` is set
- The project uses `lazy_static` for Everything SDK instance management
- Error handling uses `thiserror` for custom error types with proper serialization
- The Everything64.dll is included in the project for Windows file search functionality
- 项目使用的包管理器是pnpm 不是npm
- 这个项目是一个类似与spotlight的软件