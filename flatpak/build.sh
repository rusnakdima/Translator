#!/bin/bash

# Build script for Flatpak Linux
# This script builds your Dioxus app as a Flatpak package
# Usage: ./build.sh [build|no-build]
#   build: Build the app first then create Flatpak (default)
#   no-build: Skip building and proceed with Flatpak packaging using existing binary

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

BUILD_APP="${1:-build}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ "$BUILD_APP" != "build" ] && [ "$BUILD_APP" != "no-build" ]; then
	echo -e "${RED}Error: Invalid build option '$BUILD_APP'${NC}"
	echo "Usage: $0 [build|no-build]"
	exit 1
fi

if ! command -v flatpak &>/dev/null; then
	echo -e "${RED}Error: flatpak is not installed${NC}"
	echo "Install it with: sudo apt install flatpak"
	exit 1
fi

if ! command -v flatpak-builder &>/dev/null; then
	echo -e "${RED}Error: flatpak-builder is not installed${NC}"
	echo "Install it with: sudo apt install flatpak-builder"
	exit 1
fi

if [ "$BUILD_APP" = "build" ]; then
	if ! command -v cargo &>/dev/null; then
		echo -e "${RED}Error: cargo is not installed for build${NC}"
		echo "Install Rust from: https://rustup.rs"
		exit 1
	fi
fi

APP_ID="com.tcs.translator"
MANIFEST="${APP_ID}.yml"
BUILD_DIR="./build"
REPO_DIR="./repo"

echo -e "${YELLOW}Step 1: Installing required runtimes...${NC}"
flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo || true
flatpak install -y --user flathub org.gnome.Platform//49 org.gnome.Sdk//49 || true

if [ "$BUILD_APP" = "build" ]; then
	echo -e "${YELLOW}Step 2: Building Dioxus app...${NC}"
	cd ..
	echo "Building app with cargo..."
	cargo build --release
	VERSION="${2:-$(grep '^version' Cargo.toml | awk '{print $3}' | tr -d '"')}"
	echo "App built. Proceeding with Flatpak packaging..."
	cd "$SCRIPT_DIR"
elif [ "$BUILD_APP" = "no-build" ]; then
	cd ..
	VERSION="${2:-$(grep '^version' Cargo.toml | awk '{print $3}' | tr -d '"')}"
	cd "$SCRIPT_DIR"
	echo -e "${YELLOW}Skipping build step. Using existing binary.${NC}"
fi

echo -e "${YELLOW}Step 3: Building Flatpak...${NC}"
flatpak-builder \
	--disable-cache \
	--force-clean \
	--user \
	--install-deps-from=flathub \
	--repo="${REPO_DIR}" \
	"${BUILD_DIR}" \
	"${MANIFEST}"

echo -e "${YELLOW}Step 4: Creating Flatpak bundle...${NC}"
flatpak build-bundle "${REPO_DIR}" "translator-${VERSION}.flatpak" "${APP_ID}"
echo -e "${GREEN}=== Build Complete! ===${NC}"
echo ""
echo "Created bundle: translator-${VERSION}.flatpak"
echo ""
echo "To install the bundle:"
echo -e "  ${YELLOW}flatpak install translator-${VERSION}.flatpak${NC}"
echo ""
echo "To run your app:"
echo -e "  ${YELLOW}flatpak run ${APP_ID}${NC}"
echo ""
echo "To uninstall:"
echo -e "  ${YELLOW}flatpak uninstall --user ${APP_ID}${NC}"
