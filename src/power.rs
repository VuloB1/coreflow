use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use windows::core::GUID;
use windows::Win32::Foundation::{LocalFree, ERROR_SUCCESS, HLOCAL, WIN32_ERROR};
use windows::Win32::System::Power::*;

const SUB_PROCESSOR: GUID = GUID::from_u128(0x54533251_82be_4824_96c1_47b60b740d00);
// CPMINCORES, CPMINCORES1 (second core class on hybrid CPUs; absent on e.g. Ryzen)
const SETTINGS: [GUID; 2] = [
    GUID::from_u128(0x0cc5b647_c1df_4637_891a_dec35c318583),
    GUID::from_u128(0x0cc5b647_c1df_4637_891a_dec35c318584),
];

#[derive(Serialize, Clone)]
pub struct Status {
    pub scheme: String,
    cores: usize,
    pub unparked: bool,
    pub has_backup: bool,
    /// [ac0, dc0, ac1, dc1]; None when the setting does not exist on this CPU
    values: [Option<u32>; 4],
}

fn chk(e: WIN32_ERROR) -> Result<(), String> {
    if e == ERROR_SUCCESS { Ok(()) } else { Err(format!("Windows error {}", e.0)) }
}

fn active_scheme() -> Result<GUID, String> {
    unsafe {
        let mut p: *mut GUID = std::ptr::null_mut();
        chk(PowerGetActiveScheme(None, &mut p))?;
        let g = *p;
        let _ = LocalFree(Some(HLOCAL(p as _)));
        Ok(g)
    }
}

fn scheme_name(s: &GUID) -> String {
    unsafe {
        let mut size = 0u32;
        let _ = PowerReadFriendlyName(None, Some(s as *const GUID), None, None, None, &mut size);
        if size < 2 { return format!("{s:?}"); }
        let mut buf = vec![0u8; size as usize];
        let r = PowerReadFriendlyName(None, Some(s), None, None, Some(buf.as_mut_ptr()), &mut size);
        if r != ERROR_SUCCESS { return format!("{s:?}"); }
        let w: Vec<u16> = buf.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&c| c != 0).collect();
        String::from_utf16_lossy(&w)
    }
}

fn read(s: &GUID, set: &GUID, ac: bool) -> Option<u32> {
    let mut v = 0u32;
    let (sub, set) = (Some(&SUB_PROCESSOR as *const GUID), Some(set as *const GUID));
    let r = unsafe {
        if ac { PowerReadACValueIndex(None, Some(s as *const GUID), sub, set, &mut v) }
        else { WIN32_ERROR(PowerReadDCValueIndex(None, Some(s as *const GUID), sub, set, &mut v)) }
    };
    (r == ERROR_SUCCESS).then_some(v)
}

fn write(s: &GUID, set: &GUID, ac: bool, v: u32) -> Result<(), String> {
    let (sub, set) = (Some(&SUB_PROCESSOR as *const GUID), Some(set as *const GUID));
    chk(unsafe {
        if ac { PowerWriteACValueIndex(None, s, sub, set, v) }
        else { WIN32_ERROR(PowerWriteDCValueIndex(None, s, sub, set, v)) }
    })
}

fn current(s: &GUID) -> [Option<u32>; 4] {
    [read(s, &SETTINGS[0], true), read(s, &SETTINGS[0], false), read(s, &SETTINGS[1], true), read(s, &SETTINGS[1], false)]
}

fn backup_path() -> PathBuf {
    PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("unpark-cpu").join("backup.json")
}

fn load_backup() -> BTreeMap<String, [Option<u32>; 4]> {
    std::fs::read_to_string(backup_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn is_unparked(v: &[Option<u32>; 4]) -> bool {
    v[0].is_some() && v.iter().flatten().all(|&x| x == 100)
}

/// Cheap checks for the guard loop (no plan name, no backup read unless asked).
pub fn unparked_now() -> Result<bool, String> {
    Ok(is_unparked(&current(&active_scheme()?)))
}

pub fn has_backup() -> bool {
    active_scheme().is_ok_and(|s| load_backup().contains_key(&format!("{s:?}")))
}

pub fn status() -> Result<Status, String> {
    let s = active_scheme()?;
    let values = current(&s);
    Ok(Status {
        scheme: scheme_name(&s),
        cores: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0),
        unparked: is_unparked(&values),
        has_backup: load_backup().contains_key(&format!("{s:?}")),
        values,
    })
}

pub fn set_unpark(on: bool) -> Result<Status, String> {
    let s = active_scheme()?;
    let key = format!("{s:?}");
    let now = current(&s);
    let target = if on {
        if !is_unparked(&now) {
            let mut b = load_backup();
            b.insert(key, now);
            let p = backup_path();
            std::fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(&p, serde_json::to_string_pretty(&b).unwrap()).map_err(|e| e.to_string())?;
        }
        [Some(100); 4]
    } else {
        *load_backup().get(&key).ok_or("No backup for this power plan")?
    };
    for (i, t) in target.iter().enumerate() {
        // skip settings this CPU lacks (now[i] is None)
        if let (Some(v), Some(_)) = (t, now[i]) {
            write(&s, &SETTINGS[i / 2], i % 2 == 0, *v)?;
        }
    }
    chk(unsafe { PowerSetActiveScheme(None, Some(&s as *const GUID)) })?;
    status()
}

#[cfg(test)]
mod tests {
    #[test]
    fn reads_active_scheme() {
        let st = super::status().unwrap();
        assert!(st.values[0].is_some_and(|v| v <= 100));
        assert!(st.cores > 0);
    }
}
