# Flatpak Build for Translator

## Prerequisites

Install flatpak and flatpak-builder:

```bash
# Ubuntu/Debian
sudo apt install flatpak flatpak-builder

# Add flathub
flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
```

## Build

### Option 1: Build app then package (default)

```bash
cd flatpak
./build.sh build
```

### Option 2: Package existing binary

If you already have a built binary at `target/release/translator`:

```bash
cd flatpak
./build.sh no-build
```

## Install

```bash
flatpak install translator-{VERSION}.flatpak
```

## Run

```bash
flatpak run com.tcs.translator
```

## Uninstall

```bash
flatpak uninstall --user com.tcs.translator
```

## Notes

- The manifest uses GNOME 49 runtime
- Binary is installed to `/app/bin/translator`
- App ID: `com.tcs.translator`
