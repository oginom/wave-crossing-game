use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use super::components::*;

/// モンスターの特殊挙動を定義するコンポーネント
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SpecialBehavior {
    /// 特殊挙動なし（標準的な挙動のみ）
    None,

    /// すり抜け: 他のモンスターと相互衝突せずにすれ違う
    PassThrough,

    /// マイペース: 時々一定時間立ち止まる
    MyPace {
        stop_interval: f32,  // 立ち止まる間隔（秒）
        stop_duration: f32,  // 立ち止まる時間（秒）
    },
}

/// マイペース挙動用のタイマーコンポーネント
#[derive(Component)]
pub struct MyPaceTimer {
    pub interval_timer: Timer,
    pub stop_timer: Timer,
    pub is_stopped: bool,
}

impl MyPaceTimer {
    pub fn new(interval: f32, duration: f32) -> Self {
        Self {
            interval_timer: Timer::from_seconds(interval, TimerMode::Repeating),
            stop_timer: Timer::from_seconds(duration, TimerMode::Once),
            is_stopped: false,
        }
    }
}

/// マイペース挙動の処理システム
pub fn my_pace_system(
    time: Res<Time>,
    mut query: Query<(&SpecialBehavior, &mut MyPaceTimer, &mut Movement)>,
) {
    for (behavior, mut timer, mut movement) in &mut query {
        if let SpecialBehavior::MyPace { .. } = behavior {
            if timer.is_stopped {
                // 立ち止まり中
                timer.stop_timer.tick(time.delta());
                if timer.stop_timer.is_finished() {
                    // 立ち止まり終了、移動再開
                    timer.is_stopped = false;
                    movement.enabled = true;
                }
            } else {
                // 通常移動中
                timer.interval_timer.tick(time.delta());
                if timer.interval_timer.is_finished() {
                    // 立ち止まり開始
                    timer.is_stopped = true;
                    timer.stop_timer.reset();
                    movement.enabled = false;
                }
            }
        }
    }
}
