use bevy::prelude::*;
use crate::GameState;

use super::components::*;
use super::definitions::ObstacleVisualConfig;
use super::effects::{detect_swamp_system, apply_swamp_effect_system, wind_effect_system, wind_push_system};

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app
            // Startup: 障害物の配置（一時的にハードコード）
            .add_systems(Startup, spawn_obstacles_hardcoded)
            // Update: 効果の適用
            // 障害物効果は衝突検出の前に適用する必要がある
            .add_systems(
                Update,
                (
                    detect_swamp_system,        // 泥沼検出（OnSwampマーカーの付与/削除）
                    apply_swamp_effect_system,  // 泥沼効果適用
                    wind_effect_system,         // 風検出
                    wind_push_system,           // 風押し出し
                )
                    .chain()
                    .run_if(in_state(GameState::InGame))
                    .before(crate::feature::monster::collision::collision_detection_system)
            );
    }
}

/// プロトタイプ用：ハードコードで障害物を配置
fn spawn_obstacles_hardcoded(
    mut commands: Commands,
) {
    use crate::core::types::GridPosition;
    use crate::core::config::{GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT};
    use crate::core::types::grid_to_world;

    // 泥沼を3箇所配置
    let swamp_positions = vec![
        GridPosition { x: 3, y: 3 },
        GridPosition { x: 6, y: 7 },
        GridPosition { x: 4, y: 8 },
    ];

    for grid_pos in swamp_positions {
        let config = ObstacleVisualConfig::get_config(ObstacleKind::Swamp);
        let world_pos = grid_to_world(
            grid_pos,
            GRID_SIZE,
            FIELD_WIDTH,
            FIELD_HEIGHT,
        );

        commands.spawn((
            Obstacle,
            ObstacleKind::Swamp,
            ObstaclePosition { grid_pos },
            SwampEffect::default(),
            Sprite {
                color: Color::srgb(config.color.0, config.color.1, config.color.2),
                custom_size: Some(Vec2::splat(GRID_SIZE * config.size)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(0.0)),
        ));
    }

    // 風を3箇所配置
    let wind_positions = vec![
        GridPosition { x: 5, y: 2 },
        GridPosition { x: 8, y: 8 },
        GridPosition { x: 2, y: 5 },
    ];

    for grid_pos in wind_positions {
        let config = ObstacleVisualConfig::get_config(ObstacleKind::Wind);
        let world_pos = grid_to_world(
            grid_pos,
            GRID_SIZE,
            FIELD_WIDTH,
            FIELD_HEIGHT,
        );

        commands.spawn((
            Obstacle,
            ObstacleKind::Wind,
            ObstaclePosition { grid_pos },
            WindEffect,
            Sprite {
                color: Color::srgb(config.color.0, config.color.1, config.color.2),
                custom_size: Some(Vec2::splat(GRID_SIZE * config.size)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(0.0)),
        ));
    }
}
