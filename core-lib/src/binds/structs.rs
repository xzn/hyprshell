use crate::config::Modifier;

#[derive(Debug)]
pub struct ExecBind {
    pub mods: Vec<Modifier>,
    pub key: Box<str>,
    // pub flags: Vec<Flag>,
    pub on_release: bool,
    pub exec: Box<str>,
    // hello from bene
}

// #[derive(Debug)]
// pub enum Flag {
//     AllowRepeat,
//     DontConsume,
// }
