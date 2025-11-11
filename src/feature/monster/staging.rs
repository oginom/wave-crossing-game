use bevy::prelude::*;
use super::components::*;

/// 待機タイマーを更新し、時間が来たらMoving状態に遷移
pub fn staging_timer_system(
    time: Res<Time>,
    mut query: Query<(&mut StagingTimer, &mut MonsterState), With<Monster>>,
) {
    for (mut timer, mut state) in &mut query {
        if *state == MonsterState::Staging {
            timer.remaining -= time.delta_secs();
            if timer.remaining <= 0.0 {
                *state = MonsterState::Moving;
                info!("Monster started moving");
            }
        }
    }
}
