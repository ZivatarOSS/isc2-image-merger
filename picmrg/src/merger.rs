use image::{DynamicImage, ImageBuffer, RgbaImage};
use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Local};

#[derive(Debug, Clone, Copy)]
pub enum MergeOrientation {
    Horizontal, // Images side by side
    Vertical,   // Images stacked vertically
}

#[derive(Debug)]
pub struct ImageInfo {
    pub image: DynamicImage,
    pub width: u32,
    pub height: u32,
    pub is_vertical: bool,
}

/// Merge images from a directory based on their orientation
pub fn merge_images_in_directory(
    directory: &Path,
    image_files: &[PathBuf],
) -> Result<(), Box<dyn std::error::Error>> {
    if image_files.is_empty() {
        return Err("No image files to merge".into());
    }
    
    if image_files.len() <= 1 {
        return Err("Only one image file found, skipping merge".into());
    }

    // Find the latest creation date among all image files
    let latest_date = find_latest_creation_date(image_files)?;
    let date_string = latest_date.format("%y-%m-%d").to_string();
    let output_filename = format!("merged-{}.png", date_string);
    let output_path = directory.join(&output_filename);

    // Remove any existing merged files before creating a new one
    remove_existing_merged_files(directory)?;

    // Load all images and analyze their dimensions
    let mut image_infos = Vec::new();
    for file_path in image_files {
        match load_image_info(file_path) {
            Ok(info) => image_infos.push(info),
            Err(e) => {
                eprintln!("Warning: Failed to load {}: {}", file_path.display(), e);
                continue;
            }
        }
    }

    if image_infos.is_empty() {
        return Err("No valid images could be loaded".into());
    }

    // Determine merge orientation based on majority orientation
    let orientation = determine_merge_orientation(&image_infos);

    // Perform the merge
    let merged_image = match orientation {
        MergeOrientation::Horizontal => merge_horizontally(&image_infos)?,
        MergeOrientation::Vertical => merge_vertically(&image_infos)?,
    };

    // Save the result
    merged_image.save(&output_path)?;

    Ok(())
}

/// Find the latest creation date among the image files
fn find_latest_creation_date(image_files: &[PathBuf]) -> Result<DateTime<Local>, Box<dyn std::error::Error>> {
    let mut latest_date: Option<DateTime<Local>> = None;

    for file_path in image_files {
        let metadata = fs::metadata(file_path)?;
        
        // Try to get creation time, fall back to modified time if not available
        let file_time = metadata.created()
            .or_else(|_| metadata.modified())?;
        
        let datetime: DateTime<Local> = file_time.into();
        
        match latest_date {
            None => latest_date = Some(datetime),
            Some(current_latest) => {
                if datetime > current_latest {
                    latest_date = Some(datetime);
                }
            }
        }
    }

    latest_date.ok_or_else(|| "No valid dates found".into())
}

