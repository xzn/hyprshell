use crate::start::Globals;
use crate::util::reload_desktop_data;
use async_channel::{Receiver, Sender};
use core_lib::WarnWithDetails;
use core_lib::transfer::{
    CloseOverviewConfig, Direction, OpenOverview, OpenSwitch, SwitchOverviewConfig,
    SwitchSwitchConfig, TransferType,
};
use gtk::gio::prelude::ApplicationExt;
use gtk::prelude::EntryExt;
use tracing::{debug, trace, warn};
use windows_lib::create_windows_overview_window;

pub async fn event_handler(
    mut globals: Globals,
    event_receiver: Receiver<TransferType>,
    event_sender: Sender<TransferType>,
) {
    loop {
        if let Ok(transfer) = event_receiver.recv().await {
            let close_socket = matches!(transfer, TransferType::Restart);
            trace!("handling event: {transfer:?}");
            match transfer {
                TransferType::OpenOverview(config) => {
                    open_overview(&mut globals, event_sender.clone(), config)
                }
                TransferType::OpenSwitch(config) => open_switch(&mut globals, config),
                TransferType::SwitchOverview(config) => switch_overview(&mut globals, config),
                TransferType::SwitchSwitch(config) => switch_switch(&mut globals, config),
                TransferType::Exit => exit(&mut globals),
                TransferType::Type(text) => r#type(&mut globals, text, event_sender.clone()),
                TransferType::CloseOverview(config) => close_overview(&mut globals, config),
                TransferType::CloseSwitch => close_switch(&mut globals),
                TransferType::Restart => restart(&globals),
                TransferType::ShiftOverview(shift) => shift_overview(&mut globals, shift),
                TransferType::TabOverview(tab) => tab_overview(&mut globals, tab),
                TransferType::ShiftSwitch(shift) => shift_switch(&mut globals, shift),
                TransferType::TabSwitch(tab) => tab_switch(&mut globals, tab),
            }
            if close_socket {
                return;
            }
        }
    }
}

fn r#type(global: &mut Globals, text: String, event_sender: Sender<TransferType>) {
    if let Some(windows) = &mut global.windows {
        if let (Some(_overview), Some(launcher)) = &mut windows.overview {
            launcher_lib::update_launcher(launcher, text, event_sender)
        }
    }
}

fn open_overview(global: &mut Globals, event_sender: Sender<TransferType>, config: OpenOverview) {
    if let Some(windows) = &mut global.windows {
        if let (some_overview, Some(launcher)) = &mut windows.overview {
            let overview = if let Some(overview) = some_overview {
                overview
            } else if let Some(windows) = &global.config.windows
                && let Some(overview) = &windows.overview
            {
                let overview_data = create_windows_overview_window(&global.app, overview, windows);
                if let Ok(overview) = overview_data {
                    *some_overview = Some(overview);
                    if let Some(overview) = some_overview {
                        overview
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            } else {
                return;
            };

            if windows_lib::overview_already_open(overview) {
                switch_overview(
                    global,
                    SwitchOverviewConfig {
                        direction: if config.reverse {
                            Direction::Backward
                        } else {
                            Direction::Forward
                        },
                        workspace: false,
                    },
                );
            } else if !&windows
                .switch
                .as_ref()
                .map(windows_lib::switch_already_open)
                .unwrap_or(false)
            {
                windows_lib::open_overview(overview, event_sender)
                    .warn("Failed to open overview window");
                overview.shift = config.reverse;
                overview.tab = true;
                overview.opened = true;
                launcher_lib::open_launcher(launcher)
            } else {
                warn!("Overview or Switch already open");
            }
        } else {
            warn!("Window overview not active");
        }
    } else {
        warn!("Windows not active");
    };
}

fn open_switch(global: &mut Globals, config: OpenSwitch) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            if windows_lib::switch_already_open(switch) {
                switch_switch(
                    global,
                    SwitchSwitchConfig {
                        reverse: config.reverse,
                    },
                );
            } else if if let (Some(overview_data), _) = &windows.overview {
                !windows_lib::overview_already_open(overview_data)
            } else {
                true
            } {
                switch.shift = config.reverse;
                switch.tab = true;
                switch.opened = true;
                windows_lib::open_switch(switch, config).warn("Failed to open switch window");
            } else {
                warn!("Switch or Overview already open");
            }
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}

fn shift_overview(global: &mut Globals, shift: bool) {
    if let Some(windows) = &mut global.windows {
        if let (Some(overview), _) = &mut windows.overview {
            // if !overview.tab {
            overview.shift = shift;
            // }
        } else {
            warn!("Window overview not active");
        }
    } else {
        warn!("Windows not active");
    }
}
fn tab_overview(global: &mut Globals, tab: bool) {
    if let Some(windows) = &mut global.windows {
        if let (Some(overview), _) = &mut windows.overview {
            overview.tab = tab;
        } else {
            warn!("Window overview not active");
        }
    } else {
        warn!("Windows not active");
    }
}
fn shift_switch(global: &mut Globals, shift: bool) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            // if !switch.tab {
            switch.shift = shift;
            // }
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}
fn tab_switch(global: &mut Globals, tab: bool) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            switch.tab = tab;
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}
fn switch_switch(global: &mut Globals, config: SwitchSwitchConfig) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            if switch.shift {
                windows_lib::update_switch(switch, SwitchSwitchConfig { reverse: true });
            } else {
                windows_lib::update_switch(switch, config);
            }
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}
fn switch_overview(global: &mut Globals, config: SwitchOverviewConfig) {
    if let Some(windows) = &mut global.windows {
        if let (Some(overview), Some(launcher)) = &mut windows.overview {
            // don't switch selected window if launcher is active
            let launch = launcher.entry.text_length() > 0;
            if !launch {
                if overview.shift {
                    let mut config = config;
                    if config.direction == Direction::Forward {
                        config.direction = Direction::Backward;
                    }
                    windows_lib::update_overview(overview, config);
                } else {
                    windows_lib::update_overview(overview, config);
                }
            }
        } else {
            warn!("Window switch not active");
        }
    } else {
        warn!("Windows not active");
    }
}
fn exit(global: &mut Globals) {
    if let Some(windows) = &mut global.windows {
        let (some_overview, some_launcher) = &mut windows.overview;
        if let Some(overview) = some_overview {
            windows_lib::close_overview(overview, None);
        }
        if let Some(launcher) = some_launcher {
            launcher_lib::close_launcher_by_char(launcher, None); // this will never open a program and need the default terminal
        }
        if let Some(switch) = &mut windows.switch {
            windows_lib::close_switch(switch, false);
        };
    }
    reload_desktop_data();
}

