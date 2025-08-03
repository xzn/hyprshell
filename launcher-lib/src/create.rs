use crate::global::{LauncherConfig, LauncherData};
use crate::plugins::get_static_options_chars;
use async_channel::Sender;
use core_lib::config::{Launcher, Modifier};
use core_lib::transfer::{CloseOverviewConfig, Direction, SwitchOverviewConfig, TransferType};
use core_lib::{LAUNCHER_NAMESPACE, WarnWithDetails};
use gtk::gdk::Key;
use gtk::glib::{ControlFlow, Propagation};
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Entry, EventControllerKey, ListBox, PropagationPhase,
    SelectionMode,
};
use gtk::{Orientation, glib};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{Level, debug, span};

pub fn create_windows_overview_launcher_window(
    app: &Application,
    launcher: &Launcher,
    open_key: Box<str>,
    open_modifier: Modifier,
    data_dir: &Path,
    event_sender: Sender<TransferType>,
    grave: bool,
) -> anyhow::Result<LauncherData> {
    let _span = span!(Level::TRACE, "create_windows_overview_launcher_window").entered();

    let main_vbox = ListBox::builder()
        .css_classes(["launcher"])
        .width_request(launcher.width as i32)
        .selection_mode(SelectionMode::None)
        .build();

    let entry = Entry::builder().css_classes(["launcher-input"]).build();
    let event_sender_2 = event_sender.clone();
    entry.connect_changed(move |e| {
        launcher_entry_text_change(e.text().to_string(), event_sender_2.clone());
    });
    main_vbox.append(&entry);

    let results = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["launcher-results"])
        .spacing(3)
        .build();
    main_vbox.append(&results);

    let plugin_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["launcher-plugins"])
        .spacing(7)
        .build();
    main_vbox.append(&plugin_box);

    let window = ApplicationWindow::builder()
        .css_classes(["window"])
        .application(app)
        .child(&main_vbox)
        .default_height(10)
        .default_width(10)
        .build();
    window.init_layer_shell();
    window.set_namespace(Some(LAUNCHER_NAMESPACE));
    window.set_layer(Layer::Top);
    window.set_anchor(Edge::Top, true);
    window.set_margin(Edge::Top, 17);
    window.set_exclusive_zone(-1);
    window.present();
    window.set_visible(false);

    let event_controller = EventControllerKey::new();
    let plugin_keys = get_static_options_chars(&launcher.plugins);
    let event_sender_3 = event_sender.clone();
    let modifiers = Arc::new(Mutex::new(0u16));
    let modifiers_2 = modifiers.clone();
    let entry_2 = entry.clone();
    let results_2 = results.clone();
    let launch_modifier = launcher.launch_modifier;
    event_controller.connect_key_pressed(move |_, key, _, _| {
        handle_key(
            &entry_2,
            key,
            &plugin_keys,
            open_modifier,
            launch_modifier,
            results_2.clone(),
            modifiers_2.clone(),
            event_sender_3.clone(),
            grave,
        )
    });
    let event_sender_4 = event_sender.clone();
    event_controller.connect_key_released(move |_, key, _, _| {
        handle_release(
            key,
            open_key.clone(),
            open_modifier,
            modifiers.clone(),
            event_sender_4.clone(),
        );
    });
    event_controller.set_propagation_phase(PropagationPhase::Capture);
    entry.add_controller(event_controller);

    let entry_2 = entry.clone();
    let window_2 = window.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(400), move || {
        if window_2.is_visible() {
            entry_2.grab_focus_without_selecting(); // ensure that the entry is always focused
        }
        ControlFlow::Continue
    });

    debug!("Created launcher window ({})", window.id());

    launcher_entry_text_change("".to_string(), event_sender);

    Ok(LauncherData {
        config: LauncherConfig {
            default_terminal: launcher.default_terminal.clone(),
            max_items: launcher.max_items,
            launch_modifier: launcher.launch_modifier,
            show_when_empty: launcher.show_when_empty,
            width: launcher.width,
            data_dir: PathBuf::from(data_dir).into_boxed_path(),
            plugins: launcher.plugins.clone(),
        },
        window,
        entry,
        results,
        plugin_box,
        sorted_matches: vec![],
        static_matches: HashMap::new(),
    })
}

fn launcher_entry_text_change(text: String, event_sender: Sender<TransferType>) {
    event_sender
        .send_blocking(TransferType::Type(text))
        .warn("unable to send");
}

fn handle_release(
    key: Key,
    open_key: Box<str>,
    open_modifier: Modifier,
    mods: Arc<Mutex<u16>>,
    event_sender: Sender<TransferType>,
) {
    let mut mods = mods.lock().unwrap();
    match key {
        Key::Shift_L | Key::Shift_R if open_modifier != Modifier::Shift => *mods &= !0b0001,
        Key::Control_L | Key::Control_R if open_modifier != Modifier::Ctrl => *mods &= !0b0010,
        Key::Alt_L | Key::Alt_R if open_modifier != Modifier::Alt => *mods &= !0b0100,
        Key::Super_L | Key::Super_R if open_modifier != Modifier::Super => *mods &= !0b1000,
        _ => (),
    };

    if key == Key::Shift_L || key == Key::Shift_R {
        event_sender
            .send_blocking(TransferType::ShiftOverview(false))
            .warn("unable to send");
    } else if key == Key::from_name(&*open_key).unwrap_or(Key::VoidSymbol) {
        event_sender
            .send_blocking(TransferType::TabOverview(false))
            .warn("unable to send");
    }

    tracing::trace!("key: {}{:?}, mods: {}", key, key, mods);
}

