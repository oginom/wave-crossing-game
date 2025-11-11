use bevy::prelude::*;
use crate::core::types::GridPosition;

/// アイテムマーカーコンポーネント
#[derive(Component)]
pub struct Item;

/// アイテムの種類
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    RotationTile, // ぐるぐる床
}

/// ぐるぐる床コンポーネント
#[derive(Component)]
pub struct RotationTile {
    pub grid_pos: GridPosition,
}
