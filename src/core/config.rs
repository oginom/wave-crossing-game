/// ゲームの定数設定

// フィールド設定
pub const GRID_SIZE: f32 = 64.0; // 1マスのサイズ（ピクセル）
pub const FIELD_WIDTH: i32 = 10; // フィールドの幅（マス数）
pub const FIELD_HEIGHT: i32 = 10; // フィールドの高さ（マス数）

// モンスター設定
pub const MONSTER_SPEED: f32 = 100.0; // モンスターの移動速度（px/秒）
pub const COLLISION_THRESHOLD: f32 = 32.0; // 衝突判定距離（ピクセル）
pub const STAGING_DURATION: f32 = 2.0; // デフォルト待機時間（秒）

// 色設定（デバッグ用）
pub const GRID_COLOR: (f32, f32, f32) = (0.3, 0.3, 0.3); // グリッド線の色
pub const MONSTER_COLOR: (f32, f32, f32) = (1.0, 0.3, 0.3); // モンスターの色
pub const ITEM_COLOR: (f32, f32, f32) = (0.2, 0.8, 0.2); // アイテムの色
