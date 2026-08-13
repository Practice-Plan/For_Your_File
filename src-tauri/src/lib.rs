//! LNK File Management Center - Tauri Backend
//!
//! Global hotkey system for window activation and management
//! Protocol handling for filemgmt:// deep links
//! CLI argument handling for command-line operations
//! Expiration reminder system for temporary files

mod app_scanner;
mod cli;
mod commands;
mod db;
mod expiration;
mod hotkey;
mod lnk;
mod models;
mod notifications;
mod ppc_linker;
mod protocol;

// Re-export types needed by integration tests
pub use hotkey::HotkeyConfig;
pub use models::Entry;

use cli::CliArgs;
use commands::HotkeyState;
use hotkey::HotkeyManager;
use protocol::{parse_deep_link, ProtocolAction};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Listener, Manager,
};
use tauri_plugin_deep_link::DeepLinkExt;

/// Tray menu labels localized to the system language.
struct TrayLabels {
    show: String,
    quit: String,
}

/// Detect the system language and return localized tray menu labels.
///
/// Detection order:
/// 1. Windows preferred UI language (via `GetUserDefaultLocaleName` / `LCIDToLocaleName`)
/// 2. `LANG` environment variable (Unix fallback)
/// 3. English (default)
///
/// Supported languages: en, zh, fr, ru, ar. Falls back to English if unsupported.
fn get_tray_labels() -> TrayLabels {
    let lang = detect_system_language();
    log::info!("Detected system language for tray: {}", lang);

    match lang.as_str() {
        "zh" => TrayLabels {
            show: "显示/隐藏".to_string(),
            quit: "退出".to_string(),
        },
        "fr" => TrayLabels {
            show: "Afficher/Masquer".to_string(),
            quit: "Quitter".to_string(),
        },
        "ru" => TrayLabels {
            show: "Показать/Скрыть".to_string(),
            quit: "Выйти".to_string(),
        },
        "ar" => TrayLabels {
            show: "إظهار/إخفاء".to_string(),
            quit: "خروج".to_string(),
        },
        _ => TrayLabels {
            show: "Show/Hide".to_string(),
            quit: "Quit".to_string(),
        },
    }
}

