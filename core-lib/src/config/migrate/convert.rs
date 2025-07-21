use crate::config;
use crate::config::CURRENT_CONFIG_VERSION;
use crate::config::migrate::old_structs;

impl From<old_structs::Config> for config::Config {
    fn from(value: old_structs::Config) -> Self {
        Self {
            layerrules: value.layerrules,
            kill_bind: value.kill_bind.into(),
            windows: value
                .windows
                .map(|a| old_structs::Windows::into(a, value.launcher)),
            version: CURRENT_CONFIG_VERSION,
        }
    }
}

impl old_structs::Windows {
    fn into(value: old_structs::Windows, launcher: Option<config::Launcher>) -> config::Windows {
        config::Windows {
            scale: value.scale,
            items_per_row: value.workspaces_per_row,
            switch: value.switch.map(old_structs::Switch::into),
            overview: value.overview.map(|o| {
                old_structs::Overview::into(o, launcher, value.strip_html_from_workspace_title)
            }),
        }
    }
}

impl old_structs::Overview {
    fn into(
        value: old_structs::Overview,
        launcher: Option<config::Launcher>,
        strip_html_from_workspace_title: bool,
    ) -> config::Overview {
        config::Overview {
            key: value.open.key.to_key().into(),
            modifier: value.open.modifier,
            filter_by: value.other.filter_by,
            launcher: launcher.unwrap_or_default(),
            hide_filtered: value.other.hide_filtered,
            strip_html_from_workspace_title,
            use_grave_for_reverse: false,
        }
    }
}

impl old_structs::Switch {
    fn into(value: old_structs::Switch) -> config::Switch {
        config::Switch {
            modifier: value.open.modifier,
            filter_by: value.other.filter_by,
            ..Default::default()
        }
    }
}
