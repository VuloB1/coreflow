//! Desired state = manual switch OR any listed process running.
//! While the guard is on, a background loop re-applies it if Windows (or an OEM tool) reverts it.
use crate::power::{self, Status};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub manual: bool,
    pub guard: bool,
    pub processes: Vec<String>,
    /// keep running in the tray when the window is closed (otherwise closing it quits)
    #[serde(default = "yes")]
    pub background: bool,
    /// UI language code or "auto" (follow the system)
    #[serde(default = "auto")]
    pub lang: String,
    /// "auto" | "dark" | "light"
    #[serde(default = "auto")]
    pub theme: String,
}

fn yes() -> bool {
    true
}

fn auto() -> String {
    "auto".into()
}

impl Default for Config {
    fn default() -> Self {
        Config { manual: false, guard: true, processes: vec![], background: true, lang: auto(), theme: auto() }
    }
}

static CFG: Mutex<Option<Config>> = Mutex::new(None);

fn path() -> PathBuf {
    PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("unpark-cpu").join("config.json")
}

pub fn get() -> Config {
    let mut g = CFG.lock().unwrap();
    g.get_or_insert_with(|| {
        std::fs::read_to_string(path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    })
    .clone()
}

pub fn set(mut c: Config) -> Result<Config, String> {
    c.processes = c.processes.iter().map(|p| p.trim().to_lowercase()).filter(|p| !p.is_empty()).collect();
    let p = path();
    std::fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&p, serde_json::to_string_pretty(&c).unwrap()).map_err(|e| e.to_string())?;
    *CFG.lock().unwrap() = Some(c.clone());
    Ok(c)
}

/// Manual switch from the UI or tray; remembered so the guard keeps enforcing it.
pub fn set_manual(on: bool) -> Result<Status, String> {
    let st = power::set_unpark(on)?;
    set(Config { manual: on, ..get() })?;
    Ok(st)
}

fn any_running(names: &[String]) -> bool {
    if names.is_empty() {
        return false;
    }
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return false };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut found = false;
        let mut ok = Process32FirstW(snap, &mut e).is_ok();
        while ok && !found {
            let n = String::from_utf16_lossy(&e.szExeFile).trim_end_matches('\0').to_lowercase();
            found = names.contains(&n);
            ok = Process32NextW(snap, &mut e).is_ok();
        }
        let _ = CloseHandle(snap);
        found
    }
}

/// Pure decision: Some(target) when the state must change.
fn needed(unparked: bool, has_backup: bool, desired: bool) -> Option<bool> {
    // never "restore" without a backup: that would be touching a plan we never changed
    (unparked != desired && (desired || has_backup)).then_some(desired)
}

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut n = 0u32;
        loop {
        std::thread::sleep(Duration::from_secs(5));
        n += 1;
        // idle in the tray: hand the heap pages back to Windows once a minute
        if n % 12 == 0 && app.get_webview_window("main").is_none() {
            crate::cpu::trim();
        }
        let c = get();
        if !c.guard {
            continue;
        }
        let Ok(unparked) = power::unparked_now() else { continue };
        let desired = c.manual || any_running(&c.processes);
        if unparked == desired {
            continue;
        }
        if let Some(t) = needed(unparked, desired || power::has_backup(), desired) {
            if let Ok(s) = power::set_unpark(t) {
                let _ = app.emit("status", s);
            }
        }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::needed;
    #[test]
    fn old_config_keeps_running_in_background() {
        let c: super::Config = serde_json::from_str(r#"{"manual":false,"guard":true,"processes":[]}"#).unwrap();
        assert!(c.background);
    }

    #[test]
    fn decisions() {
        assert_eq!(needed(false, false, true), Some(true)); // unpark
        assert_eq!(needed(true, true, false), Some(false)); // restore
        assert_eq!(needed(true, false, false), None); // no backup: leave alone
        assert_eq!(needed(true, true, true), None);
        assert_eq!(needed(false, true, false), None);
    }
}
