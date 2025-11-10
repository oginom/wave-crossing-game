use bevy::prelude::*;
use crate::GameState;
use super::systems::*;

/// プレイヤー機能を提供するプラグイン
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                circular_movement_system.run_if(in_state(GameState::InGame))
            );
    }
}