/// Detect the system language as a 2-letter code (e.g., "en", "zh").
/// Returns "en" as a fallback if detection fails or the language is unsupported.
///
/// Uses the `sys-locale` crate which calls the appropriate OS API:
/// - Windows: GetUserDefaultLocaleName
/// - macOS: NSLocale.preferredLanguages
/// - Linux: LANG environment variable / locale settings
fn detect_system_language() -> String {
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    let lower = locale.to_lowercase();
    let primary = lower.split('-').next().unwrap_or("en");
    match primary {
        "zh" => "zh".to_string(),
        "fr" => "fr".to_string(),
        "ru" => "ru".to_string(),
        "ar" => "ar".to_string(),
        "en" => "en".to_string(),
        _ => "en".to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Parse CLI arguments first
    let cli_args = CliArgs::parse();

    // Handle --version flag
    if cli_args.version {
        println!("LNK File Management Center v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Handle --help flag
    if cli_args.help {
        println!("{}", CliArgs::help_text());
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            match event {
                // Intercept close button: hide to tray instead of exiting
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    let _ = window.hide();
                    api.prevent_close();
                }
                // Log focus changes for debugging minimize/restore issues
                tauri::WindowEvent::Focused(false) => {
                    log::debug!("Window lost focus");
                }
                _ => {}
            }
        })
        .setup(move |app| {
            // Initialize logging
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize database (creates tables, indexes, triggers if missing)
            if let Err(e) = db::init_database(app.handle()) {
                log::error!("Failed to initialize database: {}", e);
            }

            // Initialize system tray with context menu.
            // Menu item labels are localized at build time by detecting the system language.
            // Tauri's MenuItem text is static once built, so we resolve the language here.
            // The frontend (i18n) separately detects language via navigator/localStorage.
            let tray_labels = get_tray_labels();
            let show_item = MenuItem::with_id(app, "show", &tray_labels.show, true, None::<&str>)
                .map_err(|e| {
                log::error!("Failed to create tray menu item 'show': {}", e);
                e
            })?;
            let quit_item = MenuItem::with_id(app, "quit", &tray_labels.quit, true, None::<&str>)
                .map_err(|e| {
                log::error!("Failed to create tray menu item 'quit': {}", e);
                e
            })?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let tray_app_handle = app.handle().clone();
            let default_icon = app.default_window_icon().cloned().ok_or_else(|| {
                log::error!("Failed to get default window icon for tray");
                "Default window icon not found".to_string()
            })?;
            let _tray = TrayIconBuilder::new()
                .icon(default_icon)
                .tooltip("LNK File Management Center")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(move |tray_app, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        // Left-click toggles window visibility
                        let app_handle = tray_app.app_handle();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if window.is_minimized().unwrap_or(false) {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                            } else if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(&tray_app_handle)?;

            // Handle tray menu item clicks
            let tray_menu_app_handle = app.handle().clone();
            app.on_menu_event(move |_app_handle, event| match event.id().as_ref() {
                "show" => {
                    if let Some(window) = tray_menu_app_handle.get_webview_window("main") {
                        if window.is_minimized().unwrap_or(false) {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        } else if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                "quit" => {
                    log::info!("Quit requested from tray menu");
                    tray_menu_app_handle.exit(0);
                }
                _ => {}
            });

            // Initialize hotkey manager
            let mut hotkey_manager = HotkeyManager::new();

            // Set app handle first (needed by save_config later)
            hotkey_manager.set_app_handle(app.handle().clone());

            // Load saved configuration (or use default if no config file exists)
            let config = hotkey_manager.load_config(app.handle()).unwrap_or_default();

            // Register the hotkey using the loaded config (register() also
            // updates the internal config state with these modifiers/key)
            if let Err(e) = hotkey_manager.register(&config.modifiers, &config.key) {
                log::warn!("Failed to register hotkey: {}", e);
            }

            // Start listening for hotkey events
            if let Err(e) = hotkey_manager.start_listener(app.handle().clone()) {
                log::error!("Failed to start hotkey listener: {}", e);
            }

            // Store hotkey manager in app state
            app.manage(HotkeyState(Mutex::new(hotkey_manager)));

            // Store PPC linker state
            app.manage(ppc_linker::PpcState::new());

            // Handle CLI arguments
            handle_cli_args(app.handle(), &cli_args);

            // Listen for hotkey events from the backend
            // NOTE: This is the ONLY place that toggles window visibility on hotkey-pressed.
            // The frontend (useGlobalHotkey.ts) must NOT also toggle the window,
            // otherwise the double-toggle cancels out and the window appears unchanged.
            let app_handle = app.handle().clone();
            app.listen("hotkey-pressed", move |_event| {
                log::info!("Hotkey pressed event received");

                if let Some(window) = app_handle.get_webview_window("main") {
                    // Check minimized first: a minimized window returns is_visible() == true,
                    // so we must handle it separately to restore instead of hiding.
                    if window.is_minimized().unwrap_or(false) {
                        // Window is minimized - restore and focus
                        log::info!("Window is minimized, restoring");
                        if let Err(e) = window.unminimize() {
                            log::error!("Failed to unminimize window: {}", e);
                        }
                        if let Err(e) = window.show() {
                            log::error!("Failed to show window: {}", e);
                        }
                        if let Err(e) = window.set_focus() {
                            log::error!("Failed to focus window: {}", e);
                        }
                    } else if window.is_visible().unwrap_or(false) {
                        // Window is visible (not minimized) - hide it
                        log::info!("Window is visible, hiding");
                        if let Err(e) = window.hide() {
                            log::error!("Failed to hide window: {}", e);
                        }
                    } else {
                        // Window is hidden - show and focus it
                        log::info!("Window is hidden, showing");
                        if let Err(e) = window.show() {
                            log::error!("Failed to show window: {}", e);
                        }
                        if let Err(e) = window.set_focus() {
                            log::error!("Failed to focus window: {}", e);
                        }
                    }
                }
            });

            // Handle deep link events
            let app_handle_deep_link = app.handle().clone();
            app.listen("deep-link://filemgmt", move |event| {
                let url = event.payload().to_string();
                log::info!("Deep link received: {}", url);

                match parse_deep_link(&url) {
                    Ok(request) => {
                        log::info!(
                            "Parsed deep link action: {:?}, path: {:?}, id: {:?}, query: {:?}",
                            request.action,
                            request.path,
                            request.id,
                            request.query
                        );

                        // Show window for most actions
                        if request.action != ProtocolAction::Settings {
                            if let Some(window) = app_handle_deep_link.get_webview_window("main") {
                                if let Err(e) = window.show() {
                                    log::error!("Failed to show window: {}", e);
                                }
                                if let Err(e) = window.set_focus() {
                                    log::error!("Failed to focus window: {}", e);
                                }
                            }
                        }

                        // Emit event to frontend
                        if let Err(e) = app_handle_deep_link.emit("protocol-request", &request) {
                            log::error!("Failed to emit protocol request: {}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to parse deep link '{}': {}", url, e);
                    }
                }
            });

            // Register deep link handler
            let app_handle_register = app.handle().clone();
            if let Err(e) = app_handle_register.deep_link().register("filemgmt") {
                log::error!("Failed to register deep link handler: {}", e);
            }

            log::info!("LNK File Management Center initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_version,
            commands::parse_lnk_file,
            commands::list_installed_apps,
            commands::get_app_icon,
            commands::register_global_hotkey,
            commands::unregister_global_hotkey,
            commands::update_global_hotkey,
            commands::check_hotkey_conflict,
            commands::get_hotkey_config,
            commands::test_hotkey,
            commands::get_suggested_hotkeys,
            commands::parse_protocol_url,
            commands::handle_protocol_request,
            commands::get_cli_args,
            commands::show_window,
            commands::hide_window,
            commands::minimize_to_tray,
            commands::register_shell_extension,
            commands::unregister_shell_extension,
            commands::is_shell_extension_registered,
            commands::check_expired_entries,
            commands::get_expiring_soon,
            commands::set_expiration,
            commands::remove_expiration,
            commands::extend_expiration,
            commands::get_expiration_status,
            commands::get_expiration_counts,
            commands::delete_expired_entries,
            commands::get_expiration_config,
            commands::update_expiration_config,
            commands::show_expiration_notification,
            commands::create_group,
            commands::list_groups,
            commands::get_group,
            commands::update_group,
            commands::delete_group,
            commands::add_entry_to_group,
            commands::remove_entry_from_group,
            commands::get_group_entries,
            commands::get_entry_groups,
            commands::export_group,
            commands::import_group,
            commands::batch_add_to_group,
            commands::batch_remove_from_group,
            commands::get_entry,
            commands::create_entry,
            commands::update_entry,
            commands::delete_entry,
            commands::get_all_entries,
            commands::search_entries,
            commands::open_lnk_file,
            commands::open_entry,
            commands::open_working_directory,
            commands::open_url,
            commands::batch_create_entries,
            commands::rebuild_fts_index,
            ppc_linker::ppc_connect_auto,
            ppc_linker::ppc_status,
            ppc_linker::ppc_send_command,
            ppc_linker::ppc_disconnect,
            ppc_linker::ppc_error_codes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Handle CLI arguments passed at startup
fn handle_cli_args(app_handle: &tauri::AppHandle, args: &CliArgs) {
    // Handle --minimized flag
    if args.minimized {
        log::info!("Starting minimized to tray");
        if let Some(window) = app_handle.get_webview_window("main") {
            if let Err(e) = window.hide() {
                log::error!("Failed to hide window for minimized start: {}", e);
            }
        }
    }

    // Handle --add flag
    if let Some(path) = &args.add {
        log::info!("CLI: Adding entry from path: {}", path);
        if let Err(e) = app_handle.emit(
            "protocol-request",
            protocol::ProtocolRequest {
                action: ProtocolAction::Add,
                path: Some(path.clone()),
                id: None,
                query: None,
            },
        ) {
            log::error!("Failed to emit add request: {}", e);
        }
    }

    // Handle --open flag
    if let Some(id) = &args.open {
        log::info!("CLI: Opening entry with ID: {}", id);
        if let Err(e) = app_handle.emit(
            "protocol-request",
            protocol::ProtocolRequest {
                action: ProtocolAction::Open,
                path: None,
                id: Some(id.clone()),
                query: None,
            },
        ) {
            log::error!("Failed to emit open request: {}", e);
        }
    }

    // Handle --search flag
    if let Some(query) = &args.search {
        log::info!("CLI: Searching for: {}", query);
        if let Err(e) = app_handle.emit(
            "protocol-request",
            protocol::ProtocolRequest {
                action: ProtocolAction::Search,
                path: None,
                id: None,
                query: Some(query.clone()),
            },
        ) {
            log::error!("Failed to emit search request: {}", e);
        }
    }

    // Handle deep link URL passed as CLI argument
    if let Some(url) = &args.deep_link {
        log::info!("CLI: Processing deep link URL: {}", url);
        match parse_deep_link(url) {
            Ok(request) => {
                if let Err(e) = app_handle.emit("protocol-request", &request) {
                    log::error!("Failed to emit protocol request: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to parse deep link from CLI: {}", e);
            }
        }
    }

    // Handle file paths passed as positional arguments
    for file in &args.files {
        log::info!("CLI: Processing file: {}", file);
        if let Err(e) = app_handle.emit(
            "protocol-request",
            protocol::ProtocolRequest {
                action: ProtocolAction::Add,
                path: Some(file.clone()),
                id: None,
                query: None,
            },
        ) {
            log::error!("Failed to emit file add request: {}", e);
        }
    }
}
