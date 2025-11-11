use bevy::prelude::*;
use super::components::*;

/// 到達したモンスターを消滅させるシステム
pub fn despawn_reached_monsters(
    mut commands: Commands,
    query: Query<(Entity, &MonsterState), With<Monster>>,
) {
    for (entity, state) in &query {
        if *state == MonsterState::Reached {
            commands.entity(entity).despawn();
            info!("Despawned monster entity {:?}", entity);
        }
    }
}
