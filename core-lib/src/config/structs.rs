use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::fmt::Display;

pub(crate) const CURRENT_CONFIG_VERSION: u16 = 1;

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[default = true]
    pub layerrules: bool,
    #[default = "ctrl+shift+alt, h"]
    pub kill_bind: Box<str>,
    #[default(Some(Windows::default()))]
    pub windows: Option<Windows>,
    #[default(CURRENT_CONFIG_VERSION)]
    pub version: u16,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Windows {
    #[default = 8.5]
    pub scale: f64,
    #[default = 5]
    pub items_per_row: u8,
    #[default(None)]
    pub overview: Option<Overview>,
    #[default(None)]
    pub switch: Option<Switch>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Overview {
    #[default = true]
    pub strip_html_from_workspace_title: bool,
    pub launcher: Launcher,
    #[default = "super_l"]
    pub key: Box<str>,
    #[default(Modifier::Super)]
    pub modifier: Modifier,
    #[default(Vec::new())]
    pub filter_by: Vec<FilterBy>,
    #[default = false]
    pub hide_filtered: bool,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Launcher {
    #[default(None)]
    pub default_terminal: Option<Box<str>>,
    #[default(Modifier::Ctrl)]
    pub launch_modifier: Modifier,
    #[default = 650]
    pub width: u32,
    #[default = 5]
    pub max_items: u8,
    #[default = true]
    pub show_when_empty: bool,
    #[default = 250]
    pub animate_launch_ms: u64,
    #[default(Plugins{
        applications: Some(ApplicationsPluginConfig::default()),
        terminal: Some(EmptyConfig::default()),
        shell: None,
        websearch: Some(WebSearchConfig::default()),
        calc: Some(EmptyConfig::default()),
        path: Some(EmptyConfig::default()),
    })]
    pub plugins: Plugins,
}

// no default for this, if some elements are missing, they should be None.
// if no config for plugins is provided, use the default value from the launcher.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plugins {
    pub applications: Option<ApplicationsPluginConfig>,
    pub terminal: Option<EmptyConfig>,
    pub shell: Option<EmptyConfig>,
    pub websearch: Option<WebSearchConfig>,
    pub calc: Option<EmptyConfig>,
    pub path: Option<EmptyConfig>,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct EmptyConfig {}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct ApplicationsPluginConfig {
    #[default = 4]
    pub run_cache_weeks: u8,
    #[default = true]
    pub show_execs: bool,
    #[default = false]
    pub show_actions_submenu: bool,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct WebSearchConfig {
    #[default(vec![SearchEngine {
        url: "https://www.google.com/search?q={}".into(),
        name: "Google".into(),
        key: 'g',
    }, SearchEngine {
        url: "https://en.wikipedia.org/wiki/Special:Search?search={}".into(),
        name: "Wikipedia".into(),
        key: 'w',
    }])]
    pub engines: Vec<SearchEngine>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchEngine {
    pub url: Box<str>,
    pub name: Box<str>,
    pub key: char,
}

#[derive(SmartDefault, Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "no-default-config-values"), serde(default))]
#[serde(deny_unknown_fields)]
pub struct Switch {
    #[default(Modifier::Alt)]
    pub modifier: Modifier,
    #[default(Vec::new())]
    pub filter_by: Vec<FilterBy>,
    #[default = false]
    pub show_workspaces: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterBy {
    SameClass,
    CurrentWorkspace,
    CurrentMonitor,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    Alt,
    Ctrl,
    Super,
    Shift,
}

impl Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Modifier::Alt => write!(f, "Alt"),
            Modifier::Ctrl => write!(f, "Ctrl"),
            Modifier::Super => write!(f, "Super"),
            Modifier::Shift => write!(f, "Shift"),
        }
    }
}

impl Modifier {
    pub fn to_l_key(self) -> &'static str {
        match self {
            Modifier::Alt => "alt_l",
            Modifier::Ctrl => "control_l",
            Modifier::Super => "super_l",
            Modifier::Shift => "shift_l",
        }
    }

    pub fn to_r_key(self) -> &'static str {
        match self {
            Modifier::Alt => "alt_r",
            Modifier::Ctrl => "control_r",
            Modifier::Super => "super_r",
            Modifier::Shift => "shift_r",
        }
    }
}
