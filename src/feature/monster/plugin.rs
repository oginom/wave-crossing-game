use bevy::prelude::*;
use crate::GameState;
use super::definitions::*;
use super::special_behavior::*;
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
            .init_resource::<MonsterDefinitions>()
            .add_message::<MonsterDespawnEvent>()
            .add_systems(Startup, setup_monster_definitions)
            .add_systems(
                Update,
                (
                    spawn_monsters_system,
                    staging_timer_system,
                    my_pace_system,  // 特殊挙動システムを追加
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

/// モンスター定義を初期化（Phase 2: RONファイルから読み込み）
fn setup_monster_definitions(mut definitions: ResMut<MonsterDefinitions>) {
    // RONファイルから定義を読み込む
    let loaded = MonsterDefinitions::from_file("assets/monsters.ron");
    *definitions = loaded;
}
