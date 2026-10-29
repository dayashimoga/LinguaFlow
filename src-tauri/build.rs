fn main() {
    // Ensure production frontend assets are compiled into ../dist
    let dist_dir = std::path::Path::new("../dist");
    let dist_index = dist_dir.join("index.html");

    if !dist_index.exists() {
        println!("cargo:warning=Frontend dist not found at ../dist/index.html. Attempting npm run build...");
        let status = if cfg!(target_os = "windows") {
            std::process::Command::new("cmd")
                .args(["/C", "npm run build"])
                .current_dir("..")
                .status()
        } else {
            std::process::Command::new("sh")
                .args(["-c", "npm run build"])
                .current_dir("..")
                .status()
        };

        match status {
            Ok(s) if s.success() => {
                println!("cargo:warning=Successfully compiled frontend assets into ../dist");
            }
            Ok(s) => {
                println!("cargo:warning=npm run build exited with status: {s}");
            }
            Err(e) => {
                println!("cargo:warning=Failed to execute npm run build: {e}");
            }
        }

        // If dist/index.html still does not exist (e.g. headless/isolated Rust checks or npm unavailable),
        // generate a valid minimal placeholder so tauri::generate_context!() does not panic at compile time.
        if !dist_index.exists() {
            println!("cargo:warning=Generating fallback dist placeholder for compile-time context generation");
            let _ = std::fs::create_dir_all(dist_dir);
            let placeholder = "<!doctype html>\n<html lang=\"en\">\n<head><meta charset=\"UTF-8\" /><title>LinguaFlow</title></head>\n<body><div id=\"app\"></div></body>\n</html>\n";
            let _ = std::fs::write(&dist_index, placeholder);
        }
    }

    tauri_build::build();
}
