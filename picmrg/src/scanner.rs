use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Represents the result of scanning directories for image files
#[derive(Debug)]
pub struct ScanResult {
    pub directories: HashMap<String, Vec<PathBuf>>,
}

/// Find all directories one level down from the root path and collect image files within them
pub fn scan_for_images(root_path: &Path) -> Result<ScanResult, Box<dyn std::error::Error>> {
    let mut directories = HashMap::new();

    // Read the root directory
    let entries = fs::read_dir(root_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Only process directories (one level down)
        if path.is_dir() {
            let dir_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();

            let image_files = find_image_files(&path)?;
            
            if !image_files.is_empty() {
                directories.insert(dir_name, image_files);
            }
        }
    }

    Ok(ScanResult {
        directories,
    })
}

/// Find all image files in a given directory
fn find_image_files(dir_path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut image_files = Vec::new();
    
    let entries = fs::read_dir(dir_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Only process files (not subdirectories)
        if path.is_file() {
            // Skip merged files to avoid including them in new merges
            if let Some(file_name) = path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    if is_merged_file(name_str) {
                        continue;
                    }
                }
            }
            
            if is_image_file(&path) {
                image_files.push(path);
            }
        }
    }

    // Sort files for consistent ordering
    image_files.sort();
    Ok(image_files)
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

