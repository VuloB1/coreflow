#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod cpu;
mod guard;
mod power;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::*;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};

#[tauri::command]
fn status() -> Result<power::Status, String> {
    power::status()
}

#[tauri::command]
fn set_unpark(on: bool) -> Result<power::Status, String> {
    guard::set_manual(on)
}

#[tauri::command]
fn get_config() -> guard::Config {
    guard::get()
}

#[tauri::command]
fn set_config(cfg: guard::Config) -> Result<guard::Config, String> {
    guard::set(cfg)
}

const RUN_KEY: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
const RUN_VAL: PCWSTR = w!("Coreflow");

#[tauri::command]
fn get_autostart() -> bool {
    unsafe { RegGetValueW(HKEY_CURRENT_USER, RUN_KEY, RUN_VAL, RRF_RT_REG_SZ, None, None, None) == ERROR_SUCCESS }
}

#[tauri::command]
fn set_autostart(on: bool) -> Result<bool, String> {
    let r = unsafe {
        if on {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let cmd: Vec<u16> = format!("\"{}\" --tray", exe.display()).encode_utf16().chain([0]).collect();
            RegSetKeyValueW(HKEY_CURRENT_USER, RUN_KEY, RUN_VAL, REG_SZ.0, Some(cmd.as_ptr() as _), (cmd.len() * 2) as u32)
        } else {
            RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, RUN_VAL)
        }
    };
    if r == ERROR_SUCCESS || (!on && r == ERROR_FILE_NOT_FOUND) { Ok(on) } else { Err(format!("Windows error {}", r.0)) }
}

#[tauri::command]
fn cores() -> Result<Vec<cpu::Core>, String> {
    cpu::cores()
}

/// The window is created on demand and destroyed on close, so the webview costs no RAM while in the tray.
fn show(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Coreflow")
        .inner_size(400.0, 310.0)
        .resizable(false)
        .maximizable(false)
        .decorations(false)
        .center()
        .build();
}

const MENU_SIZE: (f64, f64) = (210.0, 112.0);

/// When the popup was shown (ms since epoch). Right after a tray click the shell often steals focus back;
/// blurs in that grace window re-take focus instead of closing the popup.
static MENU_SHOWN_AT: AtomicU64 = AtomicU64::new(0);
const MENU_GRACE_MS: u64 = 800;
static MENU_LAST_USED: AtomicU64 = AtomicU64::new(0);
const MENU_KEEP_MS: u64 = 20_000;

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// Hide (not destroy): the popup stays warm for a while so the next right-click is instant.
fn close_menu(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("menu") {
        let _ = w.hide();
        MENU_LAST_USED.store(now_ms(), Ordering::SeqCst);
    }
}

/// Build the popup hidden, so a right-click only has to show it. Called when the mouse reaches the tray icon.
/// A reaper destroys it after MENU_KEEP_MS of no use, so idle RAM stays near zero.
fn prewarm_menu(app: &AppHandle) {
    MENU_LAST_USED.store(now_ms(), Ordering::SeqCst);
    if app.get_webview_window("menu").is_some() {
        return;
    }
    MENU_SHOWN_AT.store(now_ms(), Ordering::SeqCst); // blurs fired while the window is built must not close it
    let built = WebviewWindowBuilder::new(app, "menu", WebviewUrl::App("menu.html".into()))
        .inner_size(MENU_SIZE.0, MENU_SIZE.1)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build();
    if built.is_err() {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let Some(w) = app.get_webview_window("menu") else { break };
        if matches!(w.is_visible(), Ok(true)) {
            MENU_LAST_USED.store(now_ms(), Ordering::SeqCst);
        } else if now_ms().saturating_sub(MENU_LAST_USED.load(Ordering::SeqCst)) > MENU_KEEP_MS {
            let _ = w.destroy();
            break;
        }
    });
}

/// Tray right-click: a small styled window above the cursor (native tray menus cannot be themed).
fn popup_menu(app: &AppHandle, pos: PhysicalPosition<f64>) {
    prewarm_menu(app); // no-op when already warm
    let Some(w) = app.get_webview_window("menu") else { return };
    let sf = w.scale_factor().unwrap_or(1.0);
    let (ww, wh, gap) = (MENU_SIZE.0 * sf, MENU_SIZE.1 * sf, 8.0 * sf);
    let (mx, my, mw, mh) = app
        .monitor_from_point(pos.x, pos.y)
        .ok()
        .flatten()
        .map(|m| (m.position().x as f64, m.position().y as f64, m.size().width as f64, m.size().height as f64))
        .unwrap_or((0.0, 0.0, 1e9, 1e9));
    let x = (pos.x - ww / 2.0).clamp(mx, (mx + mw - ww).max(mx));
    let y = if pos.y - wh - gap >= my { pos.y - wh - gap } else { pos.y + gap };
    let _ = w.set_position(PhysicalPosition::new(x, y.min(my + mh - wh)));
    let _ = app.emit_to("menu", "refresh", ()); // a warm popup may hold a stale state
    MENU_SHOWN_AT.store(now_ms(), Ordering::SeqCst); // restart the grace window at the real show
    let _ = w.show();
    let _ = w.set_focus();
    // if focus never sticks, no blur event will ever fire: hide it once the grace window ends
    std::thread::sleep(std::time::Duration::from_millis(MENU_GRACE_MS + 150));
    if matches!(w.is_focused(), Ok(false)) {
        let _ = w.hide();
    }
}

