use std::process::Command;
use std::path::Path;

fn main() {
    // Path to web-ui directory
    let web_ui_dir = Path::new("web-ui");
    
    if web_ui_dir.exists() {
        // Tell Cargo to re-run this script if any of these files change
        println!("cargo:rerun-if-changed=web-ui/src");
        println!("cargo:rerun-if-changed=web-ui/public");
        println!("cargo:rerun-if-changed=web-ui/index.html");
        println!("cargo:rerun-if-changed=web-ui/package.json");
        println!("cargo:rerun-if-changed=web-ui/vite.config.ts");
        
        println!("Building web-ui with bun...");
        
        // Check if bun is available
        let bun_check = Command::new("sh")
            .arg("-c")
            .arg("command -v bun")
            .output();
            
        if let Err(_) = bun_check {
            eprintln!("Warning: bun is not available in PATH. Web-UI will not be built.");
            return;
        }
        
        // Run bun install (if necessary)
        let install_status = Command::new("bun")
            .current_dir(web_ui_dir)
            .arg("install")
            .status();
            
        match install_status {
            Ok(status) if !status.success() => {
                eprintln!("Warning: bun install failed. Web-UI may not build correctly.");
            },
            Err(e) => {
                eprintln!("Failed to run bun install: {}", e);
                return;
            },
            _ => {}
        }
        
        // Run bun build
        let build_status = Command::new("bun")
            .current_dir(web_ui_dir)
            .arg("run")
            .arg("build")
            .status();
            
        match build_status {
            Ok(status) if !status.success() => {
                panic!("Failed to build web-ui");
            },
            Err(e) => {
                panic!("Failed to run bun run build: {}", e);
            },
            _ => {
                println!("Web-UI build completed successfully");
            }
        }
    } else {
        eprintln!("Warning: web-ui directory not found. Skipping web-ui build.");
    }
}