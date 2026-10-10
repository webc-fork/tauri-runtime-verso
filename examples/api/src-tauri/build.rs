fn main() {
    tauri_runtime_verso_build::get_verso_as_external_bin().unwrap();
    tauri_runtime_verso_build::remove_stale_sidecar_copy();
    tauri_build::build();
}
