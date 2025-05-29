# Testing Setup for picmrg

This document describes the comprehensive unit testing setup for the picmrg image merger application.

## Testing Framework

We use **Rust's built-in testing framework**. Please use the `cargo test` command for running tests

## Test Structure

### Test Utilities (`test_utils.rs`)
- **`generate_test_image()`**: Creates colored test images with specific dimensions
- **`setup_test_data_for_test()`**: Sets up isolated test directory structures
- **`cleanup_test_data_for_test()`**: Cleans up test data after tests complete

### Scanner Module Tests (`scanner.rs`)
- **`test_is_image_file()`**: Tests file extension recognition for various image formats
- **`test_is_merged_file()`**: Tests merged file pattern matching (merged.png, merged-YY-MM-DD.png)
- **`test_scan_for_images_with_test_data()`**: Tests directory scanning with realistic test data
- **`test_find_image_files_ordering()`**: Tests that files are returned in sorted order
- **`test_scan_nonexistent_directory()`**: Tests error handling for invalid paths
- **`test_find_image_files_empty_directory()`**: Tests handling of empty directories
- **`test_find_image_files_no_images()`**: Tests directories with non-image files

### Merger Module Tests (`merger.rs`)
- **`test_determine_merge_orientation()`**: Tests orientation detection logic
- **`test_is_merged_file()`**: Tests merged file detection
- **`test_load_image_info()`**: Tests image loading and metadata extraction
- **`test_resize_to_height()`** / **`test_resize_to_width()`**: Tests aspect ratio preservation
- **`test_merge_horizontally()`** / **`test_merge_vertically()`**: Tests image merging algorithms
- **`test_merge_images_in_directory_**()`**: Tests various merge scenarios (success, single image, no images)
- **`test_find_latest_creation_date()`**: Tests file date detection
- **`test_remove_existing_merged_files()`**: Tests cleanup of old merged files

## Test Data Structure

Each test creates its own isolated directory structure:
```
test-data-{test_name}/
├── vertical-images/       # Images taller than wide (red.png, green.jpg, blue.jpeg)
├── horizontal-images/     # Images wider than tall (yellow.png, cyan.bmp, magenta.tiff)
├── mixed-images/          # Mix of orientations (white.png, black.png, gray.webp)
├── single-image/          # Only one image (orange.png)
├── empty-dir/             # Empty directory
└── no-images/             # Non-image files (readme.txt, data.json)
```

## Generated Test Images

- **Known dimensions**: Vertical (200x400), horizontal (400x200), square (200x200)
- **Known colors**: Solid color fills (red, green, blue, yellow, cyan, magenta, etc.)
- **Various formats**: PNG, JPG, JPEG, BMP, TIFF, WebP
- **Merged files**: Pre-existing merged.png and merged-YY-MM-DD.png files for testing exclusion

## Test Coverage

The test suite covers:
- ✅ File type detection
- ✅ Directory scanning
- ✅ Image orientation detection
- ✅ Image resizing with aspect ratio preservation
- ✅ Horizontal and vertical merging algorithms
- ✅ Merged file pattern recognition
- ✅ Error handling (empty dirs, single images, invalid paths)
- ✅ File cleanup operations
- ✅ Date extraction from file metadata
- ✅ Integration testing with realistic directory structures
