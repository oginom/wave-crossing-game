use bevy::prelude::*;
use crate::core::stage_asset::StageLevelAsset;
use crate::core::config::{GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT};
use crate::core::types::grid_to_world;
use crate::feature::monster::StageLevelLoader;
use super::components::*;
use super::definitions::ObstacleVisualConfig;

/// ステージアセットから障害物をスポーン
pub fn spawn_obstacles_from_stage(
    mut commands: Commands,
    stage_assets: Res<Assets<StageLevelAsset>>,
    stage_loader: Res<StageLevelLoader>,
    mut spawned: Local<bool>,
) {
    // 既にスポーン済みならスキップ
    if *spawned {
        return;
    }

    // ステージがロードされていなければスキップ
    if !stage_loader.loaded {
        return;
    }

    let Some(stage_asset) = stage_assets.get(&stage_loader.handle) else {
        return;
    };

    info!("Spawning {} obstacles from stage", stage_asset.obstacles.len());

    for obstacle_def in &stage_asset.obstacles {
        let config = ObstacleVisualConfig::get_config(obstacle_def.kind);
        let world_pos = grid_to_world(
            obstacle_def.grid_pos,
            GRID_SIZE,
            FIELD_WIDTH,
            FIELD_HEIGHT,
        );

        let mut entity_commands = commands.spawn((
            Obstacle,
            obstacle_def.kind,
            ObstaclePosition { grid_pos: obstacle_def.grid_pos },
            Sprite {
                color: Color::srgb(config.color.0, config.color.1, config.color.2),
                custom_size: Some(Vec2::splat(GRID_SIZE * config.size)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(0.0)),
        ));

        // 種類に応じた効果コンポーネントを追加
        match obstacle_def.kind {
            ObstacleKind::Swamp => {
                entity_commands.insert(SwampEffect::default());
            }
            ObstacleKind::Wind => {
                entity_commands.insert(WindEffect);
            }
        }
    }

    // スポーン完了をマーク
    *spawned = true;
}