/// Remove any existing merged files in the directory
fn remove_existing_merged_files(directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(directory)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    if is_merged_file(name_str) {
                        fs::remove_file(&path)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Check if a filename is a merged file (merged.png or merged-yy-mm-dd.png)
fn is_merged_file(filename: &str) -> bool {
    if filename == "merged.png" {
        return true;
    }
    
    // Check for merged-yy-mm-dd.png pattern
    if filename.starts_with("merged-") && filename.ends_with(".png") {
        let date_part = &filename[7..filename.len()-4]; // Remove "merged-" and ".png"
        
        // Check if it matches yy-mm-dd pattern (8 characters with dashes at positions 2 and 5)
        if date_part.len() == 8 {
            let chars: Vec<char> = date_part.chars().collect();
            if chars[2] == '-' && chars[5] == '-' {
                // Check if other characters are digits
                let year_part = &date_part[0..2];
                let month_part = &date_part[3..5];
                let day_part = &date_part[6..8];
                
                return year_part.chars().all(|c| c.is_ascii_digit()) &&
                       month_part.chars().all(|c| c.is_ascii_digit()) &&
                       day_part.chars().all(|c| c.is_ascii_digit());
            }
        }
    }
    
    false
}

/// Load an image and extract its information
fn load_image_info(file_path: &Path) -> Result<ImageInfo, Box<dyn std::error::Error>> {
    let image = image::open(file_path)?;
    let width = image.width();
    let height = image.height();
    let is_vertical = height > width;

    Ok(ImageInfo {
        image,
        width,
        height,
        is_vertical,
    })
}

/// Determine merge orientation based on majority of image orientations
fn determine_merge_orientation(image_infos: &[ImageInfo]) -> MergeOrientation {
    let vertical_count = image_infos.iter().filter(|info| info.is_vertical).count();
    let horizontal_count = image_infos.len() - vertical_count;

    if vertical_count > horizontal_count {
        MergeOrientation::Horizontal // Vertical images -> horizontal layout
    } else {
        MergeOrientation::Vertical // Horizontal images -> vertical layout
    }
}

/// Merge images horizontally (side by side)
fn merge_horizontally(image_infos: &[ImageInfo]) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    // Find the tallest height
    let target_height = image_infos.iter().map(|info| info.height).max().unwrap();
    
    // Calculate total width needed
    let mut total_width = 0u32;
    let mut resized_images = Vec::new();

    for info in image_infos {
        let resized = resize_to_height(&info.image, target_height);
        total_width += resized.width();
        resized_images.push(resized);
    }

    // Create the output image
    let mut output: RgbaImage = ImageBuffer::new(total_width, target_height);
    
    let mut x_offset = 0;
    for resized_image in resized_images {
        let rgba_image = resized_image.to_rgba8();
        
        for (x, y, pixel) in rgba_image.enumerate_pixels() {
            output.put_pixel(x_offset + x, y, *pixel);
        }
        
        x_offset += resized_image.width();
    }

    Ok(DynamicImage::ImageRgba8(output))
}

/// Merge images vertically (stacked)
fn merge_vertically(image_infos: &[ImageInfo]) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    // Find the widest width
    let target_width = image_infos.iter().map(|info| info.width).max().unwrap();
    
    // Calculate total height needed
    let mut total_height = 0u32;
    let mut resized_images = Vec::new();

    for info in image_infos {
        let resized = resize_to_width(&info.image, target_width);
        total_height += resized.height();
        resized_images.push(resized);
    }

    // Create the output image
    let mut output: RgbaImage = ImageBuffer::new(target_width, total_height);
    
    let mut y_offset = 0;
    for resized_image in resized_images {
        let rgba_image = resized_image.to_rgba8();
        
        for (x, y, pixel) in rgba_image.enumerate_pixels() {
            output.put_pixel(x, y_offset + y, *pixel);
        }
        
        y_offset += resized_image.height();
    }

    Ok(DynamicImage::ImageRgba8(output))
}

/// Resize image to match target height while maintaining aspect ratio
fn resize_to_height(image: &DynamicImage, target_height: u32) -> DynamicImage {
    if image.height() == target_height {
        return image.clone();
    }
    
    let aspect_ratio = image.width() as f32 / image.height() as f32;
    let target_width = (target_height as f32 * aspect_ratio) as u32;
    
    image.resize(target_width, target_height, image::imageops::FilterType::Lanczos3)
}

