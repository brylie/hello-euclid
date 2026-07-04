use clap::Parser;
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "xtask")]
#[command(about = "Build and bundle Hello Euclid plugin")]
enum Args {
    /// Bundle the plugin for distribution
    #[command(name = "bundle")]
    Bundle {
        /// Release or debug build
        #[arg(long)]
        release: bool,
    },
}

fn main() {
    let args = Args::parse();

    match args {
        Args::Bundle { release } => {
            bundle_plugin(release);
        }
    }
}

fn bundle_plugin(release: bool) {
    let profile = if release { "release" } else { "debug" };
    let build_type = if release { "--release" } else { "" };

    println!(
        "{}",
        format!("Building Hello Euclid ({} profile)...", profile).cyan()
    );

    // Build the plugin library
    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--lib");
    if release {
        cmd.arg("--release");
    }

    if !cmd.status().expect("Failed to build plugin").success() {
        eprintln!("{}", "Build failed!".red());
        std::process::exit(1);
    }

    println!("{}", "Build successful!".green());

    let target_dir = PathBuf::from("target").join(profile);
    let bundle_dir = target_dir.join("bundle");

    // Create bundle directories for VST3 and CLAP
    let lib_name = if cfg!(target_os = "macos") {
        "libhello_euclid.dylib"
    } else if cfg!(target_os = "windows") {
        "hello_euclid.dll"
    } else {
        "libhello_euclid.so"
    };

    println!(
        "{}",
        format!("Creating plugin bundle in {}...", bundle_dir.display()).cyan()
    );

    // For now, just copy the compiled library
    // A full implementation would create proper VST3 and CLAP bundles

    println!(
        "{}",
        format!(
            "Plugin library ready at: {}",
            target_dir.join(lib_name).display()
        )
        .green()
    );

    #[cfg(target_os = "macos")]
    println!(
        "{}",
        "Note: Manual bundling on macOS required for VST3 packages".yellow()
    );
}
