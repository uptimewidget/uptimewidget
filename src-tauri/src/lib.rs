mod api;
mod commands;
mod i18n;
mod mute;
mod secrets;
mod settings;
mod state;
mod tray;
mod ui;
mod updater;

use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second launch just brings up the settings (or onboarding).
            ui::open_settings(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::check_now,
            commands::validate_key,
            commands::save_key,
            commands::remove_key,
            commands::get_key_info,
            commands::get_settings,
            commands::save_settings,
            commands::set_widget_size,
            commands::mute,
            commands::unmute,
            commands::open_monitor,
            commands::open_dashboard,
            commands::open_api_keys_page,
            commands::open_settings,
            commands::get_language,
            commands::open_repository,
            commands::check_for_update,
            commands::install_update,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle().clone();
            let settings = settings::load_settings(&handle);
            i18n::set_language(i18n::Lang::resolve(settings.language));
            let mute = settings::load_mute(&handle);
            let key = secrets::load_key();
            let has_key = key.is_some();
            app.manage(AppState::new(key, settings.clone(), mute));
            app.manage(updater::Updates::default());

            if let Some(widget) = app.get_webview_window(ui::WIDGET) {
                ui::apply_widget_size(&handle, &settings);
                ui::place_widget(&handle, &widget);
                let events_handle = handle.clone();
                widget.on_window_event(move |event| {
                    if let WindowEvent::Moved(position) = event {
                        let state = events_handle.state::<AppState>();
                        let mut inner = state.lock();
                        inner.widget_position = Some(*position);
                        inner.widget_position_dirty = true;
                    }
                });
            }

            tray::create(&handle)?;
            if !has_key {
                ui::open_onboarding(&handle);
            }
            updater::spawn_checker(handle.clone());
            state::spawn_poller(handle);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| match event {
        // Closing the settings or onboarding window must not quit the tray app.
        RunEvent::ExitRequested { code: None, api, .. } => api.prevent_exit(),
        RunEvent::Exit => {
            let state = app.state::<AppState>();
            let inner = state.lock();
            if let Some(pos) = inner.widget_position {
                settings::save(app, "widgetPosition", &(pos.x, pos.y));
            }
        }
        _ => {}
    });
}
