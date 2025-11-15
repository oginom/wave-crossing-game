use bevy::prelude::*;
use crate::core::{GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT};
use super::components::*;

/// モンスターを移動させるシステム
pub fn monster_movement_system(
    time: Res<Time>,
    mut query: Query<(&Movement, &mut Transform, &mut MonsterState, &CollisionState), With<Monster>>,
) {
    for (movement, mut transform, mut state, collision) in &mut query {
        // Moving状態で、かつ衝突していない場合、かつ移動が有効な場合のみ移動
        if *state == MonsterState::Moving && !collision.is_colliding && movement.enabled {
            // 進行方向に移動
            let velocity = movement.direction.to_vector() * movement.speed;
            transform.translation += velocity.extend(0.0) * time.delta_secs();

            // フィールド外に出たかチェック
            if is_out_of_bounds(transform.translation.xy()) {
                *state = MonsterState::Reached;
                info!("Monster reached the edge at {:?}", transform.translation);
            }
        }
    }
}

/// フィールド外かどうかをチェック
fn is_out_of_bounds(position: Vec2) -> bool {
    let field_width = FIELD_WIDTH as f32 * GRID_SIZE;
    let field_height = FIELD_HEIGHT as f32 * GRID_SIZE;
    let margin = GRID_SIZE * 2.0; // フィールド外のマージン

    position.x < -field_width / 2.0 - margin
        || position.x > field_width / 2.0 + margin
        || position.y < -field_height / 2.0 - margin
        || position.y > field_height / 2.0 + margin
}
