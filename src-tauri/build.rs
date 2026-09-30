fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_app_info",
            "get_preferences",
            "set_theme_preference",
        ]),
    ))
    .expect("Tauri build configuration failed");
}
