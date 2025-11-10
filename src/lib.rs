use bevy::prelude::*;

pub mod feature;
use feature::player::PlayerPlugin;

#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum GameState {
    #[default]
    InGame,
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins(PlayerPlugin);
    }
}
