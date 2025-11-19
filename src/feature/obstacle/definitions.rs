use serde::{Deserialize, Serialize};
use crate::core::types::GridPosition;
use super::components::ObstacleKind;

/// 障害物のスポーン定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObstacleDefinition {
    pub kind: ObstacleKind,
    pub grid_pos: GridPosition,
}

/// 障害物の視覚設定
#[derive(Debug, Clone)]
pub struct ObstacleVisualConfig {
    pub kind: ObstacleKind,
    pub color: (f32, f32, f32),
    pub size: f32,  // グリッドサイズに対する倍率（0.8 = 80%）
}

impl ObstacleVisualConfig {
    pub fn get_config(kind: ObstacleKind) -> Self {
        match kind {
            ObstacleKind::Swamp => Self {
                kind,
                color: (0.4, 0.3, 0.2),  // 茶色
                size: 0.9,
            },
            ObstacleKind::Wind => Self {
                kind,
                color: (0.6, 0.9, 1.0),  // 水色
                size: 0.8,
            },
        }
    }
}
