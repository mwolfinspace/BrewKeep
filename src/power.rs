// BrewKeep — Lightweight Windows awake/sleep tray utility.
// Copyright (c) 2025 BrewKeep Contributors. MIT License.

use std::path::PathBuf;
use windows::core::{Error, HRESULT, GUID};
use windows::Win32::System::Power::{
    PowerGetActiveScheme, PowerReadACValueIndex, PowerReadDCValueIndex, PowerSetActiveScheme,
    PowerWriteACValueIndex, PowerWriteDCValueIndex, SetThreadExecutionState, ES_CONTINUOUS,
    ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};

const GUID_VIDEO_SUBGROUP: GUID = GUID::from_u128(0x7516B95F_F776_4464_8C53_06167F40CC99);
const GUID_VIDEO_TIMEOUT: GUID = GUID::from_u128(0x3C0BC021_C8A8_4E07_A973_6B14CBCB2B7E);

pub fn state_file_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(appdata).join("BrewKeep").join("state.dat")
}

pub fn save_state_file(ac_timeout: u32, dc_timeout: u32) {
    let path = state_file_path();
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    let _ = std::fs::write(path, format!("{}|{}", ac_timeout, dc_timeout));
}

pub fn load_and_delete_state_file() -> Option<(u32, u32)> {
    let path = state_file_path();
    let content = std::fs::read_to_string(&path).ok()?;
    let _ = std::fs::remove_file(&path);
    let parts: Vec<&str> = content.split('|').collect();
    if parts.len() == 2 {
        let ac = parts[0].parse::<u32>().ok()?;
        let dc = parts[1].parse::<u32>().ok()?;
        Some((ac, dc))
    } else {
        None
    }
}

pub fn restore_from_state_file() -> bool {
    if let Some((ac_timeout, dc_timeout)) = load_and_delete_state_file() {
        let _ = set_monitor_timeout_values(ac_timeout, dc_timeout);
        return true;
    }
    false
}

fn err(msg: &str) -> Error {
    Error::new(HRESULT(0x80004005u32 as i32), msg)
}

fn check_dword(result: u32, msg: &str) -> Result<(), Error> {
    if result == 0 {
        Ok(())
    } else {
        Err(err(msg))
    }
}

pub struct PowerStateSnapshot {
    #[allow(dead_code)]
    pub active_scheme_guid: *mut GUID,
    pub ac_monitor_timeout: u32,
    pub dc_monitor_timeout: u32,
    restore_on_drop: bool,
}

pub fn snapshot_current() -> Result<PowerStateSnapshot, Error> {
    unsafe {
        let mut scheme_ptr: *mut GUID = std::ptr::null_mut();
        let r = PowerGetActiveScheme(None, &mut scheme_ptr);
        if r.is_err() {
            return Err(err("PowerGetActiveScheme failed"));
        }

        let mut ac_timeout: u32 = 0;
        let mut dc_timeout: u32 = 0;

        check_dword(
            PowerReadACValueIndex(
                None,
                Some(scheme_ptr),
                Some(&GUID_VIDEO_SUBGROUP),
                Some(&GUID_VIDEO_TIMEOUT),
                &mut ac_timeout,
            ),
            "PowerReadACValueIndex failed",
        )?;

        check_dword(
            PowerReadDCValueIndex(
                None,
                Some(scheme_ptr),
                Some(&GUID_VIDEO_SUBGROUP),
                Some(&GUID_VIDEO_TIMEOUT),
                &mut dc_timeout,
            ),
            "PowerReadDCValueIndex failed",
        )?;

        Ok(PowerStateSnapshot {
            active_scheme_guid: scheme_ptr,
            ac_monitor_timeout: ac_timeout,
            dc_monitor_timeout: dc_timeout,
            restore_on_drop: true,
        })
    }
}

pub fn set_monitor_timeout_values(ac_seconds: u32, dc_seconds: u32) -> Result<(), Error> {
    unsafe {
        let mut scheme_ptr: *mut GUID = std::ptr::null_mut();
        let r = PowerGetActiveScheme(None, &mut scheme_ptr);
        if r.is_err() {
            return Err(err("PowerGetActiveScheme failed"));
        }

        check_dword(
            PowerWriteACValueIndex(
                None,
                scheme_ptr,
                Some(&GUID_VIDEO_SUBGROUP),
                Some(&GUID_VIDEO_TIMEOUT),
                ac_seconds,
            ),
            "PowerWriteACValueIndex failed",
        )?;
        check_dword(
            PowerWriteDCValueIndex(
                None,
                scheme_ptr,
                Some(&GUID_VIDEO_SUBGROUP),
                Some(&GUID_VIDEO_TIMEOUT),
                dc_seconds,
            ),
            "PowerWriteDCValueIndex failed",
        )?;

        let r = PowerSetActiveScheme(None, Some(scheme_ptr));
        if r.is_err() {
            return Err(err("PowerSetActiveScheme failed"));
        }
        Ok(())
    }
}

pub fn set_monitor_timeout(seconds: u32) -> Result<(), Error> {
    set_monitor_timeout_values(seconds, seconds)
}

pub fn restore(snapshot: &PowerStateSnapshot) -> Result<(), Error> {
    let result = set_monitor_timeout_values(snapshot.ac_monitor_timeout, snapshot.dc_monitor_timeout);
    let _ = std::fs::remove_file(state_file_path());
    result
}

pub fn hold_awake() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED);
    }
}

pub fn prevent_system_sleep() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);
    }
}

pub fn release_awake() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS);
    }
}

impl Drop for PowerStateSnapshot {
    fn drop(&mut self) {
        if self.restore_on_drop {
            let _ = restore(self);
        }
    }
}
