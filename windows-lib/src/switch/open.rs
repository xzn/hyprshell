use crate::data::{SortConfig, collect_data};
use crate::global::WindowsSwitchData;
use crate::icon::set_icon;
use crate::next::find_next;
use anyhow::Context;
use core_lib::data::Active;
use core_lib::transfer::{Direction, OpenSwitch};
use core_lib::{ClientData, ClientId, WarnWithDetails};
use exec_lib::{get_current_monitor, set_remain_focused};
use gtk::gdk::Cursor;
use gtk::prelude::*;
use gtk::{Button, Fixed, Frame, Image, Label, Overflow, Overlay, pango};
use tracing::{Level, span, trace};

fn scale(value: i16, scale: f64) -> i32 {
    (value as f64 / (15f64 - scale)) as i32
}

pub fn switch_already_open(data: &WindowsSwitchData) -> bool {
    data.window.get_visible()
}

pub fn open_switch(data: &mut WindowsSwitchData, config: OpenSwitch) -> anyhow::Result<()> {
    let _span = span!(Level::TRACE, "open_switch").entered();
    set_remain_focused().warn("Failed to set no follow mouse");

    let (clients_data, active_prev) = collect_data(&SortConfig {
        filter_current_monitor: data.config.filter_current_monitor,
        filter_current_workspace: data.config.filter_current_workspace,
        filter_same_class: data.config.filter_same_class,
        sort_recent: true,
    })
    .context("Failed to collect data")?;

    // TODO find a way to check if the key was already released and close the window or open it faster
    trace!("Showing window {:?}", data.window.id());
    data.window.set_visible(true);

    let current_monitor = get_current_monitor().context("Failed to get current monitor")?;

    if data.config.show_workspaces {
        let mut hypr_clients = Vec::new();
        let mut hypr_workspaces = Vec::new();
        let mut hypr_active = Some(active_prev);
        let mut hypr_client = None;
        for (wid, workspace) in &clients_data.workspaces {
            let clients: Vec<&(ClientId, ClientData)> = {
                let mut clients = clients_data
                    .clients
                    .iter()
                    .filter(|(_, client)| client.workspace == *wid && client.enabled)
                    .collect::<Vec<_>>();
                clients.sort_by(|(_, a), (_, b)| {
                    // prefer smaller windows
                    if a.floating && b.floating {
                        (b.width * b.height).cmp(&(a.width * a.height))
                    } else {
                        a.floating.cmp(&b.floating)
                    }
                });
                clients
            };
            if clients.is_empty() {
                if let Some(workspace) = hypr_active
                    && workspace.workspace == *wid
                {
                    hypr_active = None;
                }
                continue;
            }
            if hypr_client.is_none() {
                if let Some(client) = clients.get(0) {
                    hypr_client = Some((*client).clone());
                }
            }
            if hypr_active.is_none() {
                if let Some(client) = clients.get(0) {
                    hypr_active = Some(Active {
                        client: Some(client.0),
                        workspace: *wid,
                        monitor: client.1.monitor,
                    });
                }
            }
            hypr_clients.append(&mut clients.iter().map(|client| (*client).clone()).collect());
            hypr_workspaces.push((*wid, workspace.clone()));
            // TODO add strip_html_from_workspace_title;
            let workspace_fixed = Fixed::builder()
                .width_request(scale(workspace.width as i16, data.config.scale))
                .height_request(scale(workspace.height as i16, data.config.scale))
                .build();
            let workspace_button = {
                let workspace_overlay = Overlay::builder().child(&workspace_fixed).build();
                let button = Button::builder()
                    .child(&workspace_overlay)
                    .css_classes(["workspace", "no-hover"])
                    .build();
                button.set_cursor(Cursor::from_name("pointer", None).as_ref());
                // if active.workspace == *wid {
                //     button.add_css_class("active");
                // }
                button
            };
            data.main_flow.insert(&workspace_button, -1);
            data.workspaces.insert(*wid, workspace_button);

            for (address, client) in clients {
                if !client.enabled {
                    continue;
                }

                let client_button = {
                    let title = if !client.title.trim().is_empty() {
                        &client.title
                    } else {
                        &client.class
                    };
                    let client_label = Label::builder()
                        .label(title)
                        .overflow(Overflow::Visible)
                        .margin_start(6)
                        .ellipsize(pango::EllipsizeMode::End)
                        .build();
                    let client_frame = Frame::builder()
                        .label_xalign(0.5)
                        .label_widget(&client_label)
                        .build();

                    // hide picture if client so small
                    let client_h_w = scale(client.height, data.config.scale)
                        .min(scale(client.width, data.config.scale));
                    if client_h_w > 70 {
                        let image = Image::builder()
                            .css_classes(["client-image"])
                            .pixel_size((client_h_w.clamp(50, 600) as f64 / 1.6) as i32 - 20)
                            .build();
                        if !client.enabled {
                            image.add_css_class("monochrome");
                        }
                        set_icon(&client.class, client.pid, &image);
                        client_frame.set_child(Some(&image));
                    }

                    let client_overlay = Overlay::builder()
                        .overflow(Overflow::Hidden)
                        .child(&client_frame)
                        .build();
                    let button = Button::builder()
                        .child(&client_overlay)
                        .css_classes(["client", "no-hover"])
                        .width_request(scale(client.width, data.config.scale))
                        .height_request(scale(client.height, data.config.scale))
                        .build();
                    button.set_cursor(Cursor::from_name("pointer", None).as_ref());

                    // add initial border around initial active client
                    // if active.client == Some(*address) {
                    //     button.add_css_class("active");
                    // }
                    button
                };
                workspace_fixed.put(
                    &client_button,
                    scale(client.x - workspace.x as i16, data.config.scale) as f64,
                    scale(client.y - workspace.y as i16, data.config.scale) as f64,
                );
                data.clients.insert(*address, client_button);
            }
        }
        data.hypr_data = clients_data;
        data.hypr_data.clients = hypr_clients;
        data.hypr_data.workspaces = hypr_workspaces;

        data.active = if let Some(active) = hypr_active {
            active
        } else if let Some(client) = hypr_client {
            Active {
                client: Some(client.0),
                workspace: client.1.workspace,
                monitor: client.1.monitor,
            }
        } else {
            return Err(anyhow::anyhow!("No visible clients"));
        };

        if data.config.show_workspaces {
            if let Some(button) = data.workspaces.get(&data.active.workspace) {
                button.add_css_class("active");
            }
        } else {
            if let Some(client) = data.active.client
                && let Some(button) = data.clients.get(&client)
            {
                button.add_css_class("active");
            }
        }
    } else {
        let active = find_next(
            &if config.reverse {
                Direction::Left
            } else {
                Direction::Right
            },
            data.config.show_workspaces,
            true,
            &clients_data,
            active_prev,
            data.config.items_per_row as usize,
        );

        for (address, client) in &clients_data.clients {
            if !client.enabled {
                continue;
            }
            let client_button = {
                let title = if !client.title.trim().is_empty() {
                    &client.title
                } else {
                    &client.class
                };
                let client_label = Label::builder()
                    .label(title)
                    .overflow(Overflow::Visible)
                    .margin_start(6)
                    .ellipsize(pango::EllipsizeMode::End)
                    .build();
                let client_frame = Frame::builder()
                    .label_xalign(0.5)
                    .label_widget(&client_label)
                    .build();

                // hide picture if client so small
                let client_h_w =
                    scale(current_monitor.height as i16, data.config.scale).min(scale(
                        (current_monitor.height as f32 / current_monitor.scale) as i16,
                        data.config.scale,
                    ));
                if client_h_w > 70 {
                    let image = Image::builder()
                        .css_classes(["client-image"])
                        .pixel_size((client_h_w.clamp(50, 600) as f64 / 1.5) as i32 - 20)
                        .build();
                    if !client.enabled {
                        image.add_css_class("monochrome");
                    }
                    set_icon(&client.class, client.pid, &image);
                    client_frame.set_child(Some(&image));
                }

                let client_overlay = Overlay::builder()
                    .overflow(Overflow::Hidden)
                    .child(&client_frame)
                    .build();
                let button = Button::builder()
                    .child(&client_overlay)
                    .css_classes(["client", "no-hover"])
                    .width_request(scale(
                        (current_monitor.width as f32 / current_monitor.scale) as i16,
                        data.config.scale,
                    ))
                    .height_request(scale(
                        (current_monitor.height as f32 / current_monitor.scale) as i16,
                        data.config.scale,
                    ))
                    .build();
                button.set_cursor(Cursor::from_name("pointer", None).as_ref());

                // add border around initial active client
                if active.client == Some(*address) {
                    button.add_css_class("active");
                }
                button
            };
            data.main_flow.insert(&client_button, -1);
            data.clients.insert(*address, client_button);
        }
        data.hypr_data = clients_data;
        data.active = active;
    }
    Ok(())
}
