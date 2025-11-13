use bevy::prelude::*;
use super::components::*;
use super::events::{MonsterDespawnEvent, DespawnCause};

/// 到達したモンスターを消滅させるシステム
pub fn despawn_reached_monsters(
    mut commands: Commands,
    query: Query<(Entity, &MonsterState), With<Monster>>,
    mut despawn_events: MessageWriter<MonsterDespawnEvent>,
) {
    for (entity, state) in &query {
        if *state == MonsterState::Reached {
            info!("Monster reached goal, despawning entity {:?}", entity);

            // ゴール到達イベントを発行
            despawn_events.write(MonsterDespawnEvent {
                entity,
                cause: DespawnCause::ReachedGoal,
            });

            commands.entity(entity).despawn();
        }
    }
}
