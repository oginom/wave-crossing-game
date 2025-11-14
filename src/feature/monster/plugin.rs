use bevy::prelude::*;
use crate::GameState;
use super::definitions::*;
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

/// モンスター定義を初期化（Phase 1: ハードコード）
fn setup_monster_definitions(mut definitions: ResMut<MonsterDefinitions>) {
    // 河童 - 標準的な速度と挙動
    definitions.insert(MonsterDefinition {
        kind: MonsterKind::Kappa,
        speed: 100.0,
        size: 0.6,
        color: (0.2, 0.8, 0.5),  // 緑色
        wait_threshold: 10.0,
    });

    // ゴースト - 高速移動
    definitions.insert(MonsterDefinition {
        kind: MonsterKind::Ghost,
        speed: 150.0,
        size: 0.5,
        color: (0.9, 0.9, 1.0),  // 白っぽい
        wait_threshold: 8.0,
    });

    // 化け猫 - 大型でゆっくり
    definitions.insert(MonsterDefinition {
        kind: MonsterKind::Bakeneko,
        speed: 70.0,
        size: 0.8,
        color: (0.3, 0.2, 0.4),  // 紫がかった色
        wait_threshold: 15.0,
    });
}
