# フィールド障害物実装計画

## 目的

フィールドに障害物を配置し、モンスターの移動に影響を与えるゲーム要素を追加する。障害物はステージ定義ファイル（RON）で管理し、データ駆動型の設計を実現する。

## 実装対象機能

1. **泥沼（Swamp）**
   - モンスターが上を通過すると移動速度が半分になる
   - 茶色の四角形で表示

2. **風（Wind）**
   - モンスターが真上に来たときランダムな方向に1マス分飛ばされる
   - 水色の四角形で表示

3. **ステージ定義への障害物設定の追加**
   - RONファイルで障害物の種類と配置を定義
   - カスタムアセットローダーで読み込み

---

## 実装ステップ

### Step 1: 障害物の基本構造準備

#### 1.1 障害物コンポーネントの定義

**ファイル**: `src/feature/obstacle/mod.rs`, `src/feature/obstacle/components.rs`

新しい `obstacle` フィーチャーモジュールを作成し、基本的なコンポーネントを定義する。

```rust
// components.rs

use bevy::prelude::*;
use crate::core::types::GridPosition;
use serde::{Deserialize, Serialize};

/// 障害物の基本コンポーネント
#[derive(Component)]
pub struct Obstacle;

/// 障害物の種類
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObstacleKind {
    Swamp,  // 泥沼
    Wind,   // 風
}

/// 障害物の配置情報
#[derive(Component, Debug, Clone, Copy)]
pub struct ObstaclePosition {
    pub grid_pos: GridPosition,
}

/// 泥沼の効果コンポーネント
#[derive(Component, Debug, Clone, Copy)]
pub struct SwampEffect {
    /// 速度倍率（0.5 = 半分）
    pub speed_multiplier: f32,
}

impl Default for SwampEffect {
    fn default() -> Self {
        Self {
            speed_multiplier: 0.5,
        }
    }
}

/// 風の効果コンポーネント
#[derive(Component, Debug, Clone, Copy)]
pub struct WindEffect;
```

**関連ファイル**:
- `src/feature/obstacle/mod.rs` を作成
- `src/feature/mod.rs` に `pub mod obstacle;` を追加

---

#### 1.2 障害物定義の追加

**ファイル**: `src/feature/obstacle/definitions.rs`

障害物の定義をデータで管理する構造を作成。

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::types::GridPosition;
use super::components::ObstacleKind;

/// 障害物のスポーン定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObstacleDefinition {
    pub kind: ObstacleKind,
    pub grid_pos: GridPosition,
}

/// 障害物の視覚設定
#[derive(Debug, Clone)]
pub struct ObstacleVisualConfig {
    pub kind: ObstacleKind,
    pub color: (f32, f32, f32),
    pub size: f32,  // グリッドサイズに対する倍率（0.8 = 80%）
}

impl ObstacleVisualConfig {
    pub fn get_config(kind: ObstacleKind) -> Self {
        match kind {
            ObstacleKind::Swamp => Self {
                kind,
                color: (0.4, 0.3, 0.2),  // 茶色
                size: 0.9,
            },
            ObstacleKind::Wind => Self {
                kind,
                color: (0.6, 0.9, 1.0),  // 水色
                size: 0.8,
            },
        }
    }
}
```

---

### Step 2: 障害物プラグインの作成

#### 2.1 ObstaclePlugin の骨格

**ファイル**: `src/feature/obstacle/plugin.rs`

```rust
use bevy::prelude::*;
use crate::GameState;

use super::{
    spawn::spawn_obstacles_from_stage,
    effects::{swamp_effect_system, wind_effect_system},
};

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app
            // Startup: 障害物の配置（一時的にハードコード）
            .add_systems(Startup, spawn_obstacles_hardcoded)
            // Update: 効果の適用
            .add_systems(
                Update,
                (
                    swamp_effect_system,
                    wind_effect_system,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame))
            );
    }
}

