// BrewKeep — Lightweight Windows awake/sleep tray utility.
// Copyright (c) 2025 BrewKeep Contributors. MIT License.

use crate::autostart;
use crate::power::{self, PowerStateSnapshot};
use tray_icon::menu::{CheckMenuItem, MenuItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Hold,
    ForceSleep30,
    ForceSleep60,
    Paused,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Hold => "Holding",
            Mode::ForceSleep30 => "Force Sleep 30s",
            Mode::ForceSleep60 => "Force Sleep 1min",
            Mode::Paused => "Paused",
        }
    }

    pub fn tooltip(self) -> String {
        format!("BrewKeep \u{2014} {}", self.label())
    }
}

pub struct State {
    pub current_mode: Mode,
    #[allow(dead_code)]
    pub power_snapshot: Option<PowerStateSnapshot>,
    pub hold_item: MenuItem,
    pub sleep30_item: MenuItem,
    pub sleep60_item: MenuItem,
    pub paused_item: MenuItem,
    pub autostart_item: CheckMenuItem,
}

pub fn create_items() -> (MenuItem, MenuItem, MenuItem, MenuItem, CheckMenuItem) {
    let hold_item = MenuItem::new("Hold (keep awake)", true, None);
    let sleep30_item = MenuItem::new("Force Sleep: 30s", true, None);
    let sleep60_item = MenuItem::new("Force Sleep: 1min", true, None);
    let paused_item = MenuItem::new("Paused", true, None);
    let autostart_item = CheckMenuItem::new("Auto-start with Windows", true, false, None);
    (
        hold_item,
        sleep30_item,
        sleep60_item,
        paused_item,
        autostart_item,
    )
}

impl State {
    pub fn new(
        hold_item: MenuItem,
        sleep30_item: MenuItem,
        sleep60_item: MenuItem,
        paused_item: MenuItem,
        autostart_item: CheckMenuItem,
    ) -> Self {
        let power_snapshot = power::snapshot_current().ok();
        power::hold_awake();

        Self {
            current_mode: Mode::Hold,
            power_snapshot,
            hold_item,
            sleep30_item,
            sleep60_item,
            paused_item,
            autostart_item,
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if self.current_mode == mode {
            return;
        }

        match self.current_mode {
            Mode::Hold => power::release_awake(),
            Mode::ForceSleep30 | Mode::ForceSleep60 | Mode::Paused => {}
        }

        self.current_mode = mode;

        match mode {
            Mode::Hold => power::hold_awake(),
            Mode::ForceSleep30 => {
                let _ = power::set_monitor_timeout(30);
            }
            Mode::ForceSleep60 => {
                let _ = power::set_monitor_timeout(60);
            }
            Mode::Paused => {}
        }

        self.hold_item.set_enabled(mode != Mode::Hold);
        self.sleep30_item.set_enabled(mode != Mode::ForceSleep30);
        self.sleep60_item.set_enabled(mode != Mode::ForceSleep60);
        self.paused_item.set_enabled(mode != Mode::Paused);
    }

    pub fn toggle_hold_pause(&mut self) {
        let new_mode = match self.current_mode {
            Mode::Hold => Mode::Paused,
            _ => Mode::Hold,
        };
        self.set_mode(new_mode);
    }

    pub fn update_autostart_state(&self) {
        self.autostart_item
            .set_checked(autostart::is_autostart_enabled());
    }
}
