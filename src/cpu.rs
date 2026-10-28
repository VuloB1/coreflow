use serde::Serialize;
use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::System::Performance::*;

const PDH_MORE_DATA: u32 = 0x800007D2;

#[derive(Serialize, Default)]
pub struct Core {
    name: String,
    util: f64,
    mhz: f64,
    parked: bool,
}

struct Pdh {
    query: PDH_HQUERY,
    counters: [PDH_HCOUNTER; 3], // utility, frequency, parking status
}
// PDH handles are plain tokens, only ever used under the mutex below
unsafe impl Send for Pdh {}

static PDH: Mutex<Option<Pdh>> = Mutex::new(None);

fn open() -> Result<Pdh, String> {
    unsafe {
        let mut query = PDH_HQUERY::default();
        if PdhOpenQueryW(PCWSTR::null(), 0, &mut query) != 0 {
            return Err("PDH: could not open the query".into());
        }
        let paths = [
            w!("\\Processor Information(*)\\% Processor Utility"),
            w!("\\Processor Information(*)\\Processor Frequency"),
            w!("\\Processor Information(*)\\Parking Status"),
        ];
        let mut counters = [PDH_HCOUNTER::default(); 3];
        for (p, c) in paths.iter().zip(counters.iter_mut()) {
            if PdhAddEnglishCounterW(query, *p, 0, c) != 0 {
                let _ = PdhCloseQuery(query);
                return Err("PDH: counter not available".into());
            }
        }
        let _ = PdhCollectQueryData(query); // baseline: rates need two samples
        Ok(Pdh { query, counters })
    }
}

/// (instance name, value) for every instance of one counter
fn read(c: PDH_HCOUNTER) -> Vec<(String, f64)> {
    unsafe {
        let (mut size, mut count) = (0u32, 0u32);
        if PdhGetFormattedCounterArrayW(c, PDH_FMT_DOUBLE, &mut size, &mut count, None) != PDH_MORE_DATA {
            return vec![];
        }
        let mut buf = vec![0u64; (size as usize).div_ceil(8)]; // u64 keeps the items aligned
        let items = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
        if PdhGetFormattedCounterArrayW(c, PDH_FMT_DOUBLE, &mut size, &mut count, Some(items)) != 0 {
            return vec![];
        }
        (0..count as usize)
            .map(|i| {
                let it = &*items.add(i);
                (it.szName.to_string().unwrap_or_default(), it.FmtValue.Anonymous.doubleValue)
            })
            .collect()
    }
}

/// "0,12" -> (0, 12) so cores sort numerically
fn key(n: &str) -> (u32, u32) {
    let mut p = n.split(',').map(|x| x.parse().unwrap_or(0));
    (p.next().unwrap_or(0), p.next().unwrap_or(0))
}

/// Drop the PDH query (its counters hold memory) once nobody is watching.
pub fn close() {
    if let Some(p) = PDH.lock().ok().and_then(|mut g| g.take()) {
        unsafe { let _ = PdhCloseQuery(p.query); }
    }
}

/// Give unused pages back to Windows: lowers the resident size shown in Task Manager.
pub fn trim() {
    unsafe { let _ = windows::Win32::System::ProcessStatus::EmptyWorkingSet(windows::Win32::System::Threading::GetCurrentProcess()); }
}

pub fn cores() -> Result<Vec<Core>, String> {
    let mut g = PDH.lock().map_err(|e| e.to_string())?;
    if g.is_none() {
        *g = Some(open()?);
    }
    let p = g.as_ref().unwrap();
    unsafe { PdhCollectQueryData(p.query) };
    let mut out: Vec<Core> = read(p.counters[0])
        .into_iter()
        .filter(|(n, _)| !n.contains("_Total"))
        .map(|(name, util)| Core { name, util: util.min(100.0), ..Default::default() })
        .collect();
    for (n, v) in read(p.counters[1]) {
        if let Some(c) = out.iter_mut().find(|c| c.name == n) { c.mhz = v; }
    }
    for (n, v) in read(p.counters[2]) {
        if let Some(c) = out.iter_mut().find(|c| c.name == n) { c.parked = v >= 1.0; }
    }
    out.sort_by_key(|c| key(&c.name));
    Ok(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn lists_cores() {
        super::cores().unwrap(); // baseline sample
        std::thread::sleep(std::time::Duration::from_millis(500));
        let c = super::cores().unwrap();
        assert!(c.len() >= 2, "got {} cores", c.len());
        assert!(c.iter().any(|c| c.mhz > 0.0));
    }
}
