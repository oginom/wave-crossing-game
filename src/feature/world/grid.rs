use bevy::prelude::*;
use crate::core::{GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT, GRID_COLOR};

/// グリッドを描画するシステム
pub fn draw_grid_system(mut gizmos: Gizmos) {
    let grid_color = Color::srgb(GRID_COLOR.0, GRID_COLOR.1, GRID_COLOR.2);

    let width_pixels = FIELD_WIDTH as f32 * GRID_SIZE;
    let height_pixels = FIELD_HEIGHT as f32 * GRID_SIZE;

    // 縦線を描画
    for i in 0..=FIELD_WIDTH {
        let x = (i as f32 - FIELD_WIDTH as f32 / 2.0) * GRID_SIZE;
        gizmos.line_2d(
            Vec2::new(x, -height_pixels / 2.0),
            Vec2::new(x, height_pixels / 2.0),
            grid_color,
        );
    }

    // 横線を描画
    for i in 0..=FIELD_HEIGHT {
        let y = (i as f32 - FIELD_HEIGHT as f32 / 2.0) * GRID_SIZE;
        gizmos.line_2d(
            Vec2::new(-width_pixels / 2.0, y),
            Vec2::new(width_pixels / 2.0, y),
            grid_color,
        );
    }
}