/// Resize image to match target width while maintaining aspect ratio
fn resize_to_width(image: &DynamicImage, target_width: u32) -> DynamicImage {
    if image.width() == target_width {
        return image.clone();
    }
    
    let aspect_ratio = image.height() as f32 / image.width() as f32;
    let target_height = (target_width as f32 * aspect_ratio) as u32;
    
    image.resize(target_width, target_height, image::imageops::FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_data_for_test, cleanup_test_data_for_test, generate_test_image};
    use std::path::Path;
    
    #[test]
    fn test_determine_merge_orientation() {
        // Create test image infos
        let vertical_images = vec![
            ImageInfo {
                image: generate_test_image(100, 200, [255, 0, 0]),
                width: 100,
                height: 200,
                is_vertical: true,
            },
            ImageInfo {
                image: generate_test_image(150, 300, [0, 255, 0]),
                width: 150,
                height: 300,
                is_vertical: true,
            },
        ];
        
        let horizontal_images = vec![
            ImageInfo {
                image: generate_test_image(300, 150, [255, 0, 0]),
                width: 300,
                height: 150,
                is_vertical: false,
            },
            ImageInfo {
                image: generate_test_image(400, 200, [0, 255, 0]),
                width: 400,
                height: 200,
                is_vertical: false,
            },
        ];
        
        let mixed_images = vec![
            ImageInfo {
                image: generate_test_image(100, 200, [255, 0, 0]),
                width: 100,
                height: 200,
                is_vertical: true,
            },
            ImageInfo {
                image: generate_test_image(300, 150, [0, 255, 0]),
                width: 300,
                height: 150,
                is_vertical: false,
            },
            ImageInfo {
                image: generate_test_image(200, 100, [0, 0, 255]),
                width: 200,
                height: 100,
                is_vertical: false,
            },
        ];
        
        // Test majority vertical -> horizontal merge
        assert!(matches!(determine_merge_orientation(&vertical_images), MergeOrientation::Horizontal));
        
        // Test majority horizontal -> vertical merge
        assert!(matches!(determine_merge_orientation(&horizontal_images), MergeOrientation::Vertical));
        
        // Test mixed with majority horizontal -> vertical merge
        assert!(matches!(determine_merge_orientation(&mixed_images), MergeOrientation::Vertical));
    }
    
    #[test]
    fn test_is_merged_file() {
        // Test basic merged file
        assert!(is_merged_file("merged.png"));
        
        // Test dated merged files
        assert!(is_merged_file("merged-23-12-25.png"));
        assert!(is_merged_file("merged-24-01-15.png"));
        
        // Test invalid patterns
        assert!(!is_merged_file("merged.jpg"));
        assert!(!is_merged_file("merged-2023-12-25.png"));
        assert!(!is_merged_file("merged-23-1-25.png"));
        assert!(!is_merged_file("other.png"));
    }
    
    #[test]
    fn test_load_image_info() {
        let test_root = setup_test_data_for_test("load_info").expect("Failed to setup test data");
        
        // Test loading a vertical image
        let vertical_path = Path::new(&test_root).join("vertical-images/red.png");
        let vertical_info = load_image_info(&vertical_path).expect("Failed to load vertical image");
        assert_eq!(vertical_info.width, 200);
        assert_eq!(vertical_info.height, 400);
        assert!(vertical_info.is_vertical);
        
        // Test loading a horizontal image
        let horizontal_path = Path::new(&test_root).join("horizontal-images/yellow.png");
        let horizontal_info = load_image_info(&horizontal_path).expect("Failed to load horizontal image");
        assert_eq!(horizontal_info.width, 400);
        assert_eq!(horizontal_info.height, 200);
        assert!(!horizontal_info.is_vertical);
        
        // Test loading a square image
        let square_path = Path::new(&test_root).join("mixed-images/gray.webp");
        let square_info = load_image_info(&square_path).expect("Failed to load square image");
        assert_eq!(square_info.width, 200);
        assert_eq!(square_info.height, 200);
        assert!(!square_info.is_vertical); // height == width, so not vertical
        
        cleanup_test_data_for_test(&test_root).expect("Failed to cleanup test data");
    }
    
    #[test]
    fn test_resize_to_height() {
        let image = generate_test_image(100, 200, [255, 0, 0]);
        
        // Test resizing to same height (should return clone)
        let same_height = resize_to_height(&image, 200);
        assert_eq!(same_height.width(), 100);
        assert_eq!(same_height.height(), 200);
        
        // Test resizing to different height (should maintain aspect ratio)
        let resized = resize_to_height(&image, 400);
        assert_eq!(resized.height(), 400);
        // Aspect ratio: 100/200 = 0.5, so new width should be 400 * 0.5 = 200
        assert_eq!(resized.width(), 200);
    }
    
    #[test]
    fn test_resize_to_width() {
        let image = generate_test_image(200, 100, [0, 255, 0]);
        
        // Test resizing to same width (should return clone)
        let same_width = resize_to_width(&image, 200);
        assert_eq!(same_width.width(), 200);
        assert_eq!(same_width.height(), 100);
        
        // Test resizing to different width (should maintain aspect ratio)
        let resized = resize_to_width(&image, 400);
        assert_eq!(resized.width(), 400);
        // Aspect ratio: 100/200 = 0.5, so new height should be 400 * 0.5 = 200
        assert_eq!(resized.height(), 200);
    }
    
    #[test]
    fn test_merge_horizontally() {
        let image_infos = vec![
            ImageInfo {
                image: generate_test_image(100, 200, [255, 0, 0]), // Red
                width: 100,
                height: 200,
                is_vertical: true,
            },
            ImageInfo {
                image: generate_test_image(150, 300, [0, 255, 0]), // Green
                width: 150,
                height: 300,
                is_vertical: true,
            },
        ];
        
        let merged = merge_horizontally(&image_infos).expect("Failed to merge horizontally");
        
        // Should use the tallest height (300) and sum up widths proportionally
        assert_eq!(merged.height(), 300);
        // First image: 100 * (300/200) = 150 width
        // Second image: 150 * (300/300) = 150 width  
        // Total: 150 + 150 = 300
        assert_eq!(merged.width(), 300);
    }
    
    #[test]
    fn test_merge_vertically() {
        let image_infos = vec![
            ImageInfo {
                image: generate_test_image(200, 100, [255, 0, 0]), // Red
                width: 200,
                height: 100,
                is_vertical: false,
            },
            ImageInfo {
                image: generate_test_image(300, 150, [0, 255, 0]), // Green
                width: 300,
                height: 150,
                is_vertical: false,
            },
        ];
        
        let merged = merge_vertically(&image_infos).expect("Failed to merge vertically");
        
        // Should use the widest width (300) and sum up heights proportionally
        assert_eq!(merged.width(), 300);
        // First image: 100 * (300/200) = 150 height
        // Second image: 150 * (300/300) = 150 height
        // Total: 150 + 150 = 300
        assert_eq!(merged.height(), 300);
    }
    
    #[test]
    fn test_merge_images_in_directory_single_image() {
        let test_root = setup_test_data_for_test("single").expect("Failed to setup test data");
        
        let single_dir = Path::new(&test_root).join("single-image");
        let image_files = vec![single_dir.join("orange.png")];
        
        let result = merge_images_in_directory(&single_dir, &image_files);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Only one image file"));
        
        cleanup_test_data_for_test(&test_root).expect("Failed to cleanup test data");
    }
    
    #[test]
    fn test_merge_images_in_directory_no_images() {
        let test_root = setup_test_data_for_test("merger_no_images").expect("Failed to setup test data");
        
        let empty_dir = Path::new(&test_root).join("empty-dir");
        let image_files: Vec<PathBuf> = vec![];
        
        let result = merge_images_in_directory(&empty_dir, &image_files);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No image files to merge"));
        
        cleanup_test_data_for_test(&test_root).expect("Failed to cleanup test data");
    }
    
    #[test]
    fn test_merge_images_in_directory_success() {
        let test_root = setup_test_data_for_test("success").expect("Failed to setup test data");
        
        let vertical_dir = Path::new(&test_root).join("vertical-images");
        let image_files = vec![
            vertical_dir.join("red.png"),
            vertical_dir.join("green.jpg"),
            vertical_dir.join("blue.jpeg"),
        ];
        
        // Remove any existing merged files first
        let _ = remove_existing_merged_files(&vertical_dir);
        
        let result = merge_images_in_directory(&vertical_dir, &image_files);
        assert!(result.is_ok(), "Failed to merge images: {:?}", result);
        
        // Check that a merged file was created
        let merged_files: Vec<_> = std::fs::read_dir(&vertical_dir)
            .expect("Failed to read directory")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_str()?.to_string();
                if name.starts_with("merged-") && name.ends_with(".png") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        
        assert_eq!(merged_files.len(), 1, "Expected exactly one merged file");
        
        // Verify the merged file exists and is a valid image
        let merged_path = vertical_dir.join(&merged_files[0]);
        assert!(merged_path.exists());
        let merged_image = image::open(&merged_path);
        assert!(merged_image.is_ok(), "Merged file should be a valid image");
        
        cleanup_test_data_for_test(&test_root).expect("Failed to cleanup test data");
    }
    
    #[test]
    fn test_find_latest_creation_date() {
        let test_root = setup_test_data_for_test("date").expect("Failed to setup test data");
        
        let image_files = vec![
            Path::new(&test_root).join("vertical-images/red.png"),
            Path::new(&test_root).join("vertical-images/green.jpg"),
        ];
        
        let result = find_latest_creation_date(&image_files);
        assert!(result.is_ok(), "Should find a valid date");
        
        cleanup_test_data_for_test(&test_root).expect("Failed to cleanup test data");
    }
    
    #[test]
    fn test_remove_existing_merged_files() {
        let test_root = setup_test_data_for_test("remove").expect("Failed to setup test data");
        
        let test_dir = Path::new(&test_root).join("vertical-images");
        
        // Verify merged files exist initially
        assert!(test_dir.join("merged.png").exists());
        
        // Remove merged files
        let result = remove_existing_merged_files(&test_dir);
        assert!(result.is_ok(), "Should successfully remove merged files");
        
        // Verify merged files are gone
        assert!(!test_dir.join("merged.png").exists());
        
        // Verify regular image files still exist
        assert!(test_dir.join("red.png").exists());
        assert!(test_dir.join("green.jpg").exists());
        assert!(test_dir.join("blue.jpeg").exists());
        
        cleanup_test_data_for_test(&test_root).expect("Failed to cleanup test data");
    }
} 