/// Check if a file is an image based on its extension
fn is_image_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            let ext_lower = ext_str.to_lowercase();
            matches!(ext_lower.as_str(), 
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp"
            )
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{setup_test_data_for_test, cleanup_test_data_for_test};
    
    #[test]
    fn test_is_image_file() {
        // Test various image extensions
        assert!(is_image_file(Path::new("test.jpg")));
        assert!(is_image_file(Path::new("test.jpeg")));
        assert!(is_image_file(Path::new("test.png")));
        assert!(is_image_file(Path::new("test.gif")));
        assert!(is_image_file(Path::new("test.bmp")));
        assert!(is_image_file(Path::new("test.tiff")));
        assert!(is_image_file(Path::new("test.tif")));
        assert!(is_image_file(Path::new("test.webp")));
        
        // Test case insensitivity
        assert!(is_image_file(Path::new("test.JPG")));
        assert!(is_image_file(Path::new("test.PNG")));
        assert!(is_image_file(Path::new("test.JPEG")));
        
        // Test non-image files
        assert!(!is_image_file(Path::new("test.txt")));
        assert!(!is_image_file(Path::new("test.pdf")));
        assert!(!is_image_file(Path::new("test.doc")));
        assert!(!is_image_file(Path::new("test")));
        assert!(!is_image_file(Path::new("")));
        
        // Test files without extensions
        assert!(!is_image_file(Path::new("no_extension")));
    }
    
    #[test]
    fn test_is_merged_file() {
        // Test basic merged file
        assert!(is_merged_file("merged.png"));
        
        // Test dated merged files
        assert!(is_merged_file("merged-23-12-25.png"));
        assert!(is_merged_file("merged-24-01-15.png"));
        assert!(is_merged_file("merged-99-99-99.png")); // Edge case with high numbers
        
        // Test invalid patterns
        assert!(!is_merged_file("merged.jpg")); // Wrong extension
        assert!(!is_merged_file("merged-2023-12-25.png")); // 4-digit year
        assert!(!is_merged_file("merged-23-1-25.png")); // Single digit month
        assert!(!is_merged_file("merged-23-12-5.png")); // Single digit day
        assert!(!is_merged_file("merged-ab-cd-ef.png")); // Non-numeric
        assert!(!is_merged_file("merged-23.12.25.png")); // Wrong separators
        assert!(!is_merged_file("other.png")); // Regular file
        assert!(!is_merged_file("merged-extra-23-12-25.png")); // Extra parts
        
        // Test empty and edge cases
        assert!(!is_merged_file(""));
        assert!(!is_merged_file("merged"));
        assert!(!is_merged_file("merged-"));
        assert!(!is_merged_file("merged-.png"));
    }
    
    #[test]
    fn test_scan_for_images_with_test_data() {
        // Setup test data
        let test_root = setup_test_data_for_test("scan").expect("Failed to setup test data");
        
        // Scan the test directory
        let result = scan_for_images(Path::new(&test_root)).expect("Failed to scan test data");
        
        // Verify we found the expected directories with images
        assert!(result.directories.contains_key("vertical-images"));
        assert!(result.directories.contains_key("horizontal-images"));
        assert!(result.directories.contains_key("mixed-images"));
        assert!(result.directories.contains_key("single-image"));
        
        // Verify we didn't include empty directories or directories with no images
        assert!(!result.directories.contains_key("empty-dir"));
        assert!(!result.directories.contains_key("no-images"));
        
        // Check vertical images directory (should have 3 images, excluding merged.png)
        let vertical_images = &result.directories["vertical-images"];
        assert_eq!(vertical_images.len(), 3);
        let vertical_names: Vec<String> = vertical_images.iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert!(vertical_names.contains(&"red.png".to_string()));
        assert!(vertical_names.contains(&"green.jpg".to_string()));
        assert!(vertical_names.contains(&"blue.jpeg".to_string()));
        // Should not contain merged.png
        assert!(!vertical_names.contains(&"merged.png".to_string()));
        
        // Check horizontal images directory (should have 3 images, excluding merged-23-12-25.png)
        let horizontal_images = &result.directories["horizontal-images"];
        assert_eq!(horizontal_images.len(), 3);
        let horizontal_names: Vec<String> = horizontal_images.iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert!(horizontal_names.contains(&"yellow.png".to_string()));
        assert!(horizontal_names.contains(&"cyan.bmp".to_string()));
        assert!(horizontal_names.contains(&"magenta.tiff".to_string()));
        // Should not contain merged file
        assert!(!horizontal_names.contains(&"merged-23-12-25.png".to_string()));
        
        // Check mixed images directory
        let mixed_images = &result.directories["mixed-images"];
        assert_eq!(mixed_images.len(), 3);
        let mixed_names: Vec<String> = mixed_images.iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert!(mixed_names.contains(&"white.png".to_string()));
        assert!(mixed_names.contains(&"black.png".to_string()));
        assert!(mixed_names.contains(&"gray.webp".to_string()));
        
        // Check single image directory
        let single_images = &result.directories["single-image"];
        assert_eq!(single_images.len(), 1);
        assert_eq!(single_images[0].file_name().unwrap().to_str().unwrap(), "orange.png");
        
        // Cleanup
        cleanup_test_data_for_test(&test_root).expect("Failed to cleanup test data");
    }
    
    #[test]
    fn test_find_image_files_ordering() {
        let test_root = setup_test_data_for_test("ordering").expect("Failed to setup test data");
        
        // Test that files are returned in sorted order
        let vertical_dir = Path::new(&test_root).join("vertical-images");
        let image_files = find_image_files(&vertical_dir).expect("Failed to find image files");
        
        // Convert to filenames and verify sorting
        let filenames: Vec<String> = image_files.iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        
        // Should be sorted alphabetically
        let mut expected = filenames.clone();
        expected.sort();
        assert_eq!(filenames, expected);
        
        cleanup_test_data_for_test(&test_root).expect("Failed to cleanup test data");
    }
    
    #[test]
    fn test_scan_nonexistent_directory() {
        let result = scan_for_images(Path::new("nonexistent-directory"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_find_image_files_empty_directory() {
        let test_root = setup_test_data_for_test("empty").expect("Failed to setup test data");
        
        let empty_dir = Path::new(&test_root).join("empty-dir");
        let image_files = find_image_files(&empty_dir).expect("Failed to scan empty directory");
        assert!(image_files.is_empty());
        
        cleanup_test_data_for_test(&test_root).expect("Failed to cleanup test data");
    }
    
    #[test]
    fn test_find_image_files_no_images() {
        let test_root = setup_test_data_for_test("scanner_no_images").expect("Failed to setup test data");
        
        let no_images_dir = Path::new(&test_root).join("no-images");
        let image_files = find_image_files(&no_images_dir).expect("Failed to scan no-images directory");
        assert!(image_files.is_empty());
        
        cleanup_test_data_for_test(&test_root).expect("Failed to cleanup test data");
    }
}

 