use bevy::prelude::*;
use crate::GameState;
use super::gauges::*;
use crate::feature::ui::*;

/// プレイヤー機能を提供するプラグイン
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerGauges>()
            .add_systems(Startup, setup_gauges_ui_system)
            .add_systems(
                Update,
                (
                    update_gauges_on_monster_event_system,
                    check_game_over_system,
                    update_spirit_gauge_ui_system,
                    update_void_gauge_ui_system,
                )
                    .run_if(in_state(GameState::InGame))
            );
    }
}
