# Image Merger (picmrg)

A command-line tool that automatically merges multiple images from subdirectories into single composite images. Designed for preparing evidence images for ISC2 CPE (Continuing Professional Education) credit submissions (but might be useful for others, too).

## Usage

Download the executable for your platform from the latest release or build the tool from source.

### Basic Usage

```bash
# Process current directory
./picmrg

# Process specific directory
./picmrg /path/to/images

# Show help
./picmrg -h
```

> **IMPORTANT**: picmrg will consider all images called merge.png or merged-24-04-01.png and similar as it's own previously generated merges and **WILL OVERWRITE THEM**. Do not use picmrg on directories that contain images with these names or rename them.

### Directory Structure

The tool expects a directory structure with groups of pictures in subdirectories:

```
root-directory/
├── 2024-01-15/
│   ├── screenshot1.png
│   ├── screenshot2.png
│   └── screenshot3.png
├── 2024-01-22/
│   ├── image1.jpg
│   └── image2.jpg
└── 2024-02-01/
    ├── cert1.png
    ├── cert2.png
    └── cert3.png
```

### Output

For each subdirectory containing 2 or more images, the tool creates a merged file named `merged-YY-MM-DD.png` where the date represents the latest creation/modification date of the source images.

Example output:
```
2024-01-15/
├── screenshot1.png
├── screenshot2.png
├── screenshot3.png
└── merged-24-01-16.png  ← Generated merged file
```

### Example Session

```bash
$ ./picmrg /Users/john/cpe-evidence
picmrg v1.0.0: image merger

Root path: /Users/john/cpe-evidence
✓ Successfully merged images in 2024-01-15
✓ Successfully merged images in 2024-01-22
- Skipped 2024-02-01 (only one image)
✓ Successfully merged images in 2024-02-15

Merging complete!
```

### Getting Help

Run `./picmrg -h` for quick usage information, or refer to this README for comprehensive documentation.


## What It Does

The Image Merger tool:

1. **Scans directories** - Looks for subdirectories containing image files
2. **Analyzes image orientation** - Determines whether images are primarily vertical or horizontal
3. **Merges intelligently** - Combines images based on their orientation:
   - Vertical images → Merged horizontally (side by side)
   - Horizontal images → Merged vertically (stacked)
4. **Preserves quality** - Resizes images proportionally to maintain aspect ratios
5. **Timestamps output** - Names merged files with the latest creation date from source images
6. **Cleans up** - Removes old merged files before creating new ones

## Supported Image Formats

- JPEG/JPG
- PNG
- GIF
- BMP
- TIFF/TIF
- WebP

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Cargo (comes with Rust)

### Building from Source

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd isc2-image-merger
   ```

2. Build the project:
   ```bash
   cd picmrg
   cargo build --release
   ```

3. The executable will be available at `picmrg/target/release/picmrg`

### Cross-Platform Builds

Use the provided build script to create binaries for multiple platforms:

```bash
./build-all-targets.sh
```

This creates binaries for:
- macOS (native)
- Windows 64-bit
- Linux 64-bit (Intel)
- Linux 64-bit (ARM)
- ARMv7 (Raspberry Pi)


## License

See the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

