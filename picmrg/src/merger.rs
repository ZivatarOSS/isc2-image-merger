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