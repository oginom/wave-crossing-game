use bevy::prelude::*;
use crate::core::types::GridPosition;
use serde::{Deserialize, Serialize};

/// 障害物の基本コンポーネント
#[derive(Component)]
pub struct Obstacle;

/// 障害物の種類
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObstacleKind {
    Swamp,  // 泥沼
    Wind,   // 風
}

/// 障害物の配置情報
#[derive(Component, Debug, Clone, Copy)]
pub struct ObstaclePosition {
    pub grid_pos: GridPosition,
}

/// 泥沼の効果コンポーネント
#[derive(Component, Debug, Clone, Copy)]
pub struct SwampEffect {
    /// 速度倍率（0.5 = 半分）
    pub speed_multiplier: f32,
}

impl Default for SwampEffect {
    fn default() -> Self {
        Self {
            speed_multiplier: crate::core::level::SWAMP_SPEED_MULTIPLIER,
        }
    }
}

/// 風の効果コンポーネント
#[derive(Component, Debug, Clone, Copy)]
pub struct WindEffect;

/// 泥沼の上にいることを示すマーカーコンポーネント
#[derive(Component, Debug, Clone, Copy)]
pub struct OnSwamp {
    pub speed_multiplier: f32,
}
