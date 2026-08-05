/// Starts the native shell around the ClojureScript workspace.
/// Запускає нативну оболонку навколо ClojureScript-середовища.
/// Startet die native Hülle um den ClojureScript-Arbeitsbereich.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running my-idea");
}
