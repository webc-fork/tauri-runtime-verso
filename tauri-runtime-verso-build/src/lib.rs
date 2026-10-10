//! # Tauri Runtime Verso Build
//!
//! This is a crate to help with getting started with using verso as a webview without building it yourself
//!
//! ## Example
//!
//! To use it, first add it to your build dependency, and in your build script:
//!
//! ```no_run
//! tauri_runtime_verso_build::get_verso_as_external_bin().unwrap();
//! tauri_build::build();
//! ```
//!
//! and in your tauri config file (`tauri.conf.json`), add the following
//!
//! ```json
//! {
//!   "bundle": {
//!     "externalBin": [
//!       "versoview/versoview"
//!     ]
//!   }
//! }
//! ```
//!

use std::{io, path::PathBuf};

pub use versoview_build;

/// Downloads and extracts the pre-built versoview executable
/// to `./versoview/versoview(.exe)` relative to the directory containing your `Cargo.toml` file
pub fn get_verso_as_external_bin() -> io::Result<()> {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "android" || target_os == "ios" {
        return Err(io::Error::other(
            "Versoview doesn't support mobile platforms yet",
        ));
    }

    let target_triple = std::env::var("TARGET").unwrap();

    let project_directory = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_directory = PathBuf::from(project_directory).join("versoview");

    let extension = if cfg!(windows) { ".exe" } else { "" };
    let output_executable = output_directory.join(format!("versoview-{target_triple}{extension}"));
    let output_version = output_directory.join("versoview-version.txt");

    if std::fs::exists(&output_executable)?
        && std::fs::read_to_string(&output_version).unwrap_or_default()
            == versoview_build::VERSO_VERSION
    {
        return Ok(());
    }

    versoview_build::download_and_extract_verso(&output_directory)?;

    let extracted_versoview_path = output_directory.join(format!("versoview{extension}"));
    std::fs::rename(extracted_versoview_path, &output_executable)?;
    std::fs::write(&output_version, versoview_build::VERSO_VERSION)?;

    wait_for_executable_readable(&output_executable);

    println!("cargo:rerun-if-changed={}", output_executable.display());
    println!("cargo:rerun-if-changed={}", output_version.display());

    Ok(())
}

/// The antivirus on Windows can hold a lock on the freshly downloaded executable
/// for a bit, wait for it to become readable again before returning,
/// `tauri_build::build` copies this file right after.
fn wait_for_executable_readable(path: &std::path::Path) {
    for _ in 0..20 {
        match std::fs::File::open(path) {
            Ok(_) => return,
            // ERROR_SHARING_VIOLATION, the file is locked by another process
            Err(error) if error.raw_os_error() == Some(32) => {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(error) => panic!("Failed to read the downloaded versoview executable: {error}"),
        }
    }
    panic!("The downloaded versoview executable stayed locked for more than 10 seconds");
}

/// Removes the sidecar copy of versoview from the cargo target directory if it exists.
///
/// Call this before [`tauri_build::build`](https://docs.rs/tauri-build/latest/tauri_build/fn.build.html),
/// which panics when it can't remove an existing copy itself. That transiently happens on
/// Windows while the antivirus scans a freshly copied versoview executable,
/// so remove it here with retries instead.
pub fn remove_stale_sidecar_copy() {
    let extension = if cfg!(windows) { ".exe" } else { "" };
    let target_triple = std::env::var("TARGET").unwrap();
    let sidecar_copy = std::env::var_os("OUT_DIR")
        .map(PathBuf::from)
        // OUT_DIR is `<target>/<profile>/build/<package>/out`,
        // the sidecar copy lives in `<target>/<profile>`
        .and_then(|out_dir| out_dir.ancestors().nth(3).map(|dir| dir.to_path_buf()))
        .map(|dir| dir.join(format!("versoview-{target_triple}{extension}")));

    let Some(sidecar_copy) = sidecar_copy else {
        return;
    };

    for _ in 0..10 {
        match std::fs::remove_file(&sidecar_copy) {
            // The copy is gone, or never existed, `tauri_build::build` can proceed
            Ok(()) => return,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(500)),
        }
    }
}
