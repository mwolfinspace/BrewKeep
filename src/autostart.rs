// BrewKeep — Lightweight Windows awake/sleep tray utility.
// Copyright (c) 2025 BrewKeep Contributors. MIT License.

use std::env;
use std::process::Command;
use windows::core::{w, PCWSTR, HSTRING};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const RUN_KEY_PATH: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_NAME: PCWSTR = w!("BrewKeep");
const TASK_NAME: &str = "BrewKeep";

fn find_schtasks() -> Option<String> {
    let paths = [
        "C:\\Windows\\System32\\schtasks.exe",
        "C:\\Windows\\SysWOW64\\schtasks.exe",
    ];
    for p in &paths {
        if std::path::Path::new(p).exists() {
            return Some(p.to_string());
        }
    }
    Some("schtasks.exe".to_string())
}

fn ensure_startup_task() {
    let exe_path = match env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    let restore_cmd = format!("\"{}\" --restore", exe_path.display());

    let schtasks = match find_schtasks() {
        Some(p) => p,
        None => return,
    };

    let _ = Command::new(&schtasks)
        .args([
            "/create",
            "/tn",
            TASK_NAME,
            "/tr",
            &restore_cmd,
            "/sc",
            "onstart",
            "/rl",
            "highest",
            "/f",
            "/ru",
            "SYSTEM",
        ])
        .output();
}

fn remove_startup_task() {
    let schtasks = match find_schtasks() {
        Some(p) => p,
        None => return,
    };
    let _ = Command::new(&schtasks)
        .args(["/delete", "/tn", TASK_NAME, "/f"])
        .output();
}

fn is_startup_task_active() -> bool {
    let schtasks = match find_schtasks() {
        Some(p) => p,
        None => return false,
    };
    let output = Command::new(&schtasks)
        .args(["/query", "/tn", TASK_NAME, "/fo", "list"])
        .output();
    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub fn is_autostart_enabled() -> bool {
    if is_startup_task_active() {
        return true;
    }

    unsafe {
        let mut h_key = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY_PATH, 0, KEY_READ, &mut h_key).is_err() {
            return false;
        }

        let mut data_len: u32 = 0;
        let result = RegQueryValueExW(h_key, APP_NAME, None, None, None, Some(&mut data_len));

        let _ = RegCloseKey(h_key);
        result.is_ok()
    }
}

pub fn set_autostart(enable: bool) -> Result<(), String> {
    if enable {
        ensure_startup_task();
    } else {
        remove_startup_task();
    }

    unsafe {
        let mut h_key = HKEY::default();
        let open_result = RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY_PATH, 0, KEY_WRITE, &mut h_key);
        if open_result.is_err() {
            return Err(format!("Failed to open registry key: {:?}", open_result));
        }

        if enable {
            let exe_path = env::current_exe().map_err(|e| e.to_string())?;
            let path_str = format!("\"{}\"", exe_path.display());
            let path_hstr = HSTRING::from(&path_str);
            let bytes = path_hstr.as_wide();
            let byte_slice =
                std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len() * 2);

            let write_result = RegSetValueExW(h_key, APP_NAME, 0, REG_SZ, Some(byte_slice));
            if write_result.is_err() {
                let _ = RegCloseKey(h_key);
                return Err(format!("Failed to write registry value: {:?}", write_result));
            }
        } else {
            let del_result = RegDeleteValueW(h_key, APP_NAME);
            if del_result.is_err() {
                let _ = RegCloseKey(h_key);
                return Err(format!("Failed to delete registry value: {:?}", del_result));
            }
        }

        let _ = RegCloseKey(h_key);
        Ok(())
    }
}
