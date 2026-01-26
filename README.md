# Translator

A modern cross-platform desktop translation application built with **Tauri** and **Angular**. Translate text between 15 languages instantly with a clean, responsive interface powered by `translate-shell`.

![Translator App](https://via.placeholder.com/800x400?text=Translator+App+Screenshot)

## Overview

Translator is a lightweight, fast, and secure desktop application that leverages the power of Rust (via Tauri) and the flexibility of Angular to provide a seamless translation experience. It uses the `trans` command-line utility from `translate-shell` as its backend engine, ensuring reliable and accurate translations across multiple providers.

## Features

- **🚀 Real-time Translation**: Instant translation as you type with a built-in 500ms debounce to optimize performance.
- **🌍 15 Languages Supported**: English, Spanish, French, German, Italian, Portuguese, Russian, Japanese, Korean, Chinese, Arabic, Hindi, Dutch, Polish, and Turkish.
- **🌓 Dark/Light Mode**: Automatic and manual toggle between dark and light themes for a comfortable viewing experience.
- **🔄 Language Swap**: One-click button to quickly swap between source and target languages.
- **📋 Clipboard Integration**: Copy your translations to the clipboard instantly with dedicated UI feedback.
- **⌨️ Keyboard Shortcuts**: Support for `Ctrl+Enter` (or `Cmd+Enter`) to force a translation immediately.
- **📱 Cross-Platform**: Native support for Linux, macOS, Windows, and experimental support for Android.

## Tech Stack

- **Frontend**: [Angular 20](https://angular.io/) with [TailwindCSS 4](https://tailwindcss.com/)
- **Backend**: [Tauri 2](https://tauri.app/) (Rust)
- **Engine**: [translate-shell](https://github.com/soimort/translate-shell) (`trans` CLI)
- **Package Manager**: [Bun](https://bun.sh/)

## Prerequisites

To build or run this application from source, you need:

1.  **Node.js** (v18+) & **Bun**
2.  **Rust** toolchain
3.  **translate-shell** installed and accessible in your system PATH as `trans`.

### Installing translate-shell

- **Ubuntu/Debian**: `sudo apt-get install translate-shell`
- **macOS**: `brew install translate-shell`
- **Arch Linux**: `sudo pacman -S translate-shell`

## Getting Started

### Installation

```bash
# Install frontend and development dependencies
bun install

# Install Tauri CLI globally (optional but recommended)
cargo install tauri-cli
```

### Development

```bash
# Run in development mode (with hot-reload)
bun run tauri:dev
```

### Build

We provide a specialized build script for optimized production releases:

```bash
# Build for Desktop (Linux/Windows/macOS)
bun run build:smart

# Build for Android
bun run build:smart:android

# Clean build artifacts
bun run build:clean
```

## How It Works

1.  **Frontend**: The Angular application captures user input and sends it to the Rust backend using Tauri's `invoke` system.
2.  **Backend**: The Rust service receives the text and spawns an asynchronous task to execute the `trans` command.
3.  **Communication**: Once `translate-shell` provides the output, the Rust backend emits a `translation-result` event back to the frontend.
4.  **UI Update**: The Angular component listens for these events and updates the display accordingly, handling request IDs to ensure results are matched to the correct query.

## Project Structure

```text
.
├── src/                    # Angular Frontend
│   ├── app/
│   │   ├── components/     # UI Building Blocks (Buttons, Input, etc.)
│   │   ├── services/       # Translation and State Logic
│   │   ├── views/          # Main Page Layouts
│   │   └── models/         # TypeScript Interfaces
│   └── styles.css          # Global TailwindCSS Imports
├── src-tauri/              # Rust Backend
│   ├── src/
│   │   ├── helpers/        # Command Execution Helpers
│   │   ├── services/       # Translation Logic and State Management
│   │   └── lib.rs          # Tauri Command Definitions
│   └── tauri.conf.json     # Tauri Configuration
├── scripts/                # Utility and Build Optimization Scripts
└── flatpak/                # Manifests for Flatpak Distribution
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

## License

This project is licensed under the MIT License.

## Acknowledgments

- Thanks to the [translate-shell](https://github.com/soimort/translate-shell) project for the amazing CLI tool.
- [Tauri](https://tauri.app/) for making desktop apps with Rust easy.

