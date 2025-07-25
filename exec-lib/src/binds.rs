use anyhow::Context;
use core_lib::binds::ExecBind;
use core_lib::config::Modifier;
use core_lib::{LAUNCHER_NAMESPACE, OVERVIEW_NAMESPACE, SWITCH_NAMESPACE};
use hyprland::config::binds;
use hyprland::config::binds::{Binder, Binding, Flag};
use hyprland::dispatch::DispatchType;
use hyprland::keyword::Keyword;
use tracing::trace;

pub fn apply_layerrules() -> anyhow::Result<()> {
    Keyword::set("layerrule", format!("noanim, {LAUNCHER_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("ignorezero, {LAUNCHER_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("noanim, {OVERVIEW_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("dimaround, {OVERVIEW_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("ignorezero, {OVERVIEW_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("noanim, {SWITCH_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("dimaround, {SWITCH_NAMESPACE}"))?;
    Keyword::set("layerrule", format!("ignorezero, {SWITCH_NAMESPACE}"))?;
    Ok(())
}

// ctrl+shift+alt, h
// hyprland::bind!(d e | SUPER, Key, "a" => Exec, "pkill hyprshell");
pub fn apply_exec_bind(bind: &ExecBind) -> anyhow::Result<()> {
    let binding = Binding {
        mods: bind
            .mods
            .iter()
            .map(|m| match m {
                Modifier::Alt => binds::Mod::ALT,
                Modifier::Ctrl => binds::Mod::CTRL,
                Modifier::Super => binds::Mod::SUPER,
                Modifier::Shift => binds::Mod::SHIFT,
            })
            .collect(),
        key: binds::Key::Key(&bind.key),
        flags: if bind.on_release {
            vec![Flag::n, Flag::i, Flag::t, Flag::r]
        } else {
            vec![Flag::e]
        },
        dispatcher: DispatchType::Exec(&bind.exec),
    };
    trace!("binding exec: {binding:?}");
    Binder::bind(binding).with_context(|| format!("binding exec failed: {bind:?}"))?;
    Ok(())
}
