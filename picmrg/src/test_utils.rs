#[cfg(test)]
use image::{ImageBuffer, RgbImage, DynamicImage};
#[cfg(test)]
use std::path::Path;
#[cfg(test)]
use std::fs;

#[cfg(test)]
pub fn generate_test_image(width: u32, height: u32, color: [u8; 3]) -> DynamicImage {
    let img: RgbImage = ImageBuffer::from_fn(width, height, |_x, _y| {
        image::Rgb(color)
    });
    DynamicImage::ImageRgb8(img)
}

#[cfg(test)]
pub fn save_test_image(image: &DynamicImage, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    image.save(path)?;
    Ok(())
}

#[cfg(test)]
pub fn setup_test_data_for_test(test_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let test_root = format!("test-data-{}", test_name);
    let test_path = Path::new(&test_root);
    
    // Clean up any existing test data first
    if test_path.exists() {
        fs::remove_dir_all(test_path)?;
    }
    
    // Create test directories
    fs::create_dir_all(test_path.join("vertical-images"))?;
    fs::create_dir_all(test_path.join("horizontal-images"))?;
    fs::create_dir_all(test_path.join("mixed-images"))?;
    fs::create_dir_all(test_path.join("single-image"))?;
    fs::create_dir_all(test_path.join("empty-dir"))?;
    fs::create_dir_all(test_path.join("no-images"))?;

    // Generate vertical images (taller than wide)
    let red_vertical = generate_test_image(200, 400, [255, 0, 0]); // Red
    let green_vertical = generate_test_image(150, 350, [0, 255, 0]); // Green
    let blue_vertical = generate_test_image(180, 420, [0, 0, 255]); // Blue
    
    save_test_image(&red_vertical, &test_path.join("vertical-images/red.png"))?;
    save_test_image(&green_vertical, &test_path.join("vertical-images/green.jpg"))?;
    save_test_image(&blue_vertical, &test_path.join("vertical-images/blue.jpeg"))?;

    // Generate horizontal images (wider than tall)
    let yellow_horizontal = generate_test_image(400, 200, [255, 255, 0]); // Yellow
    let cyan_horizontal = generate_test_image(350, 150, [0, 255, 255]); // Cyan
    let magenta_horizontal = generate_test_image(420, 180, [255, 0, 255]); // Magenta
    
    save_test_image(&yellow_horizontal, &test_path.join("horizontal-images/yellow.png"))?;
    save_test_image(&cyan_horizontal, &test_path.join("horizontal-images/cyan.bmp"))?;
    save_test_image(&magenta_horizontal, &test_path.join("horizontal-images/magenta.tiff"))?;

    // Generate mixed orientation images
    let white_vertical = generate_test_image(100, 300, [255, 255, 255]); // White vertical
    let black_horizontal = generate_test_image(300, 100, [0, 0, 0]); // Black horizontal
    let gray_square = generate_test_image(200, 200, [128, 128, 128]); // Gray square
    
    save_test_image(&white_vertical, &test_path.join("mixed-images/white.png"))?;
    save_test_image(&black_horizontal, &test_path.join("mixed-images/black.png"))?;
    save_test_image(&gray_square, &test_path.join("mixed-images/gray.webp"))?;

    // Generate single image
    let orange_single = generate_test_image(250, 250, [255, 165, 0]); // Orange
    save_test_image(&orange_single, &test_path.join("single-image/orange.png"))?;

    // Create non-image files in no-images directory
    fs::write(test_path.join("no-images/readme.txt"), "This is not an image")?;
    fs::write(test_path.join("no-images/data.json"), r#"{"test": true}"#)?;

    // Create some merged files that should be ignored
    let old_merged = generate_test_image(100, 100, [50, 50, 50]);
    save_test_image(&old_merged, &test_path.join("vertical-images/merged.png"))?;
    save_test_image(&old_merged, &test_path.join("horizontal-images/merged-23-12-25.png"))?;

    Ok(test_root)
}

#[cfg(test)]
pub fn cleanup_test_data_for_test(test_root: &str) -> Result<(), Box<dyn std::error::Error>> {
    let test_path = Path::new(test_root);
    if test_path.exists() {
        // Try multiple times in case of temporary file locks
        for _ in 0..3 {
            match fs::remove_dir_all(test_path) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    if !test_path.exists() {
                        return Ok(()); // Already removed
                    }
                    // Try manual cleanup
                    if let Err(_) = recursive_remove_dir(test_path) {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    return Ok(());
                }
            }
        }
        // If we still have issues, try one more manual cleanup
        let _ = recursive_remove_dir(test_path);
    }
    Ok(())
}

#[cfg(test)]
fn recursive_remove_dir(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(());
    }
    
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                recursive_remove_dir(&entry_path)?;
            } else {
                let _ = fs::remove_file(&entry_path); // Ignore errors on individual files
            }
        }
        let _ = fs::remove_dir(path); // Ignore error if already removed
    } else {
        let _ = fs::remove_file(path); // Ignore error if already removed
    }
    
    Ok(())
} 