/// プロトタイプ用：ハードコードで障害物を配置
fn spawn_obstacles_hardcoded(
    mut commands: Commands,
) {
    use crate::core::types::GridPosition;
    use super::components::*;
    use super::definitions::ObstacleVisualConfig;
    use crate::core::config::GRID_SIZE;

    // 泥沼を2箇所配置
    let swamp_positions = vec![
        GridPosition { x: 3, y: 3 },
        GridPosition { x: 6, y: 7 },
    ];

    for grid_pos in swamp_positions {
        let config = ObstacleVisualConfig::get_config(ObstacleKind::Swamp);
        let world_pos = crate::core::types::grid_to_world(
            grid_pos,
            GRID_SIZE,
            crate::core::config::FIELD_WIDTH,
            crate::core::config::FIELD_HEIGHT,
        );

        commands.spawn((
            Obstacle,
            ObstacleKind::Swamp,
            ObstaclePosition { grid_pos },
            SwampEffect::default(),
            Sprite {
                color: Color::srgb(config.color.0, config.color.1, config.color.2),
                custom_size: Some(Vec2::splat(GRID_SIZE * config.size)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(0.0)),
        ));
    }

    // 風を2箇所配置
    let wind_positions = vec![
        GridPosition { x: 5, y: 2 },
        GridPosition { x: 8, y: 8 },
    ];

    for grid_pos in wind_positions {
        let config = ObstacleVisualConfig::get_config(ObstacleKind::Wind);
        let world_pos = crate::core::types::grid_to_world(
            grid_pos,
            GRID_SIZE,
            crate::core::config::FIELD_WIDTH,
            crate::core::config::FIELD_HEIGHT,
        );

        commands.spawn((
            Obstacle,
            ObstacleKind::Wind,
            ObstaclePosition { grid_pos },
            WindEffect,
            Sprite {
                color: Color::srgb(config.color.0, config.color.1, config.color.2),
                custom_size: Some(Vec2::splat(GRID_SIZE * config.size)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(0.0)),
        ));
    }
}
```

**登録**: `src/lib.rs` の `AppPlugin` に `ObstaclePlugin` を追加

```rust
use feature::obstacle::ObstaclePlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                StageAssetPlugin,
                WorldPlugin,
                MonsterPlugin,
                ItemPlugin,
                ObstaclePlugin,  // 追加
                PlayerPlugin,
            ));
    }
}
```

---

### Step 3: 障害物の効果実装

#### 3.1 泥沼効果システム

**ファイル**: `src/feature/obstacle/effects.rs`

泥沼の上にいるモンスターの速度を減少させる。

```rust
use bevy::prelude::*;
use crate::core::types::{GridPosition, world_to_grid};
use crate::core::config::{GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT};
use crate::feature::monster::components::{Monster, Movement, MonsterProperty};
use super::components::{Obstacle, ObstacleKind, ObstaclePosition, SwampEffect};

/// 泥沼効果: モンスターが泥沼の上にいる場合、速度を減少させる
pub fn swamp_effect_system(
    swamp_query: Query<(&ObstaclePosition, &SwampEffect), (With<Obstacle>, With<SwampEffect>)>,
    mut monster_query: Query<(&Transform, &mut Movement, &MonsterProperty), With<Monster>>,
) {
    for (transform, mut movement, property) in &mut monster_query {
        let monster_grid_pos = world_to_grid(
            transform.translation.xy(),
            GRID_SIZE,
            FIELD_WIDTH,
            FIELD_HEIGHT,
        );

        // デフォルトでは元の速度に戻す
        let mut speed_multiplier = 1.0;

        // 泥沼の上にいるかチェック
        for (obstacle_pos, swamp_effect) in &swamp_query {
            if obstacle_pos.grid_pos == monster_grid_pos {
                speed_multiplier = swamp_effect.speed_multiplier;
                break;
            }
        }

        // 速度を適用（base_speedから計算）
        movement.speed = property.base_speed * speed_multiplier;
    }
}
```

**注意点**:
- 衝突検出システムで `movement.speed = 0` に設定される可能性があるため、システムの実行順序に注意
- `swamp_effect_system` は `collision_detection_system` の**前**に実行すべき

---

#### 3.2 風効果システム

**ファイル**: `src/feature/obstacle/effects.rs` に追加

風の上に来たモンスターをランダムな方向に1マス分移動させる。

```rust
use rand::prelude::*;

