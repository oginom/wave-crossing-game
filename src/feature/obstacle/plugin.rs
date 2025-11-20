use bevy::prelude::*;
use crate::GameState;

use super::spawn::spawn_obstacles_from_stage;
use super::effects::{detect_swamp_system, apply_swamp_effect_system, wind_effect_system, wind_push_system};

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app
            // Update: ステージアセットから障害物をスポーン
            .add_systems(Update, spawn_obstacles_from_stage)
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

