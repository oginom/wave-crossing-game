use bevy::prelude::*;
use crate::core::{config::*, types::*};
use crate::feature::monster::{Monster, MonsterProperty, Movement, CollisionBox};
use super::components::*;

/// ぐるぐる床の効果を適用するシステム
/// モンスターの当たり判定矩形が床のグリッドに触れている間だけ90度右に移動し、離れたら本来の方向に戻る
pub fn rotation_tile_effect_system(
    tile_query: Query<&RotationTile, With<Item>>,
    mut monster_query: Query<
        (&Transform, &CollisionBox, &MonsterProperty, &mut Movement),
        With<Monster>,
    >,
) {
    for (monster_transform, collision_box, property, mut movement) in monster_query.iter_mut() {
        let monster_pos = monster_transform.translation.xy();
        let half_size = collision_box.size / 2.0;

        // モンスターの当たり判定矩形が触れているグリッドを判定
        let mut on_rotation_tile = false;

        for tile in tile_query.iter() {
            // タイルのグリッド座標をワールド座標に変換
            let tile_world_pos = grid_to_world(tile.grid_pos, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT);
            let tile_half_size = GRID_SIZE / 2.0;

            // AABB衝突判定：モンスターの矩形とタイルのグリッド矩形が重なっているか
            let x_overlap = (monster_pos.x - tile_world_pos.x).abs() < (half_size.x + tile_half_size);
            let y_overlap = (monster_pos.y - tile_world_pos.y).abs() < (half_size.y + tile_half_size);

            if x_overlap && y_overlap {
                on_rotation_tile = true;
                break;
            }
        }

        // ぐるぐる床に触れている場合は90度右回転、それ以外は本来の方向
        if on_rotation_tile {
            // 本来の方向から90度右回転
            movement.direction = property.base_direction.rotate_clockwise();
        } else {
            // 本来の方向に戻す
            movement.direction = property.base_direction;
        }

        // speedは常に本来の速度を使用
        movement.speed = property.base_speed;
    }
}
