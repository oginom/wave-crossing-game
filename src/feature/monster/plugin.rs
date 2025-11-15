use bevy::prelude::*;
use crate::GameState;
use crate::core::MonsterDefinitionsAsset;
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
            .init_resource::<MonsterDefinitions>()
            .add_message::<MonsterDespawnEvent>()
            .add_systems(Startup, (load_monster_definitions_system, load_stage_level_system))
            .add_systems(
                Update,
                (
                    initialize_monster_definitions_system,
                    initialize_spawn_queue_system,
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

/// モンスター定義アセットをロードするシステム（起動時に一度だけ実行）
fn load_monster_definitions_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<MonsterDefinitionsAsset> = asset_server.load("monsters.ron");
    commands.insert_resource(MonsterDefinitionsLoader {
        handle,
        loaded: false,
    });
}

/// モンスター定義が読み込まれたらMonsterDefinitionsを初期化するシステム
fn initialize_monster_definitions_system(
    mut loader: ResMut<MonsterDefinitionsLoader>,
    monster_def_assets: Res<Assets<MonsterDefinitionsAsset>>,
    mut definitions: ResMut<MonsterDefinitions>,
) {
    // すでにロード済みならスキップ
    if loader.loaded {
        return;
    }

    // アセットがロードされたか確認
    if let Some(monster_def_asset) = monster_def_assets.get(&loader.handle) {
        *definitions = MonsterDefinitions::from_hashmap(monster_def_asset.to_hashmap());
        loader.loaded = true;
        info!("Monster definitions loaded: {} types", monster_def_asset.definitions.len());
    }
}
