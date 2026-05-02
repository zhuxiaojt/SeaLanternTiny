mod commands;
mod models;
mod services;
mod utils;

// 仅在 debug 构建下导入调试命令模块（发布包中不包含）
#[cfg(debug_assertions)]
use commands::debug as debug_commands;

use commands::config as config_commands;
use commands::downloader as download_commands;
use commands::java as java_commands;
use commands::logging as logging_commands;
use commands::mcs_plugin as mcs_plugin_commands;
use commands::player as player_commands;
use commands::server as server_commands;
use commands::settings as settings_commands;
use commands::system as system_commands;
use commands::update as update_commands;

use crate::services::download_manager::DownloadManager;

use std::sync::Arc;
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
#[cfg(target_os = "macos")]
use window_vibrancy::{
    apply_vibrancy, clear_vibrancy, NSVisualEffectMaterial, NSVisualEffectState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Fix white screen issue on Wayland desktop environments (tested on Arch Linux + KDE Plasma)
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    // 尽早注册全局 panic hook，确保此后所有线程发生的 panic 都能被捕获。
    // hook 触发时会收集系统信息（OS、CPU 温度、内存占用等）、
    // panic 源码位置及错误消息，写入 panic-log/ 目录下的日志文件并输出到 stderr，
    // 最终以退出码 0xFFFF 终止进程。
    services::panic_report::init_panic_hook();

    let download_manager = DownloadManager::new();

    tauri::Builder::default()
        .manage(download_manager)
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
            print!("Received second instance with args: {:?}, cwd: {:?}", args, cwd);
        }))
        .on_tray_icon_event(|app, event| {
            if let TrayIconEvent::Click { button, button_state, .. } = event {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    if let Some(window) = app.get_webview_window("main") {
                        match window.is_visible() {
                            Ok(is_visible) => {
                                if is_visible {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            Err(_) => {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            server_commands::create_server,
            server_commands::import_server,
            server_commands::add_existing_server,
            server_commands::import_modpack,
            server_commands::parse_server_core_type,
            server_commands::scan_startup_candidates,
            server_commands::collect_copy_conflicts,
            server_commands::copy_directory_contents,
            server_commands::start_server,
            server_commands::stop_server,
            server_commands::prepare_force_stop_server,
            server_commands::force_stop_server,
            server_commands::send_command,
            server_commands::get_server_list,
            server_commands::get_server_status,
            server_commands::delete_server,
            server_commands::get_server_logs,
            server_commands::update_server_name,
            server_commands::validate_server_path,
            server_commands::update_server_path,
            java_commands::detect_java,
            java_commands::validate_java_path,
            java_commands::install_java,
            java_commands::cancel_java_install,
            config_commands::read_config,
            config_commands::write_config,
            config_commands::read_server_properties,
            config_commands::write_server_properties,
            config_commands::read_server_properties_source,
            config_commands::write_server_properties_source,
            config_commands::parse_server_properties_source,
            config_commands::preview_server_properties_write,
            config_commands::preview_server_properties_write_from_source,
            system_commands::get_system_info,
            system_commands::get_server_resource_usage,
            system_commands::pick_jar_file,
            system_commands::pick_archive_file,
            system_commands::pick_startup_file,
            system_commands::pick_server_executable,
            system_commands::pick_java_file,
            system_commands::pick_save_file,
            system_commands::pick_folder,
            system_commands::pick_image_file,
            system_commands::open_file,
            system_commands::open_folder,
            system_commands::get_default_run_path,
            system_commands::get_safe_mode_status,
            system_commands::frontend_heartbeat,
            player_commands::get_whitelist,
            player_commands::get_banned_players,
            player_commands::get_ops,
            player_commands::add_to_whitelist,
            player_commands::remove_from_whitelist,
            player_commands::ban_player,
            player_commands::unban_player,
            player_commands::add_op,
            player_commands::remove_op,
            player_commands::kick_player,
            player_commands::export_logs,
            settings_commands::get_settings,
            settings_commands::save_settings,
            settings_commands::save_settings_with_diff,
            settings_commands::update_settings_partial,
            settings_commands::reset_settings,
            settings_commands::export_settings,
            settings_commands::import_settings,
            settings_commands::get_system_fonts,
            settings_commands::get_plugin_commands,
            settings_commands::update_plugin_commands,
            settings_commands::apply_acrylic,
            update_commands::check_update,
            update_commands::open_download_url,
            update_commands::download_update,
            update_commands::install_update,
            update_commands::check_pending_update,
            update_commands::clear_pending_update,
            update_commands::restart_and_install,
            update_commands::download_update_from_debug_url,
            download_commands::download_file,
            download_commands::poll_task,
            download_commands::poll_all_downloads,
            download_commands::get_server_types,
            download_commands::get_versions_by_type,
            download_commands::get_download_info,
            download_commands::cancel_download_task,
            logging_commands::get_logs,
            logging_commands::clear_logs,
            logging_commands::check_developer_mode,
            mcs_plugin_commands::m_get_plugins,
            mcs_plugin_commands::m_get_plugin_config_files,
            mcs_plugin_commands::m_toggle_plugin,
            mcs_plugin_commands::m_delete_plugin,
            mcs_plugin_commands::m_install_plugin,
            #[cfg(debug_assertions)]
            debug_commands::debug_panic
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Enter { .. }) = event {
                let _ = window.emit("tauri://drag", ());
            }
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                let _ = window.emit("tauri://drop", paths);
            }
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Leave) = event {
                let _ = window.emit("tauri://drag-cancelled", ());
            }

            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let settings = services::global::settings_manager().get();

                match settings.close_action.as_str() {
                    "minimize" => {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                    "close" => {
                        if settings.close_servers_on_exit {
                            services::global::server_manager().stop_all_servers();
                        }
                        window.app_handle().exit(0);
                    }
                    _ => {
                        api.prevent_close();
                        let _ = window.emit("close-requested", ());
                    }
                }
            }
        })
        .setup(|app| {
            #[cfg(not(target_os = "macos"))]
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.set_decorations(false) {
                    eprintln!("Failed to disable native window decorations: {}", e);
                }
            }

            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.set_decorations(true) {
                    eprintln!("Failed to enable native macOS window decorations: {}", e);
                }

                if let Err(e) = window.set_title_bar_style(TitleBarStyle::Overlay) {
                    eprintln!("Failed to set macOS title bar style to overlay: {}", e);
                }

                let acrylic_enabled = crate::services::global::settings_manager()
                    .get()
                    .acrylic_enabled;

                let native_effect_result = if acrylic_enabled {
                    apply_vibrancy(
                        &window,
                        NSVisualEffectMaterial::UnderWindowBackground,
                        Some(NSVisualEffectState::Active),
                        None,
                    )
                    .map(|_| ())
                } else {
                    clear_vibrancy(&window).map(|_| ())
                };

                if let Err(e) = native_effect_result {
                    eprintln!("Failed to sync native macOS vibrancy effect: {}", e);
                }
            }

            {
                use serde::Serialize;

                #[derive(Serialize, Clone)]
                struct ServerLogLineEvent {
                    server_id: String,
                    line: String,
                }

                let app_handle = app.handle().clone();
                let _ = services::server_log_pipeline::set_server_log_event_handler(Arc::new(
                    move |server_id, line| {
                        let event = ServerLogLineEvent {
                            server_id: server_id.to_string(),
                            line: line.to_string(),
                        };
                        app_handle
                            .emit("server-log-line", event)
                            .map_err(|e| format!("Failed to emit server log line event: {}", e))
                    },
                ));
            }

            // 前端心跳看门狗：若长时间未收到心跳则自动退出进程
            {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    use std::time::{Duration, SystemTime, UNIX_EPOCH};
                    use tokio::time::sleep;

                    loop {
                        sleep(Duration::from_secs(5)).await;

                        let last = crate::services::global::last_frontend_heartbeat();
                        if last == 0 {
                            continue;
                        }

                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        if now.saturating_sub(last) > 30 {
                            eprintln!(
                                "[Watchdog] frontend heartbeat lost, shutting down Sea Lantern",
                            );

                            let settings = crate::services::global::settings_manager().get();
                            if settings.close_servers_on_exit {
                                crate::services::global::server_manager().stop_all_servers();
                            }

                            app_handle.exit(0);
                            break;
                        }
                    }
                });
            }

            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let safe_mode_item =
                MenuItem::with_id(app, "restart-safe-mode", "以安全模式重启", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &safe_mode_item, &quit_item])?;

            let icon_bytes = include_bytes!("../icons/icon.png");
            let img = image::load_from_memory(icon_bytes)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
                .into_rgba8();
            let (width, height) = img.dimensions();
            let icon = tauri::image::Image::new_owned(img.into_raw(), width, height);

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("Sea Lantern")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "restart-safe-mode" => {
                        let settings = services::global::settings_manager().get();
                        if settings.close_servers_on_exit {
                            services::global::server_manager().stop_all_servers();
                        }

                        let default_name = if cfg!(windows) {
                            "SeaLantern.exe"
                        } else {
                            "SeaLantern"
                        };
                        let app_path = std::env::current_exe()
                            .or_else(|_| {
                                std::env::args().next().map(std::path::PathBuf::from).ok_or(
                                    std::io::Error::new(
                                        std::io::ErrorKind::NotFound,
                                        "Executable path not found",
                                    ),
                                )
                            })
                            .unwrap_or_else(|_| std::path::PathBuf::from(default_name));

                        #[cfg(target_os = "macos")]
                        {
                            if let Some(app_bundle_path) = app_path
                                .ancestors()
                                .find(|p| p.extension().is_some_and(|ext| ext == "app"))
                            {
                                match std::process::Command::new("open")
                                    .arg("-n")
                                    .arg(app_bundle_path)
                                    .arg("--args")
                                    .arg("--safe-mode")
                                    .spawn()
                                {
                                    Ok(_) => app.exit(0),
                                    Err(e) => {
                                        eprintln!(
                                            "Failed to restart in safe mode using open command: {}",
                                            e
                                        );
                                        match std::process::Command::new(&app_path)
                                            .arg("--safe-mode")
                                            .spawn()
                                        {
                                            Ok(_) => app.exit(0),
                                            Err(e) => {
                                                eprintln!("Failed to restart in safe mode: {}", e);
                                                app.exit(1);
                                            }
                                        }
                                    }
                                }
                            } else {
                                match std::process::Command::new(&app_path)
                                    .arg("--safe-mode")
                                    .spawn()
                                {
                                    Ok(_) => app.exit(0),
                                    Err(e) => {
                                        eprintln!("Failed to restart in safe mode: {}", e);
                                        app.exit(1);
                                    }
                                }
                            }
                        }

                        #[cfg(target_os = "linux")]
                        {
                            use std::fs::Permissions;
                            use std::os::unix::fs::PermissionsExt;

                            if let Ok(metadata) = app_path.metadata() {
                                let perms = metadata.permissions();
                                if (perms.mode() & 0o111) == 0 {
                                    if let Ok(()) = std::fs::set_permissions(
                                        &app_path,
                                        Permissions::from_mode(perms.mode() | 0o111),
                                    ) {
                                        eprintln!(
                                            "Added execute permissions to {}",
                                            app_path.display()
                                        );
                                    }
                                }
                            }

                            match std::process::Command::new(&app_path)
                                .arg("--safe-mode")
                                .spawn()
                            {
                                Ok(_) => app.exit(0),
                                Err(e) => {
                                    eprintln!("Failed to restart in safe mode: {}", e);
                                    app.exit(1);
                                }
                            }
                        }

                        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
                        {
                            match std::process::Command::new(&app_path)
                                .arg("--safe-mode")
                                .spawn()
                            {
                                Ok(_) => app.exit(0),
                                Err(e) => {
                                    eprintln!("Failed to restart in safe mode: {}", e);
                                    app.exit(1);
                                }
                            }
                        }
                    }
                    "quit" => {
                        let settings = services::global::settings_manager().get();
                        if settings.close_servers_on_exit {
                            services::global::server_manager().stop_all_servers();
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Sea Lantern");
}
