use bevy::{asset::AssetMetaCheck, prelude::*};
use wave_crossing_game::AppPlugin;

fn main() {
    // Set up better panic messages for WASM
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(
            DefaultPlugins.set(AssetPlugin {
                // WASM builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics on web builds on itch or with SPA dev-servers.
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
        )
        .add_plugins(AppPlugin)
        .run();
}
