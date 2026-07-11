// BrewKeep — Lightweight Windows awake/sleep tray utility.
// Copyright (c) 2025 BrewKeep Contributors. MIT License.

#![windows_subsystem = "windows"]

mod autostart;
mod power;
mod state;
mod tray;

fn main() {
    let (hold_item, sleep30_item, sleep60_item, paused_item, autostart_item) =
        state::create_items();
    let state = state::State::new(hold_item, sleep30_item, sleep60_item, paused_item, autostart_item);
    tray::run_event_loop(state);
}
