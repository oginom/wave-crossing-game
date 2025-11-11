# 最小シーン実装計画

## 目的

最低限の動きを確認できるプロトタイプシーンを実装する。このシーンでは、モンスターがマス目状のフィールドを通過し、衝突検知と回避アイテムの動作を確認できる。

## 実装対象機能

1. マス目状のフィールド（10x10程度のグリッド）
2. 上下左右からフィールドを通過するモンスター（10匹程度）
3. モンスターの待機・移動・消滅の仕組み
4. モンスター間の衝突検知と停止処理
5. アイテム「ぐるぐる床」による方向転換機能
6. プロトタイプ用の無制限アイテム設置システム

---

## 実装ステップ

### Step 1: 基盤構造の準備

#### 1.1 共通型定義の作成

**ファイル**: `src/core/mod.rs`, `src/core/types.rs`

- グリッド座標系の定義（`GridPosition`, `GridSize`）
- 4方向の列挙型（`Direction`: Up/Down/Left/Right）
- 基本的な変換関数（グリッド座標 ↔ ワールド座標）

```rust
// 実装内容例
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub const GRID_SIZE: f32 = 64.0; // 1マスのサイズ（ピクセル）
pub const FIELD_WIDTH: i32 = 10;
pub const FIELD_HEIGHT: i32 = 10;
```

**関連ファイル**:
- `src/core/mod.rs` を作成し、サブモジュールをエクスポート
- `src/lib.rs` に `pub mod core;` を追加

---

#### 1.2 定数設定の定義

**ファイル**: `src/core/config.rs`

- フィールドサイズ
- モンスターの基本速度
- 衝突判定の閾値
- 色設定（デバッグ用）

```rust
// 実装内容例
pub const MONSTER_SPEED: f32 = 100.0; // px/秒
pub const COLLISION_THRESHOLD: f32 = 32.0; // 衝突判定距離
pub const STAGING_DURATION: f32 = 2.0; // 待機時間（秒）
```

---

### Step 2: フィールド（World）機能の実装

#### 2.1 World Plugin の作成

**ファイル**: `src/feature/world/mod.rs`, `src/feature/world/plugin.rs`

- フィールド描画システムの登録
- カメラ初期設定（main.rsから移動）

#### 2.2 グリッドフィールドの描画

**ファイル**: `src/feature/world/grid.rs`

- 10x10 のマス目を線で描画
- Gizmos を使用してシンプルに実装
- グリッド原点を画面中央に配置

```rust
// システム例
fn draw_grid_system(mut gizmos: Gizmos) {
    // 縦線と横線を描画
    for i in 0..=FIELD_WIDTH {
        let x = (i as f32 - FIELD_WIDTH as f32 / 2.0) * GRID_SIZE;
        gizmos.line_2d(
            Vec2::new(x, -FIELD_HEIGHT as f32 / 2.0 * GRID_SIZE),
            Vec2::new(x, FIELD_HEIGHT as f32 / 2.0 * GRID_SIZE),
            Color::srgb(0.3, 0.3, 0.3),
        );
    }
    // 同様に横線も描画
}
```

**登録**: `lib.rs` の `AppPlugin` に `WorldPlugin` を追加

---

### Step 3: モンスター（Monster）機能の実装

#### 3.1 Monster Plugin の骨格作成

**ファイル**:
- `src/feature/monster/mod.rs`
- `src/feature/monster/plugin.rs`
- `src/feature/monster/components.rs`

**コンポーネント定義**:

```rust
// components.rs
#[derive(Component)]
pub struct Monster;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterState {
    Staging,   // 画面端で待機中
    Moving,    // 移動中
    Reached,   // 到達（消滅待ち）
}

#[derive(Component)]
pub struct Movement {
    pub direction: Direction,
    pub speed: f32,
}

#[derive(Component)]
pub struct StagingTimer {
    pub remaining: f32,
}

#[derive(Component)]
pub struct CollisionBox {
    pub size: Vec2,
}
```

---

#### 3.2 モンスターのスポーンシステム

**ファイル**: `src/feature/monster/spawn.rs`

- Wave定義の簡易版（ハードコード）
- 10匹のモンスターを上下左右からランダムにスポーン
- 各モンスターに待機時間を設定（0秒〜5秒の範囲でランダム）

```rust
// スポーン定義例
pub struct SpawnDefinition {
    pub direction: Direction,
    pub delay: f32,
}

// Startupシステムで10匹分の定義を作成
fn setup_monster_spawns(mut commands: Commands) {
    let spawns = vec![
        SpawnDefinition { direction: Direction::Right, delay: 0.0 },
        SpawnDefinition { direction: Direction::Left, delay: 1.0 },
        // ... 計10個
    ];
    commands.insert_resource(MonsterSpawnQueue { spawns, index: 0 });
}
```

---

#### 3.3 待機システム（Staging）

**ファイル**: `src/feature/monster/staging.rs`