fn close_overview(global: &mut Globals, config: CloseOverviewConfig) {
    if let Some(windows) = &mut global.windows {
        if let (some_overview, Some(launcher)) = &mut windows.overview {
            if let Some(overview) = some_overview {
                if overview.opened
                /* && !overview.tab */
                {
                    match config {
                        // return (focus active)
                        CloseOverviewConfig::None => {
                            let launcher_empty = launcher.entry.text_length() == 0;
                            let launcher_no_items = launcher.sorted_matches.is_empty();
                            if launcher_empty {
                                // close overview, kill launcher
                                windows_lib::close_overview(overview, Some(None));
                                launcher_lib::close_launcher_by_char(launcher, None);
                            } else if launcher_no_items {
                                debug!("Launcher is empty, not closing");
                            } else {
                                // kill overview, close launcher
                                windows_lib::close_overview(overview, None);
                                launcher_lib::close_launcher_by_char(launcher, Some('0'));
                            };
                        }
                        // clicked on launcher item
                        CloseOverviewConfig::LauncherClick(iden) => {
                            windows_lib::close_overview(overview, None);
                            launcher_lib::close_launcher_by_iden(launcher, &iden);
                        }
                        // typed a character in launcher
                        CloseOverviewConfig::LauncherPress(iden) => {
                            windows_lib::close_overview(overview, None);
                            launcher_lib::close_launcher_by_char(launcher, Some(iden));
                        }
                        // clicked on window
                        CloseOverviewConfig::Windows(iden) => {
                            windows_lib::close_overview(overview, Some(Some(iden)));
                            launcher_lib::close_launcher_by_char(launcher, None);
                        }
                    }
                    overview.opened = false;
                }
                windows_lib::stop_overview(overview);
                *some_overview = None;
            }
        }
    }
    reload_desktop_data()
}

fn close_switch(global: &mut Globals) {
    if let Some(windows) = &mut global.windows {
        if let Some(switch) = &mut windows.switch {
            if switch.opened
            /* && !switch.tab */
            {
                windows_lib::close_switch(switch, true);
                switch.opened = false;
            }
        }
    }
    reload_desktop_data()
}

fn restart(global: &Globals) {
    if let Some(windows) = &global.windows {
        let (some_overview, some_launcher) = &windows.overview;
        if let Some(overview) = some_overview {
            windows_lib::stop_overview(overview);
        }
        if let Some(launcher) = some_launcher {
            launcher_lib::stop_launcher(launcher);
        };
        if let Some(switch) = &windows.switch {
            windows_lib::stop_switch(switch);
        };
    }
    global.app.quit();
}