/// 風効果用のマーカー（同じモンスターが連続で風効果を受けないようにする）
#[derive(Component, Debug)]
pub struct WindAffected {
    pub last_affected_pos: GridPosition,
}

/// 風効果: モンスターが風の上に来たとき、ランダムな方向に1マス飛ばす
pub fn wind_effect_system(
    mut commands: Commands,
    wind_query: Query<&ObstaclePosition, (With<Obstacle>, With<WindEffect>)>,
    mut monster_query: Query<(Entity, &mut Transform, Option<&WindAffected>), With<Monster>>,
) {
    let mut rng = thread_rng();

    for (entity, mut transform, wind_affected) in &mut monster_query {
        let monster_grid_pos = world_to_grid(
            transform.translation.xy(),
            GRID_SIZE,
            FIELD_WIDTH,
            FIELD_HEIGHT,
        );

        // 既に風効果を受けている場合、同じマスでは再度発動しない
        if let Some(affected) = wind_affected {
            if affected.last_affected_pos == monster_grid_pos {
                continue;
            }
        }

        // 風の上にいるかチェック
        for obstacle_pos in &wind_query {
            if obstacle_pos.grid_pos == monster_grid_pos {
                // ランダムな方向に1マス飛ばす
                let directions = [
                    GridPosition { x: 1, y: 0 },   // 右
                    GridPosition { x: -1, y: 0 },  // 左
                    GridPosition { x: 0, y: 1 },   // 上
                    GridPosition { x: 0, y: -1 },  // 下
                ];

                let random_direction = directions.choose(&mut rng).unwrap();
                let new_grid_pos = GridPosition {
                    x: monster_grid_pos.x + random_direction.x,
                    y: monster_grid_pos.y + random_direction.y,
                };

                // 新しい位置がフィールド範囲内かチェック
                if crate::core::types::is_valid_grid_position(new_grid_pos, FIELD_WIDTH, FIELD_HEIGHT) {
                    let new_world_pos = crate::core::types::grid_to_world(
                        new_grid_pos,
                        GRID_SIZE,
                        FIELD_WIDTH,
                        FIELD_HEIGHT,
                    );

                    transform.translation.x = new_world_pos.x;
                    transform.translation.y = new_world_pos.y;
                }

                // WindAffectedマーカーを更新または追加
                commands.entity(entity).insert(WindAffected {
                    last_affected_pos: monster_grid_pos,
                });

                break;
            }
        }
    }
}
```

**WindAffectedコンポーネントの追加**:
- 同じマスで連続して風効果が発動しないようにするためのマーカー
- モンスターが別のマスに移動したら、再度風効果を受けられる

**Cargo.tomlへのrand追加**:
```toml
[dependencies]
rand = "0.8"
```

---

#### 3.3 システム実行順序の調整

**ファイル**: `src/feature/obstacle/plugin.rs` を更新

```rust
.add_systems(
    Update,
    (
        swamp_effect_system,
        wind_effect_system,
    )
        .chain()
        .run_if(in_state(GameState::InGame))
)
```

**MonsterPluginの実行順序も調整**（`src/feature/monster/plugin.rs`）:

```rust
.add_systems(
    Update,
    (
        spawn_monsters_system,
        staging_timer_system,
        my_pace_system,
        // 障害物効果は衝突検出の前に適用
        collision_detection_system,
        monster_movement_system,
        update_wait_meter_system,
        update_monster_color_system,
        despawn_expired_monsters_system,
        despawn_reached_monsters,
    )
        .chain()
        .run_if(in_state(GameState::InGame))
)
```

**注意**: ObstaclePluginのシステムがMonsterPluginより**後**に登録されている場合、`.before()` / `.after()` を使って明示的に順序を指定する必要がある。

---

### Step 4: ステージ定義への障害物追加

#### 4.1 StageLevelAsset への obstacles フィールド追加

**ファイル**: `src/core/stage_asset.rs`

```rust
use crate::feature::obstacle::definitions::ObstacleDefinition;

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct StageLevelAsset {
    pub stage: u32,
    pub level: u32,
    pub waves: Vec<WaveDefinition>,
    pub obstacles: Vec<ObstacleDefinition>,  // 追加
}
```

---

#### 4.2 障害物スポーンシステムの作成

**ファイル**: `src/feature/obstacle/spawn.rs`

ステージアセットから障害物を読み込んでスポーンする。

```rust
use bevy::prelude::*;
use crate::core::stage_asset::StageLevelAsset;
use crate::core::config::{GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT};
use super::components::*;
use super::definitions::ObstacleVisualConfig;

