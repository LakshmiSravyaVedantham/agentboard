use std::process::Command;

fn main() {
    // Only build frontend if the dist directory doesn't exist or source changed
    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir("frontend")
        .status()
        .expect("Failed to build frontend — is Node.js installed?");

    if !status.success() {
        panic!("Frontend build failed");
    }

    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/static");
    println!("cargo:rerun-if-changed=frontend/index.html");
}
