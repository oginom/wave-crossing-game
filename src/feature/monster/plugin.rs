use bevy::prelude::*;
use crate::GameState;
use super::spawn::*;
use super::staging::*;
use super::movement::*;
use super::despawn::*;

/// モンスター機能を提供するプラグイン
pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MonsterSpawnQueue>()
            .add_systems(
                Update,
                (
                    spawn_monsters_system,
                    staging_timer_system,
                    monster_movement_system,
                    despawn_reached_monsters,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame))
            );
    }
}
