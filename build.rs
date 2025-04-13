#![expect(
    clippy::expect_used,
    reason = "This is a build script, so want to panic if the command fails"
)]
use std::{fs, path::Path, process::Command};

// build.rs
fn main() {
    let version = Command::new("git")
        .arg("describe")
        .arg("--tags")
        .arg("--always")
        .arg("--dirty")
        .output()
        .expect("Failed to execute command");
    let version_str = String::from_utf8_lossy(&version.stdout);
    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", version_str.trim());

    // rerun if a file changes or if a new commit is made
    let dirs = ["src", "tests"];
    for dir in dirs {
        track_dir(dir);
    }
    // Track changes to the Cargo.toml file
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Track changes to the current branch HEAD
    println!("cargo:rerun-if-changed=.git/HEAD");

    // If HEAD is pointing to a branch (not detached), track the reference
    if let Ok(head) = fs::read_to_string(".git/HEAD") {
        if let Some(ref_path) = head.strip_prefix("ref: ").map(str::trim) {
            println!("cargo:rerun-if-changed=.git/{ref_path}");
        }
    }
}

fn track_dir<P: AsRef<Path>>(dir: P) {
    for entry in fs::read_dir(dir).expect("Failed to read directory") {
        let dir_entry = entry.expect("Failed to read directory entry");
        let path = dir_entry.path();
        if path.is_dir() {
            track_dir(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
