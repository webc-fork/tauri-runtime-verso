fn main() {
    // Named sidecar so the api and helloworld examples don't race
    // on the same copy in the shared cargo target directory
    tauri_runtime_verso_build::get_verso_as_external_bin_named("verso-api").unwrap();
    tauri_runtime_verso_build::remove_stale_sidecar_copy_named("verso-api");
    tauri_build::build();
}
