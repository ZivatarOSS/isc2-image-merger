#!/bin/bash

# Build script for cross-platform compilation
# This script builds for multiple targets using Docker for Linux builds

cd picmrg || exit 1

echo "Building for multiple targets..."

# Build for macOS (native)
echo "Building for macOS (native)..."
cargo build --release

# Build for Windows using cross
echo "Building for Windows..."
if command -v cross &> /dev/null; then
    cross build --target x86_64-pc-windows-gnu --release
else
    echo "Installing cross for Windows builds..."
    cargo install cross --git https://github.com/cross-rs/cross
    cross build --target x86_64-pc-windows-gnu --release
fi

# Build for Linux x86_64 using cross
echo "Building for Linux x86_64..."
cross build --target x86_64-unknown-linux-musl --release

# Build for Linux ARM64 using cross
echo "Building for Linux ARM64..."
cross build --target aarch64-unknown-linux-musl --release

# Build for ARMv7 (Raspberry Pi) using cross
echo "Building for ARMv7 (Raspberry Pi)..."
cross build --target armv7-unknown-linux-musleabihf --release

echo "All builds complete! Check the 'target' directory."
echo ""
echo "Built targets:"
echo "- macOS: target/release/picmrg"
echo "- Windows: target/x86_64-pc-windows-gnu/release/picmrg.exe"
echo "- Linux x86_64: target/x86_64-unknown-linux-musl/release/picmrg"
echo "- Linux ARM64: target/aarch64-unknown-linux-musl/release/picmrg"
echo "- Linux ARMv7: target/armv7-unknown-linux-musleabihf/release/picmrg"