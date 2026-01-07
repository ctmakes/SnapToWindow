use crate::window_manager::{SnapPosition, WindowManager};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};
use tauri_plugin_updater::UpdaterExt;

const TRAY_ID: &str = "main-tray";

// Track last known accessibility state
static LAST_ACCESSIBILITY_STATE: AtomicBool = AtomicBool::new(false);

// Track update availability
static UPDATE_AVAILABLE: AtomicBool = AtomicBool::new(false);
static UPDATE_VERSION: Mutex<Option<String>> = Mutex::new(None);

#[cfg(target_os = "macos")]
fn check_accessibility() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
fn check_accessibility() -> bool {
    true
}

fn open_accessibility_settings() {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .ok();
    }
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let accessibility_enabled = check_accessibility();
    LAST_ACCESSIBILITY_STATE.store(accessibility_enabled, Ordering::SeqCst);
    let update_available = UPDATE_AVAILABLE.load(Ordering::SeqCst);
    let update_version = UPDATE_VERSION.lock().unwrap().clone();

    // Update item (only shown if update available)
    let update_label = if let Some(v) = &update_version {
        format!("⬆️ Install Update (v{})", v)
    } else {
        "⬆️ Install Update".to_string()
    };
    let install_update = MenuItem::with_id(
        app,
        "install_update",
        &update_label,
        true,
        None::<&str>,
    )?;
    let update_sep = PredefinedMenuItem::separator(app)?;

    // Warning item (only shown if accessibility not enabled)
    let warning = MenuItem::with_id(
        app,
        "accessibility_warning",
        "⚠️ Accessibility Required",
        true,
        None::<&str>,
    )?;
    let warning_sep = PredefinedMenuItem::separator(app)?;

    // Halves
    let left_half = MenuItem::with_id(
        app,
        "left_half",
        "Left Half",
        accessibility_enabled,
        Some("ctrl+alt+left"),
    )?;
    let right_half = MenuItem::with_id(
        app,
        "right_half",
        "Right Half",
        accessibility_enabled,
        Some("ctrl+alt+right"),
    )?;
    let top_half = MenuItem::with_id(
        app,
        "top_half",
        "Top Half",
        accessibility_enabled,
        Some("ctrl+alt+up"),
    )?;
    let bottom_half = MenuItem::with_id(
        app,
        "bottom_half",
        "Bottom Half",
        accessibility_enabled,
        Some("ctrl+alt+down"),
    )?;

    // Quarters
    let top_left = MenuItem::with_id(
        app,
        "top_left",
        "Top Left",
        accessibility_enabled,
        Some("ctrl+alt+u"),
    )?;
    let top_right = MenuItem::with_id(
        app,
        "top_right",
        "Top Right",
        accessibility_enabled,
        Some("ctrl+alt+i"),
    )?;
    let bottom_left = MenuItem::with_id(
        app,
        "bottom_left",
        "Bottom Left",
        accessibility_enabled,
        Some("ctrl+alt+j"),
    )?;
    let bottom_right = MenuItem::with_id(
        app,
        "bottom_right",
        "Bottom Right",
        accessibility_enabled,
        Some("ctrl+alt+k"),
    )?;

    // Thirds
    let left_third = MenuItem::with_id(
        app,
        "left_third",
        "Left Third",
        accessibility_enabled,
        Some("ctrl+alt+d"),
    )?;
    let center_third = MenuItem::with_id(
        app,
        "center_third",
        "Center Third",
        accessibility_enabled,
        Some("ctrl+alt+f"),
    )?;
    let right_third = MenuItem::with_id(
        app,
        "right_third",
        "Right Third",
        accessibility_enabled,
        Some("ctrl+alt+g"),
    )?;
    let left_two_thirds = MenuItem::with_id(
        app,
        "left_two_thirds",
        "Left Two Thirds",
        accessibility_enabled,
        Some("ctrl+alt+e"),
    )?;
    let right_two_thirds = MenuItem::with_id(
        app,
        "right_two_thirds",
        "Right Two Thirds",
        accessibility_enabled,
        Some("ctrl+alt+r"),
    )?;

    // Other actions
    let maximize = MenuItem::with_id(
        app,
        "maximize",
        "Maximize",
        accessibility_enabled,
        Some("ctrl+alt+enter"),
    )?;
    let center = MenuItem::with_id(
        app,
        "center",
        "Center",
        accessibility_enabled,
        Some("ctrl+alt+c"),
    )?;

    // Separators
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let sep4 = PredefinedMenuItem::separator(app)?;

    // Settings, Updates, and Quit
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let check_updates = MenuItem::with_id(app, "check_updates", "Check for Updates...", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit SnapToWindow", true, None::<&str>)?;

    let menu = match (accessibility_enabled, update_available) {
        (true, true) => Menu::with_items(
            app,
            &[
                // Update at top
                &install_update,
                &update_sep,
                // Halves
                &left_half,
                &right_half,
                &top_half,
                &bottom_half,
                &sep1,
                // Quarters
                &top_left,
                &top_right,
                &bottom_left,
                &bottom_right,
                &sep2,
                // Thirds
                &left_third,
                &center_third,
                &right_third,
                &left_two_thirds,
                &right_two_thirds,
                &sep3,
                // Other
                &maximize,
                &center,
                &sep4,
                // App controls
                &settings,
                &quit,
            ],
        )?,
        (true, false) => Menu::with_items(
            app,
            &[
                // Halves
                &left_half,
                &right_half,
                &top_half,
                &bottom_half,
                &sep1,
                // Quarters
                &top_left,
                &top_right,
                &bottom_left,
                &bottom_right,
                &sep2,
                // Thirds
                &left_third,
                &center_third,
                &right_third,
                &left_two_thirds,
                &right_two_thirds,
                &sep3,
                // Other
                &maximize,
                &center,
                &sep4,
                // App controls
                &settings,
                &check_updates,
                &quit,
            ],
        )?,
        (false, true) => Menu::with_items(
            app,
            &[
                // Update at top
                &install_update,
                &update_sep,
                // Warning
                &warning,
                &warning_sep,
                // Halves (disabled)
                &left_half,
                &right_half,
                &top_half,
                &bottom_half,
                &sep1,
                // Quarters (disabled)
                &top_left,
                &top_right,
                &bottom_left,
                &bottom_right,
                &sep2,
                // Thirds (disabled)
                &left_third,
                &center_third,
                &right_third,
                &left_two_thirds,
                &right_two_thirds,
                &sep3,
                // Other (disabled)
                &maximize,
                &center,
                &sep4,
                // App controls
                &settings,
                &quit,
            ],
        )?,
        (false, false) => Menu::with_items(
            app,
            &[
                // Warning at top
                &warning,
                &warning_sep,
                // Halves (disabled)
                &left_half,
                &right_half,
                &top_half,
                &bottom_half,
                &sep1,
                // Quarters (disabled)
                &top_left,
                &top_right,
                &bottom_left,
                &bottom_right,
                &sep2,
                // Thirds (disabled)
                &left_third,
                &center_third,
                &right_third,
                &left_two_thirds,
                &right_two_thirds,
                &sep3,
                // Other (disabled)
                &maximize,
                &center,
                &sep4,
                // App controls
                &settings,
                &check_updates,
                &quit,
            ],
        )?,
    };

    let tooltip = match (accessibility_enabled, update_available) {
        (true, true) => "SnapToWindow - ⬆️ Update Available",
        (true, false) => "SnapToWindow",
        (false, true) => "SnapToWindow - ⬆️ Update | ⚠️ Accessibility Required",
        (false, false) => "SnapToWindow - ⚠️ Accessibility Required",
    };

    let tray_icon =
        Image::from_bytes(include_bytes!("../icons/tray.png")).expect("Failed to load tray icon");

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .icon(tray_icon)
        .icon_as_template(true)
        .menu(&menu)
        .tooltip(tooltip)
        .show_menu_on_left_click(true);

    // Show warning indicator next to icon on macOS when accessibility is disabled or update available
    if !accessibility_enabled || update_available {
        builder = builder.title("!");
    }

    builder
        .on_menu_event(|app, event| {
            let position = match event.id.as_ref() {
                // Accessibility warning
                "accessibility_warning" => {
                    open_accessibility_settings();
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                    None
                }
                // Halves
                "left_half" => Some(SnapPosition::LeftHalf),
                "right_half" => Some(SnapPosition::RightHalf),
                "top_half" => Some(SnapPosition::TopHalf),
                "bottom_half" => Some(SnapPosition::BottomHalf),
                // Quarters
                "top_left" => Some(SnapPosition::TopLeft),
                "top_right" => Some(SnapPosition::TopRight),
                "bottom_left" => Some(SnapPosition::BottomLeft),
                "bottom_right" => Some(SnapPosition::BottomRight),
                // Thirds
                "left_third" => Some(SnapPosition::LeftThird),
                "center_third" => Some(SnapPosition::CenterThird),
                "right_third" => Some(SnapPosition::RightThird),
                "left_two_thirds" => Some(SnapPosition::LeftTwoThirds),
                "right_two_thirds" => Some(SnapPosition::RightTwoThirds),
                // Other
                "maximize" => Some(SnapPosition::Maximize),
                "center" => Some(SnapPosition::Center),
                // Non-snap actions
                "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                    None
                }
                "check_updates" => {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        match check_for_updates(&app_handle).await {
                            Ok(true) => println!("Update available, tray updated"),
                            Ok(false) => println!("No updates available"),
                            Err(e) => eprintln!("Update check failed: {}", e),
                        }
                    });
                    None
                }
                "install_update" => {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = do_install_update(&app_handle).await {
                            eprintln!("Failed to install update: {}", e);
                        }
                    });
                    None
                }
                "quit" => {
                    app.exit(0);
                    None
                }
                _ => None,
            };

            if let Some(pos) = position {
                let manager = WindowManager::new();
                if let Err(e) = manager.snap_to(pos) {
                    eprintln!("Failed to snap window: {}", e);
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Check for updates and update tray if available
async fn check_for_updates(app: &AppHandle) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let updater = app.updater()?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            println!("Update available: {}", version);

            // Store update info
            UPDATE_AVAILABLE.store(true, Ordering::SeqCst);
            *UPDATE_VERSION.lock().unwrap() = Some(version.clone());

            // Rebuild tray on main thread (required for macOS)
            let app_clone = app.clone();
            app.run_on_main_thread(move || {
                if let Some(tray) = app_clone.remove_tray_by_id(TRAY_ID) {
                    drop(tray);
                }
                setup_tray(&app_clone).ok();
            }).ok();

            // Notify frontend
            app.emit("update-available", &version).ok();

            Ok(true)
        }
        Ok(None) => {
            println!("App is up to date");
            UPDATE_AVAILABLE.store(false, Ordering::SeqCst);
            *UPDATE_VERSION.lock().unwrap() = None;
            Ok(false)
        }
        Err(e) => {
            Err(Box::new(e))
        }
    }
}

/// Install the available update
async fn do_install_update(app: &AppHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let updater = app.updater()?;

    if let Some(update) = updater.check().await? {
        println!("Installing update: {}", update.version);

        let mut downloaded = 0;
        update.download_and_install(
            |chunk_length, content_length| {
                downloaded += chunk_length;
                println!("Downloaded {} of {:?}", downloaded, content_length);
            },
            || {
                println!("Download complete, preparing to install...");
            },
        ).await?;

        // Restart the app to apply the update
        app.restart();
    }

    Ok(())
}

/// Public function to check for updates at startup
pub async fn check_for_updates_startup(app: &AppHandle) {
    match check_for_updates(app).await {
        Ok(true) => println!("Update available on startup"),
        Ok(false) => println!("App is up to date"),
        Err(e) => eprintln!("Startup update check failed: {}", e),
    }
}

/// Refresh the tray to update accessibility status (only if changed)
pub fn refresh_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let current = check_accessibility();
    let last = LAST_ACCESSIBILITY_STATE.load(Ordering::SeqCst);

    // Only rebuild if state changed
    if current != last {
        if let Some(tray) = app.remove_tray_by_id(TRAY_ID) {
            drop(tray);
        }
        setup_tray(app)?;
    }

    Ok(())
}
