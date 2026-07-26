/// Helper to restart the application by spawning a detached process
/// that waits for this process to exit and then opens it again.
pub fn restart_application() {
    println!("🔄 Restarting application...");

    if let Ok(exe_path) = std::env::current_exe() {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            // Check if we are running inside a packaged macOS .app bundle
            let mut app_path: Option<std::path::PathBuf> = None;
            #[cfg(target_os = "macos")]
            {
                if exe_path.to_string_lossy().contains(".app/Contents/MacOS/") {
                    if let Some(contents_dir) = exe_path.parent() {
                        if let Some(contents) = contents_dir.parent() {
                            if let Some(app_bundle) = contents.parent() {
                                if app_bundle.extension().and_then(|s| s.to_str()) == Some("app") {
                                    app_path = Some(app_bundle.to_path_buf());
                                }
                            }
                        }
                    }
                }
            }

            // Build the shell command. Paths are single-quoted to handle spaces safely.
            let cmd = if let Some(path) = app_path {
                let p = path.to_string_lossy().replace("'", "'\\''");
                format!("sleep 0.5 && open -n '{}'", p)
            } else {
                let p = exe_path.to_string_lossy().replace("'", "'\\''");
                format!("sleep 0.5 && '{}'", p)
            };

            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .spawn();
        }

        #[cfg(target_os = "windows")]
        {
            // Use cmd.exe on Windows — 'sh' is not available by default
            let exe_str = exe_path.to_string_lossy().to_string();
            let _ = std::process::Command::new("cmd")
                .args(["/C", "ping", "-n", "2", "127.0.0.1", ">nul", "&&"])
                .arg(&exe_str)
                .spawn();
        }
    } else {
        eprintln!("❌ Failed to determine executable path for restart");
    }

    // Cleanly exit the current process so the new one can start fresh
    std::process::exit(0);
}
