# Translator

A modern cross-platform desktop translation application built with **Tauri** and **Angular**. Translate text between 15 languages instantly with a clean, responsive interface.

![Translator App](https://via.placeholder.com/800x400?text=Translator+App+Screenshot)

## Features

- **15 Languages Supported**: English, Spanish, French, German, Italian, Portuguese, Russian, Japanese, Korean, Chinese, Arabic, Hindi, Dutch, Polish, Turkish
- **Dark/Light Theme**: Toggle between dark and light modes
- **Language Swap**: Quickly swap source and target languages
- **Clipboard Support**: Copy translations to clipboard with one click
- **Character Limit**: 5000 character limit per translation
- **Real-time Translation**: Instant translation as you type
- **Cross-Platform**: Runs on Linux, macOS, Windows, and Android

## Tech Stack

- **Frontend**: Angular 20 with TailwindCSS
- **Backend**: Tauri 2 (Rust)
- **Translation Engine**: translate-shell

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Bun](https://bun.sh/) package manager
- [Rust](https://www.rust-lang.org/) toolchain
- [translate-shell](https://github.com/soimort/translate-shell) (`trans` command)

### Install translate-shell

**Ubuntu/Debian:**
```bash
sudo apt-get install translate-shell
```

**macOS:**
```bash
brew install translate-shell
```

**From source:**
```bash
git clone https://github.com/soimort/translate-shell
cd translate-shell
make
sudo make install
```

## Getting Started

### Installation

```bash
# Install dependencies
bun install

# Install Tauri CLI
bun add -D @tauri-apps/cli

# Install Rust dependencies
cargo install tauri-cli
```

### Development

```bash
# Run in development mode (frontend + Tauri dev server)
bun run tauri:dev

# Or run frontend separately
bun start
```

### Build

```bash
# Build for desktop (Linux)
bun run build:smart

# Build for Android
bun run build:smart:android

# Build optimized production version
bun run build:optimized
```

## Project Structure

```
translator/
├── src/                    # Angular frontend
│   ├── app/
│   │   ├── components/     # Reusable UI components
│   │   ├── views/          # Page views
│   │   ├── services/       # Angular services
│   │   ├── models/         # TypeScript models
│   │   └── helpers/        # Utility functions
│   └── index.html
├── src-tauri/              # Tauri/Rust backend
│   ├── src/
│   │   ├── helpers/        # Rust helper modules
│   │   ├── routes/         # API routes
│   │   ├── services/       # Rust services
│   │   └── models/         # Rust models
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/                # Build scripts
├── flatpak/                # Flatpak configuration
└── package.json
```

## Available Languages

| Code | Language |
|------|----------|
| en   | English  |
| es   | Spanish  |
| fr   | French   |
| de   | German   |
| it   | Italian  |
| pt   | Portuguese |
| ru   | Russian  |
| ja   | Japanese |
| ko   | Korean   |
| zh   | Chinese  |
| ar   | Arabic   |
| hi   | Hindi    |
| nl   | Dutch    |
| pl   | Polish   |
| tr   | Turkish  |

## Supported Platforms

- Linux (DEB, AppImage, Flatpak)
- macOS
- Windows
- Android

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Build smaller, faster, and more secure desktop applications
- [Angular](https://angular.io/) - The modern web developer's platform
- [translate-shell](https://github.com/soimort/translate-shell) - Command-line translator
- [TailwindCSS](https://tailwindcss.com/) - Utility-first CSS framework
