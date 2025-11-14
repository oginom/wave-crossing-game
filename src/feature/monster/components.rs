use bevy::prelude::*;
use crate::core::Direction;
use crate::core::level;
use super::definitions::MonsterKind;

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
    pub kind: MonsterKind,
    pub base_direction: Direction,
    pub base_speed: f32,
    pub base_size: f32,
    pub base_color: (f32, f32, f32),
}

impl MonsterProperty {
    pub fn new(kind: MonsterKind, direction: Direction, speed: f32, size: f32, color: (f32, f32, f32)) -> Self {
        Self {
            kind,
            base_direction: direction,
            base_speed: speed,
            base_size: size,
            base_color: color,
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

/// 待機メーター（モンスターが停止している時間を計測）
#[derive(Component, Debug, Clone, Copy)]
pub struct WaitMeter {
    /// 現在の待機時間（秒）
    pub current: f32,
    /// 消滅する待機時間の閾値（秒）
    pub threshold: f32,
    /// 前フレームでの移動状態（速度がゼロかどうか）
    pub was_stopped: bool,
}

impl WaitMeter {
    pub fn new(threshold: f32) -> Self {
        Self {
            current: 0.0,
            threshold,
            was_stopped: false,
        }
    }

    /// 進行度の比率（0.0～1.0）
    pub fn progress_ratio(&self) -> f32 {
        (self.current / self.threshold).min(1.0)
    }

    /// 閾値を超えているか
    pub fn is_expired(&self) -> bool {
        self.current >= self.threshold
    }
}

impl Default for WaitMeter {
    fn default() -> Self {
        Self::new(level::WAIT_THRESHOLD)
    }
}
