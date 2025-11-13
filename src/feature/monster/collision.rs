use bevy::prelude::*;
use super::components::*;
use crate::core::GRID_SIZE;

/// 衝突検知システム
/// 次フレームの予測位置と現在の他モンスターの位置で矩形衝突判定を行う
pub fn collision_detection_system(
    mut query: Query<(Entity, &Transform, &Movement, &CollisionBox, &mut CollisionState, &MonsterState), With<Monster>>,
) {
    // 全モンスターの位置情報を事前に収集
    let monsters: Vec<_> = query
        .iter()
        .map(|(e, t, _, cb, _, s)| (e, t.translation, cb.size, *s))
        .collect();

    // 各モンスターについて衝突判定
    for (entity, transform, movement, collision_box, mut collision_state, state) in &mut query {
        // Moving状態のモンスターのみ衝突判定を行う
        if *state != MonsterState::Moving {
            collision_state.is_colliding = false;
            continue;
        }

        // 自分の予測位置を計算（0.1 * SIZE = 6.4ピクセル先）
        let direction_vector = movement.direction.to_vector();
        let check_distance = 0.1 * GRID_SIZE;
        let predicted_pos = transform.translation + direction_vector.extend(0.0) * check_distance;

        collision_state.is_colliding = false;

        // 他のモンスターとの衝突をチェック
        for (other_entity, other_pos, other_size, other_state) in &monsters {
            // 自分自身はスキップ
            if entity == *other_entity {
                continue;
            }

            // 相手もMoving状態でない場合はスキップ
            if *other_state != MonsterState::Moving {
                continue;
            }

            // 予測位置と相手の現在位置で矩形衝突判定
            if check_aabb_collision(predicted_pos, collision_box.size, *other_pos, *other_size) {
                collision_state.is_colliding = true;
                break;
            }
        }
    }
}

/// AABB（Axis-Aligned Bounding Box）矩形衝突判定
fn check_aabb_collision(pos1: Vec3, size1: Vec2, pos2: Vec3, size2: Vec2) -> bool {
    let half_size1 = size1 / 2.0;
    let half_size2 = size2 / 2.0;

    // x軸の重なりチェック
    let x_overlap = (pos1.x - pos2.x).abs() < (half_size1.x + half_size2.x);
    // y軸の重なりチェック
    let y_overlap = (pos1.y - pos2.y).abs() < (half_size1.y + half_size2.y);

    // 両軸で重なっていれば衝突
    x_overlap && y_overlap
}
