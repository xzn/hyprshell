use core_lib::binds::{ExecBind, generate_transfer_socat};
use core_lib::config::{Modifier, Windows};
use core_lib::transfer::{OpenOverview, OpenSwitch, TransferType};
use gtk::gdk::Key;

pub fn generate_open_keybinds(windows: &Windows) -> Vec<ExecBind> {
    let mut binds = Vec::new();
    if let Some(overview) = &windows.overview {
        binds.push(ExecBind {
            mods: vec![overview.modifier],
            key: overview.key.clone(),
            on_release: false,
            exec: generate_transfer_socat(&TransferType::OpenOverview(OpenOverview {
                reverse: false,
            }))
            .into_boxed_str(),
        });
        binds.push(ExecBind {
            mods: vec![overview.modifier, Modifier::Shift],
            key: overview.key.clone(),
            on_release: false,
            exec: generate_transfer_socat(&TransferType::OpenOverview(OpenOverview {
                reverse: true,
            }))
            .into_boxed_str(),
        });

        if overview.use_grave_for_reverse {
            binds.push(ExecBind {
                mods: vec![overview.modifier],
                key: Box::from("grave"),
                on_release: false,
                exec: generate_transfer_socat(&TransferType::OpenOverview(OpenOverview {
                    reverse: false,
                }))
                .into_boxed_str(),
            });
        }

        let mut overview_release = generate_transfer_socat(&TransferType::TabOverview(false));
        let switch_release = generate_transfer_socat(&TransferType::TabSwitch(false));

        if let Some(_) = &windows.switch {
            if Key::from_name(&*overview.key).unwrap_or(Key::VoidSymbol) == Key::Tab {
                overview_release.push_str(" && ");
                overview_release.push_str(&switch_release);
            } else {
                binds.push(ExecBind {
                    mods: vec![],
                    key: Box::from("tab"),
                    on_release: true,
                    exec: switch_release.into_boxed_str(),
                });
            }
        }

        binds.push(ExecBind {
            mods: vec![],
            key: overview.key.clone(),
            on_release: true,
            exec: overview_release.into_boxed_str(),
        });

        // binds.push(ExecBind {
        //     mods: vec![],
        //     key: overview.modifier.to_l_key().into(),
        //     on_release: true,
        //     exec: generate_transfer_socat(&TransferType::CloseOverview(
        //         core_lib::transfer::CloseOverviewConfig::None,
        //     ))
        //     .into_boxed_str(),
        // });

        // binds.push(ExecBind {
        //     mods: vec![],
        //     key: overview.modifier.to_r_key().into(),
        //     on_release: true,
        //     exec: generate_transfer_socat(&TransferType::CloseOverview(
        //         core_lib::transfer::CloseOverviewConfig::None,
        //     ))
        //     .into_boxed_str(),
        // });
    }
    if let Some(switch) = &windows.switch {
        binds.push(ExecBind {
            mods: vec![switch.modifier],
            key: Box::from("tab"),
            on_release: false,
            exec: generate_transfer_socat(&TransferType::OpenSwitch(OpenSwitch { reverse: false }))
                .into_boxed_str(),
        });
        if switch.use_grave_for_reverse {
            binds.push(ExecBind {
                mods: vec![switch.modifier],
                key: Box::from("grave"),
                on_release: false,
                exec: generate_transfer_socat(&TransferType::OpenSwitch(OpenSwitch {
                    reverse: true,
                }))
                .into_boxed_str(),
            });
        }
        binds.push(ExecBind {
            mods: vec![switch.modifier, Modifier::Shift],
            key: Box::from("tab"),
            on_release: false,
            exec: generate_transfer_socat(&TransferType::OpenSwitch(OpenSwitch { reverse: true }))
                .into_boxed_str(),
        });
        match switch.modifier {
            Modifier::Alt => {
                binds.push(ExecBind {
                    mods: vec![],
                    key: Box::from("Alt_L"),
                    on_release: true,
                    exec: generate_transfer_socat(&TransferType::CloseSwitch).into_boxed_str(),
                });
                binds.push(ExecBind {
                    mods: vec![],
                    key: Box::from("Alt_R"),
                    on_release: true,
                    exec: generate_transfer_socat(&TransferType::CloseSwitch).into_boxed_str(),
                });
            }
            Modifier::Ctrl => {
                binds.push(ExecBind {
                    mods: vec![],
                    key: Box::from("Control_L"),
                    on_release: true,
                    exec: generate_transfer_socat(&TransferType::CloseSwitch).into_boxed_str(),
                });
                binds.push(ExecBind {
                    mods: vec![],
                    key: Box::from("Control_R"),
                    on_release: true,
                    exec: generate_transfer_socat(&TransferType::CloseSwitch).into_boxed_str(),
                });
            }
            Modifier::Super => {
                binds.push(ExecBind {
                    mods: vec![],
                    key: Box::from("Super_L"),
                    on_release: true,
                    exec: generate_transfer_socat(&TransferType::CloseSwitch).into_boxed_str(),
                });
                binds.push(ExecBind {
                    mods: vec![],
                    key: Box::from("Super_R"),
                    on_release: true,
                    exec: generate_transfer_socat(&TransferType::CloseSwitch).into_boxed_str(),
                });
            }
            Modifier::Shift => {
                binds.push(ExecBind {
                    mods: vec![],
                    key: Box::from("Shift_L"),
                    on_release: true,
                    exec: generate_transfer_socat(&TransferType::CloseSwitch).into_boxed_str(),
                });
                binds.push(ExecBind {
                    mods: vec![],
                    key: Box::from("Shift_R"),
                    on_release: true,
                    exec: generate_transfer_socat(&TransferType::CloseSwitch).into_boxed_str(),
                });
            }
        }
    }

    binds
}
