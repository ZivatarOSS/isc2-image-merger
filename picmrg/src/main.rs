use std::env;
use std::path::PathBuf;
use std::io::Write;

mod scanner;
mod merger;

fn main() {
    println!("picmrg v{}: image merger\n", env!("CARGO_PKG_VERSION"));
    let args: Vec<String> = env::args().collect();
    
    // Check for help flag
    if args.len() > 1 && args[1] == "-h" {
        print_usage(&args[0]);
        return;
    }
    
    // Determine root path
    let root_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir().expect("Failed to get current directory")
    };
    
    println!("Root path: {}", root_path.display());
    
    // Scan for images
    match scanner::scan_for_images(&root_path) {
        Ok(scan_result) => {
            
            // Merge images in each directory (in alphabetical order)
            let mut sorted_directories: Vec<_> = scan_result.directories.iter().collect();
            sorted_directories.sort_by_key(|(dir_name, _)| *dir_name);
            
            for (dir_name, image_files) in sorted_directories {
                let dir_path = root_path.join(dir_name);
                
                // Print initial status
                print!("\rMerging images in directory: {} ... ", dir_name);
                std::io::stdout().flush().unwrap();
                
                match merger::merge_images_in_directory(&dir_path, image_files) {
                    Ok(()) => {
                        print!("\r✓ Successfully merged images in {}", dir_name);
                        // Pad with spaces to clear any remaining characters, then newline
                        println!("{}", " ".repeat(20));
                    },
                    Err(e) => {
                        let error_msg = e.to_string();
                        if error_msg.contains("Only one image file") {
                            print!("\r- Skipped {} (only one image)", dir_name);
                            println!("{}", " ".repeat(20));
                        } else {
                            print!("\r✗ Failed to merge images in {}: {}", dir_name, e);
                            println!("{}", " ".repeat(10));
                        }
                    },
                }
            }
            
            if scan_result.directories.is_empty() {
                println!("No directories with images found to merge.");
            } else {
                println!("\nMerging complete!");
            }
        }
        Err(e) => {
            eprintln!("Error scanning for images: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_usage(program_name: &str) {
    println!("Usage: {} [ROOT_PATH]", program_name);
    println!();
    println!("Arguments:");
    println!("  ROOT_PATH    Directory to use as root path (default: current directory)");
    println!("  -h           Show this help message");
    println!();
    println!("Examples:");
    println!("  {}           # Use current directory", program_name);
    println!("  {} /path/to/images  # Use specified directory", program_name);
}
