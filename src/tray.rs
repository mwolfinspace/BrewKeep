// BrewKeep — Lightweight Windows awake/sleep tray utility.
// Copyright (c) 2025 BrewKeep Contributors. MIT License.

use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};

use crate::state::{Mode, State};

const ICON_ON_BYTES: &[u8] = include_bytes!("../icon.ico");
const ICON_OFF_BYTES: &[u8] = include_bytes!("../icon_off.ico");

#[derive(Debug, Clone)]
pub enum UserEvent {
    TrayEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

fn load_icon(bytes: &[u8]) -> Icon {
    let icon_dir =
        ico::IconDir::read(std::io::Cursor::new(bytes)).expect("Failed to read icon");
    let entry = icon_dir
        .entries()
        .into_iter()
        .max_by_key(|e| e.width().max(e.height()))
        .expect("Icon has no entries");
    let image = entry.decode().expect("Failed to decode icon entry");
    Icon::from_rgba(image.rgba_data().to_vec(), image.width(), image.height())
        .expect("Failed to create tray icon")
}

fn icon_for_mode(mode: Mode) -> Icon {
    match mode {
        Mode::Hold => load_icon(ICON_ON_BYTES),
        Mode::ForceSleep30 | Mode::ForceSleep60 | Mode::Paused => load_icon(ICON_OFF_BYTES),
    }
}

pub fn run_event_loop(mut state: State) {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::TrayEvent(event));
    }));

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let icon = icon_for_mode(state.current_mode);

    let menu = Menu::new();
    menu.append(&state.hold_item).unwrap();
    menu.append(&state.sleep30_item).unwrap();
    menu.append(&state.sleep60_item).unwrap();
    menu.append(&state.paused_item).unwrap();
    menu.append(&PredefinedMenuItem::separator()).unwrap();
    menu.append(&state.autostart_item).unwrap();
    menu.append(&PredefinedMenuItem::separator()).unwrap();
    let exit_item = MenuItem::new("Exit", true, None);
    menu.append(&exit_item).unwrap();

    state.update_autostart_state();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(state.current_mode.tooltip())
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon");

    let mut tray_icon = Some(tray_icon);

    let hold_id = state.hold_item.id().clone();
    let sleep30_id = state.sleep30_item.id().clone();
    let sleep60_id = state.sleep60_item.id().clone();
    let paused_id = state.paused_item.id().clone();
    let autostart_id = state.autostart_item.id().clone();
    let exit_id = exit_item.id().clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::TrayEvent(tray_event)) => {
                if let tray_icon::TrayIconEvent::Click {
                    button: tray_icon::MouseButton::Left,
                    button_state: tray_icon::MouseButtonState::Up,
                    ..
                } = tray_event
                {
                    state.toggle_hold_pause();
                    if let Some(ref mut icon) = tray_icon {
                        let _ = icon.set_tooltip(Some(state.current_mode.tooltip()));
                        let _ = icon.set_icon(Some(icon_for_mode(state.current_mode)));
                    }
                }
            }
            Event::UserEvent(UserEvent::MenuEvent(menu_event)) => {
                if menu_event.id == hold_id {
                    state.set_mode(Mode::Hold);
                } else if menu_event.id == sleep30_id {
                    state.set_mode(Mode::ForceSleep30);
                } else if menu_event.id == sleep60_id {
                    state.set_mode(Mode::ForceSleep60);
                } else if menu_event.id == paused_id {
                    state.set_mode(Mode::Paused);
                } else if menu_event.id == autostart_id {
                    let current = crate::autostart::is_autostart_enabled();
                    let _ = crate::autostart::set_autostart(!current);
                    state.update_autostart_state();
                } else if menu_event.id == exit_id {
                    tray_icon.take();
                    *control_flow = ControlFlow::Exit;
                }

                if let Some(ref mut icon) = tray_icon {
                    let _ = icon.set_tooltip(Some(state.current_mode.tooltip()));
                    let _ = icon.set_icon(Some(icon_for_mode(state.current_mode)));
                }
            }
            _ => {}
        }
    });
}
