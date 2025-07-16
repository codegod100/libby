# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

This project uses [just](https://github.com/casey/just) as the command runner. Key commands:

- `just` or `just build-release` - Build with release profile
- `just build-debug` - Build with debug profile  
- `just run` - Build and run the application
- `just check` - Run clippy linter with pedantic warnings
- `just check-json` - Run clippy with JSON output for IDE integration
- `just clean` - Run cargo clean
- `just install` - Install application files to system
- `just vendor` - Create vendored dependency tarball
- `just build-vendored` - Build with vendored dependencies

## Architecture

This is a COSMIC desktop application built with Rust using the libcosmic framework. The architecture follows COSMIC's application patterns:

### Core Structure
- **main.rs**: Application entry point, sets up i18n and cosmic app settings
- **app.rs**: Main application model implementing `cosmic::Application` trait
  - Contains `AppModel` with navigation, context pages, and configuration
  - Handles messages, view rendering, and subscriptions
- **config.rs**: Application configuration using COSMIC config system
- **i18n.rs**: Internationalization setup using Fluent

### Key Components
- Uses COSMIC's navigation bar with three placeholder pages
- Context drawer for displaying additional content (About page)
- Configuration persistence through COSMIC config system
- Fluent-based localization with fallback to English

### Application Details
- App ID: `com.github.codegod100.libby`
- Based on COSMIC app template structure
- Includes standard desktop integration (icons, .desktop file, appdata)
- Uses vergen for build-time git information

## Dependencies
Main dependencies include libcosmic (GUI framework), tokio (async runtime), i18n-embed (localization), and open (URL handling). The app uses COSMIC's theming, windowing, and configuration systems.

## Development Notes
The codebase follows COSMIC application conventions. When adding features, follow the existing message-passing pattern and use COSMIC widgets. Configuration changes should use the `Config` struct and COSMIC's config watching system.