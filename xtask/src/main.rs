use clap::Parser;
use colored::Colorize;
use std::fs;
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

    println!(
        "{}",
        format!("Building Hello Euclid ({} profile)...", profile).cyan()
    );

    // Build the plugin library from parent directory
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--lib")
        .arg("--manifest-path")
        .arg("../Cargo.toml");
    if release {
        cmd.arg("--release");
    }

    if !cmd.status().expect("Failed to build plugin").success() {
        eprintln!("{}", "Build failed!".red());
        std::process::exit(1);
    }

    println!("{}", "Build successful!".green());

    let target_dir = PathBuf::from("../target").join(profile);
    let bundle_dir = target_dir.join("bundle");

    // Create bundle directory
    if bundle_dir.exists() {
        fs::remove_dir_all(&bundle_dir).expect("Failed to remove old bundle dir");
    }
    fs::create_dir_all(&bundle_dir).expect("Failed to create bundle dir");

    #[cfg(target_os = "macos")]
    {
        let lib_path = target_dir.join("libhello_euclid.dylib");

        // Create VST3 bundle
        create_vst3_bundle(&lib_path, &bundle_dir).expect("Failed to create VST3 bundle");

        // Create CLAP bundle
        create_clap_bundle(&lib_path, &bundle_dir).expect("Failed to create CLAP bundle");

        println!(
            "{}",
            format!("Plugin bundles ready in: {}", bundle_dir.display()).green()
        );
        println!("  • {}", bundle_dir.join("HelloEuclid.vst3").display());
        println!("  • {}", bundle_dir.join("HelloEuclid.clap").display());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let lib_name = if cfg!(target_os = "windows") {
            "hello_euclid.dll"
        } else {
            "libhello_euclid.so"
        };
        println!(
            "{}",
            format!(
                "Plugin library ready at: {}",
                target_dir.join(lib_name).display()
            )
            .green()
        );
    }
}

#[cfg(target_os = "macos")]
fn create_vst3_bundle(
    lib_path: &Path,
    bundle_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let vst3_bundle = bundle_dir.join("HelloEuclid.vst3");
    let macos_dir = vst3_bundle.join("Contents/MacOS");

    fs::create_dir_all(&macos_dir)?;

    // Copy dylib into bundle
    let plugin_name = "HelloEuclid";
    fs::copy(lib_path, macos_dir.join(plugin_name))?;

    // Create PkgInfo
    fs::write(vst3_bundle.join("Contents/PkgInfo"), "BNDL????")?;

    // Create minimal Info.plist
    let plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>HelloEuclid</string>
  <key>CFBundleIdentifier</key>
  <string>brylie.hello-euclid.vst3</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>CFBundlePackageType</key>
  <string>BNDL</string>
  <key>CFBundleSignature</key>
  <string>????</string>
  <key>CFBundleExecutable</key>
  <string>HelloEuclid</string>
</dict>
</plist>"#;

    fs::write(vst3_bundle.join("Contents/Info.plist"), plist)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn create_clap_bundle(
    lib_path: &Path,
    bundle_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let clap_bundle = bundle_dir.join("HelloEuclid.clap");
    let macos_dir = clap_bundle.join("Contents/MacOS");

    fs::create_dir_all(&macos_dir)?;

    // Copy dylib into bundle
    let plugin_name = "HelloEuclid";
    fs::copy(lib_path, macos_dir.join(plugin_name))?;

    // Create PkgInfo
    fs::write(clap_bundle.join("Contents/PkgInfo"), "BNDL????")?;

    // Create minimal Info.plist
    let plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>HelloEuclid</string>
  <key>CFBundleIdentifier</key>
  <string>brylie.hello-euclid.clap</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>CFBundlePackageType</key>
  <string>BNDL</string>
  <key>CFBundleSignature</key>
  <string>????</string>
  <key>CFBundleExecutable</key>
  <string>HelloEuclid</string>
</dict>
</plist>"#;

    fs::write(clap_bundle.join("Contents/Info.plist"), plist)?;

    Ok(())
}