- `MonsterState::Staging` のモンスターのタイマーを減らす
- タイマーが0になったら `MonsterState::Moving` に遷移
- 移動用コンポーネント（`Movement`）を追加

```rust
fn staging_timer_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut StagingTimer, &mut MonsterState), With<Monster>>,
) {
    for (entity, mut timer, mut state) in &mut query {
        if *state == MonsterState::Staging {
            timer.remaining -= time.delta_seconds();
            if timer.remaining <= 0.0 {
                *state = MonsterState::Moving;
                // Movementコンポーネントを追加（spawn時の方向を使用）
            }
        }
    }
}
```

---

#### 3.4 移動システム

**ファイル**: `src/feature/monster/movement.rs`

- `MonsterState::Moving` のモンスターを `Movement` の方向に移動
- フィールド外に出たら `MonsterState::Reached` に遷移

```rust
fn monster_movement_system(
    time: Res<Time>,
    mut query: Query<(&Movement, &mut Transform, &mut MonsterState), With<Monster>>,
) {
    for (movement, mut transform, mut state) in &mut query {
        if *state == MonsterState::Moving {
            let velocity = get_direction_vector(movement.direction) * movement.speed;
            transform.translation += velocity.extend(0.0) * time.delta_seconds();

            // フィールド外判定
            if is_out_of_bounds(transform.translation.xy()) {
                *state = MonsterState::Reached;
            }
        }
    }
}
```

---

#### 3.5 消滅システム

**ファイル**: `src/feature/monster/despawn.rs`

- `MonsterState::Reached` のモンスターを削除

```rust
fn despawn_reached_monsters(
    mut commands: Commands,
    query: Query<(Entity, &MonsterState), With<Monster>>,
) {
    for (entity, state) in &query {
        if *state == MonsterState::Reached {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

---

### Step 4: 衝突検知システムの実装

#### 4.1 衝突検知コンポーネント

**ファイル**: `src/feature/monster/collision.rs`

**実装内容**:
- 前方にいる他のモンスターを検知
- 距離が閾値以下なら「衝突中」フラグを立てる
- 衝突中は移動を停止

```rust
#[derive(Component, Default)]
pub struct CollisionState {
    pub is_colliding: bool,
}

fn collision_detection_system(
    mut query: Query<(Entity, &Transform, &Movement, &CollisionBox, &mut CollisionState), With<Monster>>,
) {
    let monsters: Vec<_> = query.iter().map(|(e, t, m, c, _)| (e, t.translation, m.direction, c.size)).collect();

    for (entity, mut transform, movement, collision_box, mut collision_state) in &mut query {
        collision_state.is_colliding = false;

        // 前方にいる他のモンスターをチェック
        for (other_entity, other_pos, _, other_size) in &monsters {
            if entity == *other_entity {
                continue;
            }

            // 進行方向にいるかチェック
            if is_in_front_of(transform.translation.xy(), movement.direction, *other_pos) {
                let distance = transform.translation.xy().distance(*other_pos);
                if distance < COLLISION_THRESHOLD {
                    collision_state.is_colliding = true;
                    break;
                }
            }
        }
    }
}
```

---

#### 4.2 衝突時の停止処理

**ファイル**: `src/feature/monster/movement.rs` に追加

- `movement_system` を修正し、`CollisionState::is_colliding` が true なら移動しない

```rust
fn monster_movement_system(
    time: Res<Time>,
    mut query: Query<(&Movement, &mut Transform, &mut MonsterState, &CollisionState), With<Monster>>,
) {
    for (movement, mut transform, mut state, collision) in &mut query {
        if *state == MonsterState::Moving && !collision.is_colliding {
            // 移動処理（前述と同じ）
        }
    }
}
```

---

### Step 5: アイテム「ぐるぐる床」の実装

#### 5.1 Item Plugin の作成

**ファイル**:
- `src/feature/item/mod.rs`
- `src/feature/item/plugin.rs`
- `src/feature/item/components.rs`

**コンポーネント**:

```rust
#[derive(Component)]
pub struct Item;

#[derive(Component)]
pub enum ItemKind {
    RotationTile, // ぐるぐる床
}

