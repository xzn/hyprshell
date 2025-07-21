use crate::binds::structs::ExecBind;
use crate::config::Modifier;
use anyhow::bail;

pub fn generate_bind_kill(kill_bind: &str) -> anyhow::Result<ExecBind> {
    let a = kill_bind
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let mods = match a.first() {
        Some(s) => {
            let mut parsed_mods = Vec::new();
            for m in s.split('+') {
                let mod_parsed: anyhow::Result<Modifier> = match m.to_ascii_lowercase().as_str() {
                    "alt" => Ok(Modifier::Alt),
                    "ctrl" | "control" => Ok(Modifier::Ctrl),
                    "super" | "win" => Ok(Modifier::Super),
                    "shift" => Ok(Modifier::Shift),
                    _ => bail!("Unknown modifier: {}", m),
                };
                parsed_mods.push(mod_parsed?);
            }
            parsed_mods
        }
        None => bail!("No mods specified"),
    };
    let key = match a.get(1) {
        Some(s) => Box::from(*s),
        None => bail!("No key provided in bind: {}", kill_bind),
    };

    let bind = ExecBind {
        key,
        mods,
        on_release: false,
        exec: "pkill hyprshell".into(),
    };
    Ok(bind)
}
