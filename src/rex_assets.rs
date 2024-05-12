use bracket_lib::{terminal, terminal::rex::XpFile, terminal::EMBED};

terminal::embedded_resource!(SMALL_DUNGEON, "../resources/th.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    pub fn new() -> RexAssets {
        terminal::link_resource!(SMALL_DUNGEON, "../resources/th.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/th.xp").unwrap(),
        }
    }
}
