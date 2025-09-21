use std::process::Command;

fn main() {
    // Get the current git commit hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Get the current build time
    let build_time = chrono::Utc::now().to_rfc3339();

    // Set environment variables for use in code
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}

