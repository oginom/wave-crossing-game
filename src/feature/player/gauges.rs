use bevy::prelude::*;
use crate::feature::monster::{MonsterDespawnEvent, DespawnCause};
use crate::GameState;
use crate::core::level;

/// プレイヤーのゲージ（魂と虚）
#[derive(Resource, Debug, Clone)]
pub struct PlayerGauges {
    pub spirit: SpiritGauge,
    pub void: VoidGauge,
}

impl Default for PlayerGauges {
    fn default() -> Self {
        Self {
            spirit: SpiritGauge::new(level::SPIRIT_MAX, level::SPIRIT_INITIAL),
            void: VoidGauge::new(level::VOID_MAX),
        }
    }
}

/// 魂（スピリット）ゲージ
#[derive(Debug, Clone)]
pub struct SpiritGauge {
    pub current: f32,
    pub max: f32,
}

impl SpiritGauge {
    pub fn new(max: f32, initial: f32) -> Self {
        Self {
            current: initial.min(max),
            max,
        }
    }

    /// 魂を増加させる
    pub fn add(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// 魂を消費する（成功したらtrue）
    pub fn consume(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    /// ゲージの比率（0.0～1.0）
    pub fn ratio(&self) -> f32 {
        self.current / self.max
    }
}

/// 虚（ヴォイド）ゲージ
#[derive(Debug, Clone)]
pub struct VoidGauge {
    pub current: f32,
    pub max: f32,
}

impl VoidGauge {
    pub fn new(max: f32) -> Self {
        Self {
            current: 0.0,
            max,
        }
    }

    /// 虚を増加させる
    pub fn add(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    /// ゲージが満タンか
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// ゲージの比率（0.0～1.0）
    pub fn ratio(&self) -> f32 {
        self.current / self.max
    }
}

/// モンスター消滅イベントを受けてゲージを更新
pub fn update_gauges_on_monster_event_system(
    mut gauges: ResMut<PlayerGauges>,
    mut events: MessageReader<MonsterDespawnEvent>,
) {
    for event in events.read() {
        match event.cause {
            DespawnCause::ReachedGoal => {
                gauges.spirit.add(level::SPIRIT_GAIN_PER_GOAL);
                info!(
                    "Spirit +{}: {:.1}/{:.1}",
                    level::SPIRIT_GAIN_PER_GOAL,
                    gauges.spirit.current,
                    gauges.spirit.max
                );
            }
            DespawnCause::WaitExpired => {
                gauges.void.add(level::VOID_GAIN_PER_DESPAWN);
                info!(
                    "Void +{}: {:.1}/{:.1}",
                    level::VOID_GAIN_PER_DESPAWN,
                    gauges.void.current,
                    gauges.void.max
                );
            }
        }
    }
}

/// Voidゲージが満タンになったらゲームオーバー
pub fn check_game_over_system(
    gauges: Res<PlayerGauges>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if gauges.void.is_full() {
        warn!("Game Over: Void gauge is full");
        next_state.set(GameState::GameOver);
    }
}