#[derive(Component)]
pub struct RotationTile {
    pub grid_pos: GridPosition,
}
```

---

#### 5.2 アイテム設置システム（プロトタイプ版）

**ファイル**: `src/feature/item/placement.rs`

- マウスクリックでグリッド座標を取得
- クリックされたマスに「ぐるぐる床」を配置
- 無制限に設置可能（上書きも可能）

```rust
fn place_item_on_click(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        // マウス座標をワールド座標に変換
        let world_pos = get_mouse_world_position(windows, camera_query);
        let grid_pos = world_to_grid(world_pos);

        // グリッド範囲内チェック
        if is_valid_grid_position(grid_pos) {
            // アイテムをスポーン
            commands.spawn((
                Item,
                ItemKind::RotationTile,
                RotationTile { grid_pos },
                Sprite {
                    color: Color::srgb(0.2, 0.8, 0.2),
                    custom_size: Some(Vec2::splat(GRID_SIZE * 0.8)),
                    ..default()
                },
                Transform::from_translation(grid_to_world(grid_pos).extend(0.0)),
            ));
        }
    }
}
```

---

#### 5.3 ぐるぐる床の効果適用

**ファイル**: `src/feature/item/rotation_tile.rs`

- モンスターがぐるぐる床の上にいるか判定
- 上にいる場合、進行方向を90度回転

```rust
fn rotation_tile_effect_system(
    tile_query: Query<(&RotationTile, &Transform), With<Item>>,
    mut monster_query: Query<(&Transform, &mut Movement), With<Monster>>,
) {
    for (monster_transform, mut movement) in &mut monster_query {
        let monster_grid_pos = world_to_grid(monster_transform.translation.xy());

        for (tile, _) in &tile_query {
            if tile.grid_pos == monster_grid_pos {
                // 進行方向を90度回転（右回り）
                movement.direction = rotate_direction_90(movement.direction);
            }
        }
    }
}

fn rotate_direction_90(dir: Direction) -> Direction {
    match dir {
        Direction::Up => Direction::Right,
        Direction::Right => Direction::Down,
        Direction::Down => Direction::Left,
        Direction::Left => Direction::Up,
    }
}
```

**注意**: 毎フレーム回転し続けないよう、「このフレームで回転済み」フラグの管理が必要
→ 初期実装では簡易的に「タイルの中心を通過した瞬間」のみ回転する仕組みを検討

---

### Step 6: 統合とテスト

#### 6.1 全プラグインの登録

**ファイル**: `src/lib.rs`

```rust
use feature::{
    player::PlayerPlugin,
    world::WorldPlugin,
    monster::MonsterPlugin,
    item::ItemPlugin,
};

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                WorldPlugin,
                MonsterPlugin,
                ItemPlugin,
                PlayerPlugin, // 既存
            ));
    }
}
```

---

#### 6.2 動作確認項目

1. **フィールド表示**
   - 10x10 のグリッドが正しく表示される
   - グリッド原点が画面中央

2. **モンスターのスポーン**
   - 10匹のモンスターが画面端に出現
   - それぞれ異なる待機時間で待機

3. **モンスターの移動**
   - 待機時間経過後、設定された方向に移動
   - フィールドを通過して反対側に到達
   - 到達後に消滅

4. **衝突検知**
   - 正面衝突時に両方のモンスターが停止
   - 直角衝突時にどちらか一方が先に進む

5. **ぐるぐる床**
   - マウスクリックでグリッド上に配置可能
   - モンスターが踏むと90度方向転換
   - 正面衝突を回避できる

---

## ファイル構成まとめ

```text
src/
├─ main.rs                 # エントリーポイント
├─ lib.rs                  # AppPlugin（全体統合）
├─ core/
│   ├─ mod.rs
│   ├─ types.rs            # GridPosition, Direction など
│   └─ config.rs           # 定数設定
├─ feature/
│   ├─ mod.rs
│   ├─ player/             # 既存（そのまま）
│   ├─ world/
│   │   ├─ mod.rs
│   │   ├─ plugin.rs
│   │   └─ grid.rs         # グリッド描画
│   ├─ monster/
│   │   ├─ mod.rs
│   │   ├─ plugin.rs
│   │   ├─ components.rs
│   │   ├─ spawn.rs        # スポーン管理
│   │   ├─ staging.rs      # 待機システム
│   │   ├─ movement.rs     # 移動＋停止
│   │   ├─ collision.rs    # 衝突検知
│   │   └─ despawn.rs      # 消滅処理
│   └─ item/
│       ├─ mod.rs
│       ├─ plugin.rs
│       ├─ components.rs
│       ├─ placement.rs    # マウスクリック設置
│       └─ rotation_tile.rs # ぐるぐる床の効果
```

---

## 実装順序の推奨

1. **Step 1（基盤）** → すべての機能で使う共通部分
2. **Step 2（フィールド）** → 視覚的フィードバックが得られる
3. **Step 3（モンスター基本）** → スポーン→移動→消滅を確認
4. **Step 4（衝突）** → 衝突挙動を確認
5. **Step 5（アイテム）** → ゲームプレイ要素を追加
6. **Step 6（統合テスト）** → 全体の動作確認

---

## 今後の拡張予定

- Wave定義の外部ファイル化（RON/JSON）
- アイテムの種類追加（加速床、ジャンプ台など）
- 障害物の実装（泥沼など）
- UIの追加（残りモンスター数、タイマーなど）
- ゴースト型モンスター（すり抜け）の実装

---

## 備考

- このプランはプロトタイプのため、パフォーマンス最適化は行わない
- モンスターの見た目は単純な四角形（SpriteBundle + Color）
- デバッグ用途のため、Gizmos を積極的に使用
- 実装中に問題が発生した場合は、このプランを更新する
