use bevy::prelude::*;
use crate::core::Direction;

/// モンスターを示すマーカーコンポーネント
#[derive(Component)]
pub struct Monster;

/// モンスターの状態
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterState {
    /// 画面端で待機中
    Staging,
    /// 移動中
    Moving,
    /// 到達（消滅待ち）
    Reached,
}

/// モンスターの本来のパラメータ（アイテムなどの影響を受けない基本値）
#[derive(Component, Debug, Clone, Copy)]
pub struct MonsterProperty {
    pub base_direction: Direction,
    pub base_speed: f32,
}

impl MonsterProperty {
    pub fn new(direction: Direction, speed: f32) -> Self {
        Self {
            base_direction: direction,
            base_speed: speed,
        }
    }
}

/// 移動情報（実際の移動に使用される、アイテムや環境の影響を受けた値）
#[derive(Component, Debug, Clone, Copy)]
pub struct Movement {
    pub direction: Direction,
    pub speed: f32,
}

impl Movement {
    pub fn new(direction: Direction, speed: f32) -> Self {
        Self { direction, speed }
    }
}

/// 待機タイマー
#[derive(Component, Debug)]
pub struct StagingTimer {
    pub remaining: f32,
}

impl StagingTimer {
    pub fn new(duration: f32) -> Self {
        Self { remaining: duration }
    }
}

/// 衝突判定用のボックス
#[derive(Component, Debug, Clone, Copy)]
pub struct CollisionBox {
    pub size: Vec2,
}

impl CollisionBox {
    pub fn new(size: Vec2) -> Self {
        Self { size }
    }
}

/// 衝突状態
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct CollisionState {
    pub is_colliding: bool,
}

impl CollisionState {
    pub fn new() -> Self {
        Self { is_colliding: false }
    }
}
