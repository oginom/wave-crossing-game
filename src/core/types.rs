use bevy::prelude::*;

/// グリッド座標
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// モンスターの進行方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// 方向を単位ベクトルに変換
    pub fn to_vector(self) -> Vec2 {
        match self {
            Direction::Up => Vec2::new(0.0, 1.0),
            Direction::Down => Vec2::new(0.0, -1.0),
            Direction::Left => Vec2::new(-1.0, 0.0),
            Direction::Right => Vec2::new(1.0, 0.0),
        }
    }

    /// 方向を90度右回転
    pub fn rotate_clockwise(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }
}

/// グリッド座標をワールド座標に変換
pub fn grid_to_world(grid_pos: GridPosition, grid_size: f32, field_width: i32, field_height: i32) -> Vec2 {
    Vec2::new(
        (grid_pos.x as f32 - field_width as f32 / 2.0 + 0.5) * grid_size,
        (grid_pos.y as f32 - field_height as f32 / 2.0 + 0.5) * grid_size,
    )
}

/// ワールド座標をグリッド座標に変換
pub fn world_to_grid(world_pos: Vec2, grid_size: f32, field_width: i32, field_height: i32) -> GridPosition {
    GridPosition::new(
        (world_pos.x / grid_size + field_width as f32 / 2.0).floor() as i32,
        (world_pos.y / grid_size + field_height as f32 / 2.0).floor() as i32,
    )
}

/// グリッド座標がフィールド範囲内かチェック
pub fn is_valid_grid_position(grid_pos: GridPosition, field_width: i32, field_height: i32) -> bool {
    grid_pos.x >= 0 && grid_pos.x < field_width && grid_pos.y >= 0 && grid_pos.y < field_height
}