/// ステージアセットから障害物をスポーン
pub fn spawn_obstacles_from_stage(
    mut commands: Commands,
    stage_assets: Res<Assets<StageLevelAsset>>,
    stage_handle: Res<Handle<StageLevelAsset>>,  // Startup時にロードしたハンドル
) {
    let Some(stage_asset) = stage_assets.get(&stage_handle) else {
        return;
    };

    for obstacle_def in &stage_asset.obstacles {
        let config = ObstacleVisualConfig::get_config(obstacle_def.kind);
        let world_pos = crate::core::types::grid_to_world(
            obstacle_def.grid_pos,
            GRID_SIZE,
            FIELD_WIDTH,
            FIELD_HEIGHT,
        );

        let mut entity_commands = commands.spawn((
            Obstacle,
            obstacle_def.kind,
            ObstaclePosition { grid_pos: obstacle_def.grid_pos },
            Sprite {
                color: Color::srgb(config.color.0, config.color.1, config.color.2),
                custom_size: Some(Vec2::splat(GRID_SIZE * config.size)),
                ..default()
            },
            Transform::from_translation(world_pos.extend(0.0)),
        ));

        // 種類に応じた効果コンポーネントを追加
        match obstacle_def.kind {
            ObstacleKind::Swamp => {
                entity_commands.insert(SwampEffect::default());
            }
            ObstacleKind::Wind => {
                entity_commands.insert(WindEffect);
            }
        }
    }
}
```

**plugin.rsの更新**:
```rust
.add_systems(Startup, spawn_obstacles_from_stage)
```

ハードコード版の `spawn_obstacles_hardcoded` は削除またはコメントアウト。

---

#### 4.3 RONファイルの更新

**ファイル**: `assets/stages/stage1_level1.ron`

```ron
(
    stage: 1,
    level: 1,
    waves: [
        // ... 既存のWave定義 ...
    ],
    obstacles: [
        // 泥沼
        (kind: Swamp, grid_pos: (x: 3, y: 3)),
        (kind: Swamp, grid_pos: (x: 6, y: 7)),
        (kind: Swamp, grid_pos: (x: 4, y: 8)),
        // 風
        (kind: Wind, grid_pos: (x: 5, y: 2)),
        (kind: Wind, grid_pos: (x: 8, y: 8)),
        (kind: Wind, grid_pos: (x: 2, y: 5)),
    ],
)
```

**注意**: `GridPosition` の serde デシリアライズが正しく動作するか確認。必要に応じて `#[derive(Serialize, Deserialize)]` を `core/types.rs` の `GridPosition` に追加。

---

### Step 5: 設定ファイルへの定数追加

#### 5.1 レベルデザイン定数の追加

**ファイル**: `src/core/level.rs`

```rust
// ========================================
// 障害物関連
// ========================================

/// 泥沼の速度倍率
pub const SWAMP_SPEED_MULTIPLIER: f32 = 0.5;
```

`SwampEffect::default()` をこの定数を使うように変更:

