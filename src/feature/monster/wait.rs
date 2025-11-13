use bevy::prelude::*;
use super::components::{Monster, WaitMeter, CollisionState, MonsterState};
use super::events::{MonsterDespawnEvent, DespawnCause};

/// モンスターの待機時間を更新
///
/// 停止中（衝突中）はwait値を増加させ、移動再開時にリセットする
pub fn update_wait_meter_system(
    time: Res<Time>,
    mut query: Query<(&CollisionState, &MonsterState, &mut WaitMeter), With<Monster>>,
) {
    for (collision_state, state, mut wait_meter) in query.iter_mut() {
        // Moving状態のモンスターのみ処理
        if *state != MonsterState::Moving {
            continue;
        }

        // 衝突中は停止しているとみなす
        let is_stopped = collision_state.is_colliding;

        if is_stopped {
            // 停止中：wait値を増加
            wait_meter.current += time.delta_secs();
            if wait_meter.current > wait_meter.was_stopped as u32 as f32 {
                trace!(
                    "Monster wait time: {:.1}/{:.1}s",
                    wait_meter.current,
                    wait_meter.threshold
                );
            }
        } else {
            // 移動中：wait値をリセット
            if wait_meter.current > 0.0 {
                debug!("Monster resumed movement, resetting wait meter");
                wait_meter.current = 0.0;
            }
        }

        wait_meter.was_stopped = is_stopped;
    }
}

/// wait値に応じてモンスターの色を変化させる
///
/// 待機時間が進むほど黒くなる（最大で30%の明るさまで落ちる）
pub fn update_monster_color_system(
    mut query: Query<(&WaitMeter, &mut Sprite), With<Monster>>,
) {
    for (wait_meter, mut sprite) in query.iter_mut() {
        let ratio = wait_meter.progress_ratio();

        // 待機時間が進むほど黒くなる（RGB値を減少）
        let brightness = 1.0 - (ratio * 0.7); // 最大で30%の明るさまで落ちる
        sprite.color = Color::srgb(brightness, brightness, brightness);
    }
}

/// wait値が閾値を超えたモンスターを消滅させる
///
/// 消滅時にMonsterDespawnEventを発行する
pub fn despawn_expired_monsters_system(
    mut commands: Commands,
    query: Query<(Entity, &WaitMeter), With<Monster>>,
    mut despawn_events: MessageWriter<MonsterDespawnEvent>,
) {
    for (entity, wait_meter) in query.iter() {
        if wait_meter.is_expired() {
            info!(
                "Monster despawned due to wait timeout ({:.1}s >= {:.1}s)",
                wait_meter.current, wait_meter.threshold
            );

            // イベントを発行
            despawn_events.write(MonsterDespawnEvent {
                entity,
                cause: DespawnCause::WaitExpired,
            });

            // エンティティを削除
            commands.entity(entity).despawn();
        }
    }
}
