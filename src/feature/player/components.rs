use bevy::prelude::*;

/// プレイヤーキャラクターのマーカーコンポーネント
#[derive(Component)]
pub struct Player;

/// 円運動のパラメータ
#[derive(Component)]
pub struct CircularMotion {
    /// 円の中心座標
    pub center: Vec2,
    /// 円の半径
    pub radius: f32,
    /// 回転速度（ラジアン/秒）
    pub angular_velocity: f32,
    /// 現在の角度（ラジアン）
    pub current_angle: f32,
}

impl Default for CircularMotion {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            radius: 200.0,
            angular_velocity: 1.0, // 1 rad/s ≈ 57度/秒
            current_angle: 0.0,
        }
    }
}
