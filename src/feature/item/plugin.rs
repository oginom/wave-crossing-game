use bevy::prelude::*;
use super::placement::*;
use super::rotation_tile::*;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app
            // システムの登録
            .add_systems(Update, (
                // アイテム配置システム
                place_item_on_click,
                // ぐるぐる床の効果システム
                rotation_tile_effect_system,
            ));
    }
}
