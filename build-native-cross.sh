#!/bin/bash

# Alternative build script using native Rust cross-compilation
# This approach uses rustup targets and system linkers

cd picmrg || exit 1

echo "Setting up cross-compilation targets..."

# Add required targets
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-musl
rustup target add armv7-unknown-linux-musleabihf

echo "Building for multiple targets..."

# Build for macOS (native)
echo "Building for macOS (native)..."
cargo build --release

# Build for Windows (requires mingw-w64)
echo "Building for Windows..."
if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    cargo build --target x86_64-pc-windows-gnu --release
else
    echo "Warning: mingw-w64 not found. Install with: brew install mingw-w64"
    echo "Skipping Windows build..."
fi

# Build for Linux x86_64 (requires musl-cross)
echo "Building for Linux x86_64..."
if command -v musl-gcc &> /dev/null; then
    CC=musl-gcc cargo build --target x86_64-unknown-linux-musl --release
else
    echo "Warning: musl-cross not found. Install with: brew install FiloSottile/musl-cross/musl-cross"
    echo "Skipping Linux x86_64 build..."
fi

echo "Build complete! Note: Some targets may have been skipped due to missing dependencies."
echo ""
echo "To install missing dependencies:"
echo "  brew install mingw-w64                           # For Windows builds"
echo "  brew install FiloSottile/musl-cross/musl-cross   # For Linux builds"
echo ""
echo "For ARM targets, consider using the Docker-based approach (build-all-targets.sh)" 