#[allow(clippy::too_many_arguments)]
fn handle_key(
    entry: &Entry,
    key: Key,
    plugin_keys: &[Key],
    open_modifier: Modifier,
    launch_modifier: Modifier,
    results: gtk::Box,
    mods: Arc<Mutex<u16>>,
    event_sender: Sender<TransferType>,
    grave: bool,
) -> Propagation {
    let mut mods = mods.lock().unwrap();
    match key {
        Key::Shift_L | Key::Shift_R if open_modifier != Modifier::Shift => *mods |= 0b0001,
        Key::Control_L | Key::Control_R if open_modifier != Modifier::Ctrl => *mods |= 0b0010,
        Key::Alt_L | Key::Alt_R if open_modifier != Modifier::Alt => *mods |= 0b0100,
        Key::Super_L | Key::Super_R if open_modifier != Modifier::Super => *mods |= 0b1000,
        _ => (),
    };
    let launch_mod = match launch_modifier {
        Modifier::Shift => (*mods & 0b0001) != 0,
        Modifier::Ctrl => (*mods & 0b0010) != 0,
        Modifier::Alt => (*mods & 0b0100) != 0,
        Modifier::Super => (*mods & 0b1000) != 0,
    };
    tracing::trace!(
        "key: {}{:?}, mods: {}, launch_mod: {}, launch_modifier: {}",
        key,
        key,
        mods,
        launch_mod,
        launch_modifier
    );
    if launch_mod && plugin_keys.contains(&key) {
        let ch = key
            .name()
            .unwrap_or_default()
            .to_string()
            .pop()
            .unwrap_or('a');
        event_sender
            .send_blocking(TransferType::CloseOverview(
                CloseOverviewConfig::LauncherPress(ch),
            ))
            .warn("unable to send");
        return Propagation::Stop;
    }

    if ((key == Key::Alt_L || key == Key::Alt_R) && open_modifier == Modifier::Alt)
        || ((key == Key::Control_L || key == Key::Control_R) && open_modifier == Modifier::Ctrl)
        || ((key == Key::Super_L || key == Key::Super_R) && open_modifier == Modifier::Super)
    {
        event_sender
            .send_blocking(TransferType::CloseOverview(CloseOverviewConfig::None))
            .warn("unable to send");
        return Propagation::Stop;
    }

    match (launch_mod, key) {
        (_, Key::Escape) => {
            event_sender
                .send_blocking(TransferType::Exit)
                .warn("unable to send");
            *mods = 0b0000;
            Propagation::Stop
        }
        (_, Key::Tab) => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: false,
                    direction: Direction::Forward,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        (_, Key::ISO_Left_Tab) => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: false,
                    direction: Direction::Backward,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        (_, Key::grave) => {
            if grave {
                event_sender
                    .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                        workspace: false,
                        direction: Direction::Left,
                    }))
                    .warn("unable to send");
                Propagation::Stop
            } else {
                Propagation::Proceed
            }
        }
        (_, Key::Up) => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Up,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        (_, Key::Down) => {
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Down,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        (_, Key::Left) => {
            if !entry.text().is_empty() {
                // allow to use in text in launcher
                return Propagation::Proceed;
            }
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Left,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        (_, Key::Right) => {
            if !entry.text().is_empty() {
                // allow to use in text in launcher
                return Propagation::Proceed;
            }
            event_sender
                .send_blocking(TransferType::SwitchOverview(SwitchOverviewConfig {
                    workspace: true,
                    direction: Direction::Right,
                }))
                .warn("unable to send");
            Propagation::Stop
        }
        (_, Key::Return) | (_, Key::KP_Enter) => {
            if results.first_child().is_some() {
                event_sender
                    .send_blocking(TransferType::CloseOverview(CloseOverviewConfig::None))
                    .warn("unable to send");
                *mods = 0b0000;
            }
            Propagation::Stop
        }
        (true, Key::_1) => {
            if results.observe_children().into_iter().len() > 1 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('1'),
                    ))
                    .warn("unable to send");
                *mods = 0b0000;
            }
            Propagation::Stop
        }
        (true, Key::_2) => {
            if results.observe_children().into_iter().len() > 2 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('2'),
                    ))
                    .warn("unable to send");
                *mods = 0b0000;
            }
            Propagation::Stop
        }
        (true, Key::_3) => {
            if results.observe_children().into_iter().len() > 3 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('3'),
                    ))
                    .warn("unable to send");
                *mods = 0b0000;
            }
            Propagation::Stop
        }
        (true, Key::_4) => {
            if results.observe_children().into_iter().len() > 4 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('4'),
                    ))
                    .warn("unable to send");
                *mods = 0b0000;
            }
            Propagation::Stop
        }
        (true, Key::_5) => {
            if results.observe_children().into_iter().len() > 5 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('5'),
                    ))
                    .warn("unable to send");
                *mods = 0b0000;
            }
            Propagation::Stop
        }
        (true, Key::_6) => {
            if results.observe_children().into_iter().len() > 6 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('6'),
                    ))
                    .warn("unable to send");
                *mods = 0b0000;
            }
            Propagation::Stop
        }
        (true, Key::_7) => {
            if results.observe_children().into_iter().len() > 7 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('7'),
                    ))
                    .warn("unable to send");
                *mods = 0b0000;
            }
            Propagation::Stop
        }
        (true, Key::_8) => {
            if results.observe_children().into_iter().len() > 8 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('8'),
                    ))
                    .warn("unable to send");
                *mods = 0b0000;
            }
            Propagation::Stop
        }
        (true, Key::_9) => {
            if results.observe_children().into_iter().len() > 9 {
                event_sender
                    .send_blocking(TransferType::CloseOverview(
                        CloseOverviewConfig::LauncherPress('9'),
                    ))
                    .warn("unable to send");
                *mods = 0b0000;
            }
            Propagation::Stop
        }
        _ => Propagation::Proceed,
    }
}