```rust
impl Default for SwampEffect {
    fn default() -> Self {
        Self {
            speed_multiplier: crate::core::level::SWAMP_SPEED_MULTIPLIER,
        }
    }
}
```

---

### Step 6: 統合とテスト

#### 6.1 動作確認項目

1. **障害物の表示**
   - ステージ開始時に障害物が正しい位置に表示される
   - 泥沼は茶色、風は水色の四角形

2. **泥沼効果**
   - モンスターが泥沼の上を通過するとき、移動速度が半分になる
   - 泥沼を出ると元の速度に戻る

3. **風効果**
   - モンスターが風の上に来ると、ランダムな方向に1マス飛ばされる
   - 同じ風マスで連続発動しない
   - フィールド外に飛ばされない

4. **ステージ定義からの読み込み**
   - RONファイルの `obstacles` フィールドから正しく読み込まれる
   - 障害物の種類と位置が定義通り

5. **他システムとの相互作用**
   - 泥沼で減速中のモンスター同士の衝突判定が正しく動作
   - 風で飛ばされた直後の位置で衝突判定が正しく動作

---

## ファイル構成まとめ

```text
src/
├── core/
│   ├── level.rs              # SWAMP_SPEED_MULTIPLIER 追加
│   ├── stage_asset.rs        # StageLevelAsset に obstacles 追加
│   └── types.rs              # GridPosition に Serialize/Deserialize 追加
└── feature/
    └── obstacle/             # 新規作成
        ├── mod.rs
        ├── plugin.rs         # ObstaclePlugin
        ├── components.rs     # Obstacle, ObstacleKind, etc.
        ├── definitions.rs    # ObstacleDefinition, VisualConfig
        ├── spawn.rs          # spawn_obstacles_from_stage
        └── effects.rs        # swamp_effect_system, wind_effect_system

assets/
└── stages/
    └── stage1_level1.ron     # obstacles フィールド追加

Cargo.toml                    # rand クレート追加
```

---

## 実装順序の推奨

1. **Step 1（基本構造）** → コンポーネント定義とモジュール作成
2. **Step 2（プラグイン骨格）** → ハードコードで障害物配置、視覚確認
3. **Step 3（効果実装）** → 泥沼と風の効果システム実装
4. **Step 4（データ駆動化）** → RONファイルからの読み込み対応
5. **Step 5（定数化）** → マジックナンバーを設定ファイルに移動
6. **Step 6（統合テスト）** → 全体の動作確認

---

## 今後の拡張予定

- 障害物の追加種類（ワープゲート、加速床、ジャンプ台など）
- 障害物のアニメーション効果（パーティクル、回転など）
- 障害物のテクスチャ（現在は単色の四角形）
- 障害物の削除・配置機能（プレイヤーがアイテムとして使用）
- 障害物ごとの視覚・音響エフェクト

---

## 備考

- プロトタイプ段階では視覚表現は単色の四角形で十分
- 風効果のランダム性はテストで再現性が必要な場合、シード設定を検討
- 障害物とアイテムの違い:
  - **障害物**: ステージに固定配置、プレイヤーは操作不可
  - **アイテム**: プレイヤーが配置可能（ぐるぐる床など）
- システム実行順序は慎重に設定（特に速度変更と衝突判定の関係）

---

## 技術的注意点

### GridPositionのシリアライズ対応

`core/types.rs` の `GridPosition` に以下を追加:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}
```

### 風効果の連続発動防止

`WindAffected` コンポーネントを使用して、同じマスでの連続発動を防ぐ。
モンスターが別のマスに移動したら、新しい風マスで再度効果を受けられるようにする。

### 泥沼とMyPace挙動の相互作用

MyPaceで停止中（`Movement.enabled = false`）のモンスターには泥沼効果は適用されない
（停止中は速度0なので、倍率をかけても変わらない）。

### 風効果と衝突判定

風で飛ばされた直後の位置で他のモンスターと重なる可能性がある。
この場合、次フレームの衝突判定で停止処理が発動する。
