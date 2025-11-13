use bevy::prelude::*;

/// モンスター消滅イベント
#[derive(Message, Debug, Clone, Copy)]
pub struct MonsterDespawnEvent {
    /// 消滅したモンスターのエンティティ
    pub entity: Entity,
    /// 消滅の原因
    pub cause: DespawnCause,
}

/// モンスター消滅の原因
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DespawnCause {
    /// ゴールに到達した
    ReachedGoal,
    /// 待機時間が閾値を超えた
    WaitExpired,
}
