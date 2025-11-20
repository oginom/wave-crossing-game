use bevy::prelude::*;
use rand::prelude::*;

use crate::core::types::{GridPosition, world_to_grid, grid_to_world, is_valid_grid_position};
use crate::core::config::{GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT};
use crate::feature::monster::{Monster, Movement, MonsterProperty};
use super::components::{Obstacle, ObstaclePosition, SwampEffect, WindEffect, OnSwamp};

/// 風効果用のマーカー（同じモンスターが連続で風効果を受けないようにする）
#[derive(Component, Debug)]
pub struct WindAffected {
    pub last_affected_pos: GridPosition,
}

/// 風による押し出し移動中のコンポーネント
#[derive(Component, Debug)]
pub struct WindPush {
    pub start_pos: Vec2,
    pub target_pos: Vec2,
    pub elapsed: f32,
    pub duration: f32,
}

/// 泥沼検出システム: モンスターが泥沼の上にいるかチェックしてOnSwampマーカーを付与/削除
pub fn detect_swamp_system(
    mut commands: Commands,
    swamp_query: Query<(&ObstaclePosition, &SwampEffect), (With<Obstacle>, With<SwampEffect>)>,
    monster_query: Query<(Entity, &Transform, Option<&OnSwamp>), With<Monster>>,
) {
    for (entity, transform, on_swamp) in &monster_query {
        let monster_grid_pos = world_to_grid(
            transform.translation.xy(),
            GRID_SIZE,
            FIELD_WIDTH,
            FIELD_HEIGHT,
        );

        // 泥沼の上にいるかチェック
        let mut is_on_swamp = false;
        let mut speed_multiplier = 1.0;

        for (obstacle_pos, swamp_effect) in &swamp_query {
            if obstacle_pos.grid_pos == monster_grid_pos {
                is_on_swamp = true;
                speed_multiplier = swamp_effect.speed_multiplier;
                break;
            }
        }

        // OnSwampマーカーの付与/削除
        match (is_on_swamp, on_swamp) {
            // 泥沼の上にいるが、まだマーカーがない → 追加
            (true, None) => {
                commands.entity(entity).insert(OnSwamp { speed_multiplier });
            }
            // 泥沼の外にいるが、マーカーがある → 削除
            (false, Some(_)) => {
                commands.entity(entity).remove::<OnSwamp>();
            }
            // それ以外（状態が変わっていない）→ 何もしない
            _ => {}
        }
    }
}

/// 泥沼効果適用システム: OnSwampマーカーを持つモンスターの速度を減少させる
pub fn apply_swamp_effect_system(
    mut query: Query<(&mut Movement, &MonsterProperty, &OnSwamp), With<Monster>>,
) {
    for (mut movement, property, on_swamp) in &mut query {
        // Movement.enabledがfalseの場合（WindPush中など）はスキップ
        if !movement.enabled {
            continue;
        }

        // 速度を適用
        movement.speed = property.base_speed * on_swamp.speed_multiplier;
    }
}

/// 風効果: モンスターが風の上に来たとき、ランダムな方向に1マス飛ばす（0.2秒かけて移動）
pub fn wind_effect_system(
    mut commands: Commands,
    wind_query: Query<&ObstaclePosition, (With<Obstacle>, With<WindEffect>)>,
    mut monster_query: Query<
        (Entity, &Transform, &mut Movement, Option<&WindAffected>, Option<&WindPush>),
        With<Monster>
    >,
) {
    let mut rng = thread_rng();

    for (entity, transform, mut movement, wind_affected, wind_push) in &mut monster_query {
        // 既にWindPush中の場合はスキップ
        if wind_push.is_some() {
            continue;
        }

        let monster_grid_pos = world_to_grid(
            transform.translation.xy(),
            GRID_SIZE,
            FIELD_WIDTH,
            FIELD_HEIGHT,
        );

        // 既に風効果を受けている場合、同じマスでは再度発動しない
        if let Some(affected) = wind_affected {
            if affected.last_affected_pos == monster_grid_pos {
                continue;
            }
        }

        // 風の上にいるかチェック
        for obstacle_pos in &wind_query {
            if obstacle_pos.grid_pos == monster_grid_pos {
                // ランダムな方向に1マス飛ばす
                let directions = [
                    GridPosition { x: 1, y: 0 },   // 右
                    GridPosition { x: -1, y: 0 },  // 左
                    GridPosition { x: 0, y: 1 },   // 上
                    GridPosition { x: 0, y: -1 },  // 下
                ];

                let random_direction = directions.choose(&mut rng).unwrap();
                let new_grid_pos = GridPosition {
                    x: monster_grid_pos.x + random_direction.x,
                    y: monster_grid_pos.y + random_direction.y,
                };

                // 新しい位置がフィールド範囲内かチェック
                if is_valid_grid_position(new_grid_pos, FIELD_WIDTH, FIELD_HEIGHT) {
                    let new_world_pos = grid_to_world(
                        new_grid_pos,
                        GRID_SIZE,
                        FIELD_WIDTH,
                        FIELD_HEIGHT,
                    );

                    // WindPushコンポーネントを追加して補間移動を開始
                    commands.entity(entity).insert(WindPush {
                        start_pos: transform.translation.xy(),
                        target_pos: new_world_pos,
                        elapsed: 0.0,
                        duration: crate::core::level::WIND_PUSH_DURATION,
                    });

                    // 通常の移動を停止
                    movement.enabled = false;

                    // WindAffectedマーカーを更新または追加
                    commands.entity(entity).insert(WindAffected {
                        last_affected_pos: monster_grid_pos,
                    });
                }

                break;
            }
        }
    }
}

/// 風による押し出し移動を処理するシステム
pub fn wind_push_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Movement, &mut WindPush)>,
) {
    for (entity, mut transform, mut movement, mut wind_push) in &mut query {
        wind_push.elapsed += time.delta_secs();
        let t = (wind_push.elapsed / wind_push.duration).min(1.0);

        // lerp で補間
        let new_pos = wind_push.start_pos.lerp(wind_push.target_pos, t);
        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;

        // 完了したらコンポーネント削除して通常移動を再開
        if t >= 1.0 {
            commands.entity(entity).remove::<WindPush>();
            movement.enabled = true;
        }
    }
}
