use bevy::app::App;

pub mod debug_info;
pub mod input;
pub mod player;
pub mod portal;
pub mod scene;
pub mod ui;

mod seal {
    pub struct Seal;
}

pub trait AppExt {
    fn register_types(&mut self) -> &mut Self;

    /// This method is used to seal the trait and prevent downstream crates from implementing it.
    fn _seal(seal: seal::Seal);
}

impl AppExt for App {
    fn register_types(&mut self) -> &mut Self {
        use player::AppExt as PlayerAppExt;
        use portal::AppExt as PortalAppExt;

        self.register_player_types().register_portal_types()
    }

    fn _seal(_seal: seal::Seal) {}
}
