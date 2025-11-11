use bevy::prelude::*;
use super::grid::*;

/// ワールド（フィールド）機能を提供するプラグイン
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_camera)
            .add_systems(Update, draw_grid_system);
    }
}

/// カメラの初期設定
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}
