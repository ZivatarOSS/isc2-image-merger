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

 