// BrewKeep — Lightweight Windows awake/sleep tray utility.
// Copyright (c) 2025 BrewKeep Contributors. MIT License.

#![windows_subsystem = "windows"]

mod autostart;
mod power;
mod state;
mod tray;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--restore") {
        // Headless restore mode: restore power plan from state.dat and exit.
        // Used by Task Scheduler at system startup (before user login)
        // to ensure the power plan is restored before Windows applies
        // any stale 30s timeout from a previous Force Sleep session.
        power::restore_from_state_file();
        return;
    }

    let (hold_item, sleep30_item, sleep60_item, paused_item, autostart_item) =
        state::create_items();
    let state = state::State::new(hold_item, sleep30_item, sleep60_item, paused_item, autostart_item);
    tray::run_event_loop(state);
}
