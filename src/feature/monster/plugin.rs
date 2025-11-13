use bevy::prelude::*;
use crate::GameState;
use super::spawn::*;
use super::staging::*;
use super::movement::*;
use super::collision::*;
use super::despawn::*;
use super::wait::*;
use super::events::*;

/// モンスター機能を提供するプラグイン
pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MonsterSpawnQueue>()
            .add_message::<MonsterDespawnEvent>()
            .add_systems(
                Update,
                (
                    spawn_monsters_system,
                    staging_timer_system,
                    collision_detection_system,
                    monster_movement_system,
                    update_wait_meter_system,
                    update_monster_color_system,
                    despawn_expired_monsters_system,
                    despawn_reached_monsters,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame))
            );
    }
}
