pub mod commands;
pub mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = mobile_claw::runtime::config::RuntimeConfig::default();
    let state = AppState::new(config);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::device::discover_devices,
            commands::device::get_all_devices,
            commands::device::get_device_by_id,
            commands::device::connect_device,
            commands::device::disconnect_device,
            commands::device::execute_device_command,
            commands::chat::send_message,
            commands::chat::stream_message,
            commands::chat::get_conversation_history,
            commands::chat::clear_conversation,
            commands::model::get_model_status,
            commands::model::load_model,
            commands::model::unload_model,
            commands::model::get_available_models,
            commands::model::get_hardware_info,
            commands::model::download_model,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::profile::get_user_profile,
            commands::profile::update_user_preferences,
            commands::profile::get_recommendations,
        ])
        .setup(|app| {
            let state = app.state::<AppState>();
            let state_clone = state.inner().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = state_clone.initialize().await {
                    eprintln!("Failed to initialize app state: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
