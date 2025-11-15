use bevy::prelude::*;

pub mod core;
pub mod feature;

use core::StageAssetPlugin;
use feature::world::WorldPlugin;
use feature::monster::MonsterPlugin;
use feature::item::ItemPlugin;
use feature::player::PlayerPlugin;

#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum GameState {
    #[default]
    InGame,
    GameOver,
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                StageAssetPlugin,
                WorldPlugin,
                MonsterPlugin,
                ItemPlugin,
                PlayerPlugin,
            ));
    }
}