#[tauri::command]
async fn open_main(app: AppHandle) {
    close_menu(&app);
    show(&app);
}

#[tauri::command]
fn tray_toggle(app: AppHandle) -> Result<(), String> {
    let s = guard::set_manual(!power::status()?.unparked)?;
    let _ = app.emit("status", s);
    close_menu(&app);
    Ok(())
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// `coreflow --on|--off|--status` runs without opening the window.
fn cli() -> Option<i32> {
    use std::io::Write;
    let arg = std::env::args().nth(1)?;
    if !matches!(arg.as_str(), "--on" | "--off" | "--status") {
        return None;
    }
    unsafe { let _ = windows::Win32::System::Console::AttachConsole(u32::MAX); }
    let mut out = std::fs::OpenOptions::new().write(true).open("CONOUT$").ok();
    let r = match arg.as_str() {
        "--on" => guard::set_manual(true),
        "--off" => guard::set_manual(false),
        _ => power::status(),
    };
    let msg = match &r {
        Ok(s) => format!("{}: {}", s.scheme, if s.unparked { "unparked" } else { "Windows parking" }),
        Err(e) => format!("error: {e}"),
    };
    // redirected/piped stdout works as is; otherwise fall back to the parent console
    if writeln!(std::io::stdout(), "{msg}").is_err() {
        if let Some(o) = out.as_mut() { let _ = writeln!(o, "{msg}"); }
    }
    Some(r.is_err() as i32)
}

fn main() {
    if let Some(code) = cli() {
        std::process::exit(code);
    }
    tauri::Builder::default()
        .setup(|app| {
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Coreflow")
                // building a window on the event-loop thread deadlocks on Windows: use worker threads
                .on_tray_icon_event(|t, e| match e {
                    TrayIconEvent::Enter { .. } | TrayIconEvent::Move { .. } => {
                        // hovering the icon: start warming the popup (throttled, Move fires constantly)
                        if now_ms().saturating_sub(MENU_LAST_USED.load(Ordering::SeqCst)) > 1000 {
                            MENU_LAST_USED.store(now_ms(), Ordering::SeqCst);
                            let app = t.app_handle().clone();
                            std::thread::spawn(move || prewarm_menu(&app));
                        }
                    }
                    TrayIconEvent::Click { button, button_state: MouseButtonState::Up, position, .. } => {
                        let app = t.app_handle().clone();
                        match button {
                            MouseButton::Left => { std::thread::spawn(move || show(&app)); }
                            MouseButton::Right => { std::thread::spawn(move || popup_menu(&app, position)); }
                            _ => {}
                        }
                    }
                    _ => {}
                })
                .build(app)?;
            if !std::env::args().any(|a| a == "--tray") {
                show(app.handle());
            }
            guard::spawn(app.handle().clone());
            cpu::trim();
            Ok(())
        })
        .on_window_event(|w, e| {
            if w.label() == "main" && matches!(e, WindowEvent::Destroyed) {
                cpu::close();
                if guard::get().background {
                    cpu::trim();
                } else {
                    w.app_handle().exit(0);
                }
            }
            if let (true, WindowEvent::Focused(false)) = (w.label() == "menu", e) {
                if now_ms().saturating_sub(MENU_SHOWN_AT.load(Ordering::SeqCst)) < MENU_GRACE_MS {
                    let w = w.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(60));
                        let _ = w.set_focus();
                    });
                } else {
                    let _ = w.hide();
                    MENU_LAST_USED.store(now_ms(), Ordering::SeqCst);
                }
            }
            if w.label() == "menu" && matches!(e, WindowEvent::Destroyed) {
                cpu::trim();
            }
        })
        .invoke_handler(tauri::generate_handler![status, set_unpark, cores, get_config, set_config, get_autostart, set_autostart, open_main, tray_toggle, quit_app])
        .build(tauri::generate_context!())
        .expect("error while running Coreflow")
        .run(|_, e| {
            // closing the last window must not quit: the app lives in the tray (code is Some on app.exit)
            if let RunEvent::ExitRequested { api, code: None, .. } = e {
                api.prevent_exit();
            }
        });
}
