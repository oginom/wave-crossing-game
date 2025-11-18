use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::core::{config::*, types::*, level};
use crate::feature::player::PlayerGauges;
use super::components::*;

/// マウスクリックでアイテムを配置するシステム
pub fn place_item_on_click(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    // 既存のアイテムを削除するため
    existing_items: Query<(Entity, &RotationTile), With<Item>>,
    // プレイヤーゲージ
    mut gauges: ResMut<PlayerGauges>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        // マウス座標をワールド座標に変換
        if let Some(world_pos) = get_mouse_world_position(&windows, &camera_query) {
            let grid_pos = world_to_grid(world_pos, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT);

            // グリッド範囲内チェック
            if is_valid_grid_position(grid_pos, FIELD_WIDTH, FIELD_HEIGHT) {
                // 魂を消費（不足していれば設置できない）
                if !gauges.spirit.consume(level::ITEM_PLACEMENT_COST) {
                    info!(
                        "アイテム設置失敗: 魂が不足しています（必要: {}, 現在: {:.1}）",
                        level::ITEM_PLACEMENT_COST,
                        gauges.spirit.current
                    );
                    return;
                }

                info!(
                    "アイテム設置: 魂 -{} ({:.1}/{:.1})",
                    level::ITEM_PLACEMENT_COST,
                    gauges.spirit.current,
                    gauges.spirit.max
                );

                // 同じ座標に既存のアイテムがあれば削除（上書き）
                for (entity, rotation_tile) in existing_items.iter() {
                    if rotation_tile.grid_pos == grid_pos {
                        commands.entity(entity).despawn();
                    }
                }

                // アイテムをスポーン
                let world_pos = grid_to_world(grid_pos, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT);
                commands.spawn((
                    Item,
                    ItemKind::RotationTile,
                    RotationTile { grid_pos },
                    Sprite {
                        color: Color::srgb(ITEM_COLOR.0, ITEM_COLOR.1, ITEM_COLOR.2),
                        custom_size: Some(Vec2::splat(GRID_SIZE * 0.8)),
                        ..default()
                    },
                    Transform::from_translation(world_pos.extend(0.0)),
                ));
            }
        }
    }
}

/// マウスのワールド座標を取得
fn get_mouse_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = camera_query.single().ok()?;

    let cursor_position = window.cursor_position()?;

    // ビューポート座標をワールド座標に変換
    camera.viewport_to_world_2d(camera_transform, cursor_position).ok()
}
