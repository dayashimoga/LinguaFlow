fn main() {
    // Ensure production frontend assets are compiled into ../dist
    let dist_index = std::path::Path::new("../dist/index.html");
    if !dist_index.exists() {
        println!("cargo:warning=Frontend dist not found at ../dist/index.html. Executing npm run build...");
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
    }

    tauri_build::build()
}
