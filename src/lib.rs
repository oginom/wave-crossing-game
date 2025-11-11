use bevy::prelude::*;

pub mod core;
pub mod feature;

use feature::player::PlayerPlugin;
use feature::world::WorldPlugin;

#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum GameState {
    #[default]
    InGame,
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                WorldPlugin,
                PlayerPlugin,
            ));
    }
}
