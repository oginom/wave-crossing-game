/// ゲームの技術的な定数設定
/// グローバルで不変な設定値（変更するとゲーム全体に影響）

// フィールド設定
pub const GRID_SIZE: f32 = 64.0; // 1マスのサイズ（ピクセル）
pub const FIELD_WIDTH: i32 = 10; // フィールドの幅（マス数）
pub const FIELD_HEIGHT: i32 = 10; // フィールドの高さ（マス数）

// 衝突判定
pub const COLLISION_THRESHOLD: f32 = 32.0; // 衝突判定距離（ピクセル）

// 色設定（デバッグ用）
pub const GRID_COLOR: (f32, f32, f32) = (0.3, 0.3, 0.3); // グリッド線の色
pub const MONSTER_COLOR: (f32, f32, f32) = (1.0, 0.3, 0.3); // モンスターの色
pub const ITEM_COLOR: (f32, f32, f32) = (0.2, 0.8, 0.2); // アイテムの色
