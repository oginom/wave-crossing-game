# Bevy 2D Game Development Guidelines  
*(Version: Bevy 0.17.2)*

## 概要

本ドキュメントは、**Bevy (v0.17.2)** を用いた 2D ゲーム開発における実装方針・構成・命名規則・AIコーディング方針を定めるものである。  
目的は、**スケーラブルかつモジュール化された小規模ゲーム開発の基盤を提供**し、将来的な機能拡張やAI補助による自動生成を容易にすること。

---

## 1. Bevy の設計思想と本プロジェクト方針

Bevy は **ECS（Entity Component System）** を中心としたデータ駆動型の Rust 製ゲームエンジンであり、以下の設計哲学を持つ。

- **Simplicity（シンプルさ）**：構成は明快で、直感的な Rust コードで表現される。  
- **Modularity（モジュール性）**：全ての機能はプラグイン単位で拡張可能。  
- **Data-Driven（データ駆動）**：状態や構造をデータで定義し、システムがそれを処理する。  
- **Parallelism（並列実行）**：ECS が自動的にデータアクセスを解析し、システムを並列化。  

本プロジェクトではこれらの原則を踏まえ、**機能ごとにプラグインを分離し、共通ロジックを共有する構造**を採用する。  
特に、敵キャラクターが30種類以上存在することを想定し、**データ駆動＋共通AI＋拡張可能な個別挙動モジュール**構成を基本とする。

---

## 2. ディレクトリ構成

下記は本プロジェクトの標準的な構成例である。

```text
wave_crossing_game/
├─ Cargo.toml
├─ docs/
│  ├─ development/
│  │  └─ dev_guide.md
│  └─ plans/
│     ├─ 01_minimal_scene.md
│     └─ 02_game_states.md
└─ src/
   ├─ main.rs
   ├─ lib.rs                # AppPlugin (全体組み立て)
   ├─ core/                 # 共通基盤
   │   ├─ mod.rs
   │   ├─ types.rs          # GridPosition, Direction, 座標変換関数
   │   ├─ config.rs         # 技術的定数 (GRID_SIZE, FIELD_WIDTH等)
   │   └─ level.rs          # ゲームバランス調整値 (MONSTER_SPEED等)
   └─ feature/              # ゲーム要素単位
       ├─ mod.rs
       ├─ world/
       │   ├─ mod.rs
       │   ├─ plugin.rs
       │   └─ grid.rs       # グリッド描画
       ├─ monster/
       │   ├─ mod.rs
       │   ├─ plugin.rs
       │   ├─ components.rs # Monster, MonsterState, Movement等
       │   ├─ spawn.rs      # スポーン管理
       │   ├─ staging.rs    # 待機フェーズ
       │   ├─ movement.rs   # 移動システム
       │   ├─ collision.rs  # 衝突検出
       │   ├─ wait.rs       # 待機メーター
       │   ├─ despawn.rs    # 消滅処理
       │   └─ events.rs     # MonsterDespawnEvent
       ├─ item/
       │   ├─ mod.rs
       │   ├─ plugin.rs
       │   ├─ components.rs # Item, ItemKind, RotationTile
       │   ├─ placement.rs  # アイテム配置
       │   └─ rotation_tile.rs # ぐるぐる床の効果
       ├─ player/
       │   ├─ mod.rs
       │   ├─ plugin.rs
       │   └─ gauges.rs     # SpiritGauge, VoidGauge
       └─ ui/
           ├─ mod.rs
           └─ gauges.rs     # ゲージUI表示
```

### 構成の考え方

* `core/` は共通型・設定・アニメーション等の「基盤」
  * **技術的定数** (`config.rs`) と **ゲームバランス** (`level.rs`) を明確に分離
  * グリッド座標システム (`types.rs`) で座標管理を統一
* `feature/` 以下は**ゲーム要素単位（ドメイン単位）**
* `monster/` は敵の全要素を集約し、**共通ロジック＋個別拡張**構成
  * 将来的に `types/` ディレクトリを追加し、個別モンスター実装を配置予定
* `assets/` に外部設定ファイル（RON / JSON）を置くことで**データ駆動**化（将来対応）

---

## 3. コーディング方針

### 3.1 プラグイン指向設計

すべての主要要素（Player, Monster, Combat, UIなど）は **Plugin構造体** で定義する。

```rust
pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_monster_config)
            .add_systems(
                Update,
                (
                    spawn_monster,
                    monster_ai_system,
                    movement_system,
                    attack_system,
                )
                .run_if(in_state(GameState::InGame))
            );
    }
}
```

これにより、

* **main.rs は極限まで薄く保つ**
* **AppPlugin** が全体を構築（`lib.rs`で登録）
* **各プラグインは責務を限定**（SRPの徹底）

---

### 3.2 Stateベースのフェーズ管理

状態遷移を明示的に制御することで、
**Update負荷削減とシステムの明確なON/OFF**を実現する。

```rust
#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum GameState {
    #[default]
    InGame,
    GameOver,
}
```

**現在の実装**: プロトタイプフェーズでは `InGame` と `GameOver` の2状態のみ実装。
将来的に `MainMenu`, `Paused` などを追加予定。

`OnEnter` / `OnExit` / `run_if(in_state(...))` を用いることで
フェーズに応じた処理を分離する。

---

### 3.3 イベント駆動による疎結合化

複数システム間の通信は、直接参照ではなく **Bevyのメッセージングシステム** を用いる。

**Bevy 0.17以降の変更点**:
- `EventWriter`/`EventReader` は `MessageWriter`/`MessageReader` に変更
- イベントの登録は `.add_message::<EventType>()` を使用

```rust
#[derive(Event)]
pub struct MonsterDespawnEvent {
    pub entity: Entity,
    pub cause: DespawnCause,
}

fn monster_despawn_system(
    mut messages: MessageWriter<MonsterDespawnEvent>,
    query: Query<(Entity, &Monster, &WaitMeter)>,
) {
    for (entity, _, wait_meter) in &query {
        if wait_meter.is_expired() {
            messages.send(MonsterDespawnEvent {
                entity,
                cause: DespawnCause::WaitExpired
            });
        }
    }
}

fn handle_despawn_system(
    mut messages: MessageReader<MonsterDespawnEvent>,
    mut gauges: ResMut<PlayerGauges>,
) {
    for event in messages.read() {
        match event.cause {
            DespawnCause::ReachedGoal => gauges.spirit.add(10.0),
            DespawnCause::WaitExpired => gauges.void.add(5.0),
        }
    }
}
```

別システムで `MessageReader` を介して反応し、
**依存のない拡張可能な設計**を維持する。

---

## 4. 敵キャラクター構成指針

### 4.1 共通コンポーネント

```rust
#[derive(Component)]
pub struct Monster;

#[derive(Component, Clone, Copy, Debug)]
pub enum MonsterKind {
    Slime,
    Goblin,
    Shooter,
    Tank,
    BossSlug,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}
```

### 4.2 コンフィグ駆動設計

敵定義は `MonsterConfig` に集約し、RON/JSON で外部化可能。

```rust
#[derive(Clone)]
pub struct MonsterDefinition {
    pub kind: MonsterKind,
    pub hp: f32,
    pub speed: f32,
    pub attack_damage: f32,
    pub texture_path: &'static str,
    pub ai_pattern: AiPattern,
}
```

> 各敵の行動・ステータス・見た目を外部ファイルで定義し、
> **「コードを増やさずに敵を増やせる」構造**を目指す。

---

### 4.3 AIと個別挙動

* 共通AIは `ai.rs` にまとめる。
* 特殊挙動は `monster/types/` に分離する。

```rust
match (def.ai_pattern, ai.state) {
    (AiPattern::MeleeChaser, AiState::Idle) => { ... }
    (AiPattern::BossScripted, _) => { handle_boss_ai(entity, &mut ai, &def); }
    _ => {}
}
```

---

## 5. サンプルコードまとめ

### main.rs

```rust
use bevy::prelude::*;
use wave_crossing_game::AppPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AppPlugin)
        .run();
}
```

**Bevy 0.17の変更点**: `.add_plugin()` は `.add_plugins()` に統一された。

### lib.rs

```rust
use bevy::prelude::*;

pub mod core;
pub mod feature;

use feature::world::WorldPlugin;
use feature::monster::MonsterPlugin;
use feature::item::ItemPlugin;
use feature::player::PlayerPlugin;

#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum GameState {
    #[default]
    InGame,
    GameOver,
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                WorldPlugin,
                MonsterPlugin,
                ItemPlugin,
                PlayerPlugin,
            ));
    }
}
```

**Bevy 0.17の変更点**:
- `.add_state()` は `.init_state()` に変更
- `.add_plugins()` はタプルで複数のプラグインをまとめて登録可能

---

## 6. 開発ルール

| 項目          | 指針                                                                    |
| ----------- | --------------------------------------------------------------------- |
| **責務分離**    | 各モジュールは「1つの役割（Plugin単位）」に限定する。                                        |
| **AI補助生成**  | AIコーディング時は構造を壊さず、既存Plugin/Componentに従う。                               |
| **命名規則**    | `snake_case` for variables/functions, `PascalCase` for types/plugins. |
| **イベント駆動**  | 他システムとの直接依存は禁止。イベント経由で通信。                                             |
| **データ駆動**   | 敵やアイテムなどは原則RON/JSONからロード。                                             |
| **ドキュメント化** | 各PluginとSystemに `///` ドキュメンテーションコメント必須。                               |

---

## 7. この設計の利点

1. **スケール性**：敵の追加がデータ記述中心で済み、コード爆発を防ぐ。
2. **再利用性**：共通システムを汎用的に保つことで複数タイトルで流用可能。
3. **保守性**：機能単位のプラグインで依存関係を局所化。
4. **テスト容易性**：`MinimalPlugins`を利用して単体テスト可能。
5. **AI統合性**：AIコーディング支援に適した分離構造（pluginごと生成が容易）。

---

## 8. まとめ

本設計は、Bevy の理念である **「データ駆動＋モジュール性＋並列性」** に忠実であり、
小規模ゲームでありながら長期的なスケールアップに耐える構造を持つ。

この方針を守ることで、
AIによる自動コード生成・テスト・ドキュメント生成の一貫性を維持できる。

> **Bevyの力を最大限に活かす鍵は、Plugin構造とECS分離。**
>
> "Keep systems small, data rich, and decoupled."

---

## 9. 大量モンスター管理方針

本ゲームでは複数のモンスターが同時に画面上を移動し、プレイヤーはそれらを避けながら交差点を渡る。
このセクションでは、**大量のモンスターインスタンスを効率的に管理する設計方針**を定める。

---

### 9.1 スポーン管理とステージング方式

#### 基本方針

モンスターは **画面の端で待機してから動き始める** ステージング方式を採用する。
これにより、プレイヤーは「どのような敵が来るか」を事前に確認でき、戦略的なゲームプレイが可能になる。

#### 実装構成

**コンポーネント設計**

```rust
#[derive(Component)]
pub struct MonsterSpawnQueue {
    pub waves: Vec<WaveDefinition>,
    pub current_wave: usize,
}

#[derive(Component)]
pub struct StagingArea {
    pub position: Vec2,        // 待機位置（画面端）
    pub direction: Direction,  // 進行方向（Left/Right/Up/Down）
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterState {
    Staging,      // 画面端で待機中
    Moving,       // 移動中
    Attacking,    // 攻撃中
    Dying,        // 死亡モーション中
}

#[derive(Clone)]
pub struct WaveDefinition {
    pub monsters: Vec<MonsterKind>,
    pub spawn_interval: f32,  // 秒単位
    pub direction: Direction,
}
```

**スポーンシステムの流れ**

1. **Wave定義の読み込み**（`OnEnter(GameState::InGame)`）
   - RON/JSONから Wave 構成を読み込み
   - `MonsterSpawnQueue` リソースに登録

2. **ステージングフェーズ**（`MonsterState::Staging`）
   - モンスターを画面端の `StagingArea` に配置
   - 見た目：待機モーション（アイドルアニメーション）
   - 待機時間：数秒〜数分程度（Wave設計次第で調整可能）

3. **移動開始**（`MonsterState::Staging` → `MonsterState::Moving`）
   - タイマー経過で `MonsterState::Moving` に遷移
   - 通常のAI・移動システムが動作開始

**サンプルコード**

```rust
fn spawn_wave_system(
    mut commands: Commands,
    time: Res<Time>,
    mut queue: ResMut<MonsterSpawnQueue>,
    monster_defs: Res<MonsterDefinitions>,
    asset_server: Res<AssetServer>,
) {
    if queue.current_wave >= queue.waves.len() {
        return; // 全Wave完了
    }

    let wave = &queue.waves[queue.current_wave];

    // spawn_interval 経過ごとにモンスターを生成
    if time.elapsed_seconds() > queue.next_spawn_time {
        let monster_kind = wave.monsters[queue.spawn_index];
        let def = monster_defs.get(monster_kind);

        let staging_pos = get_staging_position(wave.direction);

        commands.spawn((
            Monster,
            monster_kind,
            MonsterState::Staging,
            StagingArea {
                position: staging_pos,
                direction: wave.direction,
            },
            Health { current: def.hp, max: def.hp },
            SpriteBundle {
                texture: asset_server.load(def.texture_path),
                transform: Transform::from_translation(staging_pos.extend(0.0)),
                ..default()
            },
            AnimationState::Idle, // 待機モーション
        ));

        queue.spawn_index += 1;
        queue.next_spawn_time = time.elapsed_seconds() + wave.spawn_interval;
    }
}

fn staging_to_moving_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MonsterState, &StagingArea, &mut AnimationState), With<Monster>>,
    time: Res<Time>,
) {
    for (entity, mut state, staging, mut anim) in &mut query {
        if *state == MonsterState::Staging {
            // 待機時間経過で移動開始
            if time.elapsed_seconds() > staging.enter_time + staging.duration {
                *state = MonsterState::Moving;
                *anim = AnimationState::Walking; // 移動モーション

                // 移動コンポーネントを追加
                commands.entity(entity).insert(Velocity {
                    value: get_direction_vector(staging.direction) * def.speed,
                });
            }
        }
    }
}
```

---

### 9.2 パフォーマンス最適化（将来対応）

#### オブジェクトプール

現時点では **実装しない** が、将来的にパフォーマンス上の問題が発生した場合、
以下のようなオブジェクトプール方式を導入する。

**導入判断基準**

- 同時表示モンスター数が **100体以上** になった場合
- スポーン/デスポーンによる FPS低下が観測された場合
- プロファイリングで Entity生成/破棄がボトルネックと判明した場合

**想定実装方針**

```rust
#[derive(Resource)]
pub struct MonsterPool {
    inactive: Vec<Entity>,
    active: HashMap<Entity, MonsterKind>,
}

impl MonsterPool {
    pub fn acquire(&mut self, commands: &mut Commands) -> Entity {
        self.inactive.pop().unwrap_or_else(|| commands.spawn_empty().id())
    }

    pub fn release(&mut self, entity: Entity, commands: &mut Commands) {
        // コンポーネントをリセットし、非表示化
        commands.entity(entity).insert(Visibility::Hidden);
        self.inactive.push(entity);
    }
}
```

> **注意**: Bevyの Entity再利用は慎重に行う必要がある。
> Component削除漏れによるバグを防ぐため、専用のリセット関数を用意する。

---

### 9.3 開発フェーズごとの推奨事項

| フェーズ        | 同時表示数目安 | 推奨実装                               |
| ----------- | ------- | ---------------------------------- |
| **プロトタイプ** | 〜30体   | 単純なSpawn/Despawn（本セクション9.1を実装）      |
| **アルファ版**   | 30〜100体 | Wave管理の洗練 + プロファイリング実施            |
| **ベータ版**    | 100体〜   | 必要に応じてオブジェクトプール導入（9.2）+ 空間分割検討    |

---

### 9.4 まとめ

本ゲームの大量モンスター管理は以下の原則に基づく：

1. **ステージング方式**でプレイヤーに事前情報を提供
2. **Wave定義**をデータ駆動化し、外部ファイルで管理
3. **ECSの並列性**を活かし、特別な最適化は後回し
4. **オブジェクトプール**は必要になってから導入

> "Premature optimization is the root of all evil."
>
> まずは動く実装を作り、計測してから最適化する。

---

## 10. 現在の実装における独自システム

本セクションでは、dev_guideの基本方針には記載されていないが、
現在の実装で導入されている独自のゲームシステムを説明する。

---

### 10.1 グリッドベース座標管理

本ゲームは**グリッドベースの座標系**を採用しており、
`core/types.rs` で以下の機能を提供する：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

/// グリッド座標をワールド座標に変換
pub fn grid_to_world(grid_pos: GridPosition, grid_size: f32, field_width: i32, field_height: i32) -> Vec2;

/// ワールド座標をグリッド座標に変換
pub fn world_to_grid(world_pos: Vec2, grid_size: f32, field_width: i32, field_height: i32) -> GridPosition;

/// グリッド座標がフィールド範囲内かチェック
pub fn is_valid_grid_position(grid_pos: GridPosition, field_width: i32, field_height: i32) -> bool;
```

**設計意図**：
- アイテム配置やモンスター移動をグリッド単位で管理
- 座標変換の一貫性を保証
- デバッグ時の可視性向上（グリッド線描画との連携）

---

### 10.2 設定ファイルの2段階分離

`core/config.rs` と `core/level.rs` で設定を明確に分離：

#### config.rs - 技術的定数
```rust
// フィールド設定
pub const GRID_SIZE: f32 = 64.0;
pub const FIELD_WIDTH: i32 = 10;
pub const FIELD_HEIGHT: i32 = 10;

// 衝突判定
pub const COLLISION_THRESHOLD: f32 = 32.0;

// 色設定（デバッグ用）
pub const GRID_COLOR: (f32, f32, f32) = (0.3, 0.3, 0.3);
```

#### level.rs - ゲームバランス調整値
```rust
// モンスター関連
pub const MONSTER_SPEED: f32 = 100.0;
pub const STAGING_DURATION: f32 = 2.0;
pub const WAIT_THRESHOLD: f32 = 10.0;

// プレイヤーゲージ関連
pub const SPIRIT_MAX: f32 = 100.0;
pub const SPIRIT_INITIAL: f32 = 50.0;
pub const SPIRIT_GAIN_PER_GOAL: f32 = 10.0;
pub const VOID_MAX: f32 = 100.0;
pub const VOID_GAIN_PER_DESPAWN: f32 = 5.0;
```

**分離の利点**：
- ゲームバランス調整時に `level.rs` のみ変更すればよい
- 技術的制約（グリッドサイズ等）との混同を防ぐ
- 将来的なデータ駆動化への移行が容易

---

### 10.3 待機メーター（WaitMeter）システム

モンスターが一定時間停止すると消滅する仕組み：

```rust
#[derive(Component, Debug, Clone, Copy)]
pub struct WaitMeter {
    pub current: f32,      // 現在の待機時間（秒）
    pub threshold: f32,    // 消滅する待機時間の閾値（秒）
    pub was_stopped: bool, // 前フレームでの停止状態
}

impl WaitMeter {
    pub fn progress_ratio(&self) -> f32 {
        (self.current / self.threshold).min(1.0)
    }

    pub fn is_expired(&self) -> bool {
        self.current >= self.threshold
    }
}
```

**システムフロー**：
1. モンスターが衝突などで停止すると `WaitMeter.current` が増加
2. 閾値に達すると `MonsterDespawnEvent` が発行される
3. プレイヤーの `VoidGauge` が増加

**ゲームデザイン意図**：
- プレイヤーの失敗（モンスターを長時間停止させる）にペナルティを課す
- ゲージがいっぱいになるとゲームオーバー

---

### 10.4 プレイヤーゲージシステム（魂と虚）

2つの独立したゲージでゲーム進行を管理：

```rust
#[derive(Resource, Debug, Clone)]
pub struct PlayerGauges {
    pub spirit: SpiritGauge,
    pub void: VoidGauge,
}

/// 魂（スピリット）ゲージ
pub struct SpiritGauge {
    pub current: f32,
    pub max: f32,
}

/// 虚（ヴォイド）ゲージ
pub struct VoidGauge {
    pub current: f32,
    pub max: f32,
}
```

**ゲージの役割**：

| ゲージ    | 増加条件                  | ゲーム的意味            | 満タン時の挙動  |
| ------ | --------------------- | ----------------- | -------- |
| Spirit | モンスターがゴールに到達          | プレイヤーの成功          | （現在は未実装） |
| Void   | モンスターが待機時間切れで消滅（失敗） | プレイヤーの失敗の蓄積（ペナルティ） | ゲームオーバー  |

将来的には、Spirit を使ってアイテムを購入する等の拡張を想定。

---

### 10.5 MonsterPropertyとMovementの分離

モンスターの「基本パラメータ」と「実際の移動パラメータ」を分離：

```rust
/// モンスターの本来のパラメータ（アイテムなどの影響を受けない基本値）
#[derive(Component, Debug, Clone, Copy)]
pub struct MonsterProperty {
    pub base_direction: Direction,
    pub base_speed: f32,
}

/// 移動情報（実際の移動に使用される、アイテムや環境の影響を受けた値）
#[derive(Component, Debug, Clone, Copy)]
pub struct Movement {
    pub direction: Direction,
    pub speed: f32,
}
```

**設計意図**：
- アイテム効果（例：ぐるぐる床で方向転換）を適用しても、元の状態にリセット可能
- デバッグ時に「本来の挙動」と「現在の挙動」を比較可能
- 将来的なバフ/デバフシステムの基盤

---

### 10.6 衝突検出システム

モンスター同士の衝突を検出し、停止させる：

```rust
#[derive(Component, Debug, Clone, Copy)]
pub struct CollisionBox {
    pub size: Vec2,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct CollisionState {
    pub is_colliding: bool,
}
```

**システム実装** (`collision.rs`):
- 全モンスターペアに対してAABB（Axis-Aligned Bounding Box）衝突判定
- 衝突時は `Movement.speed` を 0 にして停止
- 衝突解消時は `MonsterProperty.base_speed` から速度を復元

**Bevy 0.17対応**：
- システム実行順序を `.chain()` で明示的に制御
- `Query` のパラメータ順序に注意（Bevy 0.17で厳格化）

---

### 10.7 アイテム配置システム

マウスクリックでグリッド上にアイテムを配置：

```rust
#[derive(Component)]
pub struct Item;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    RotationTile, // ぐるぐる床
}

#[derive(Component)]
pub struct RotationTile {
    pub grid_pos: GridPosition,
}
```

**実装機能**：
- マウスカーソル位置をグリッド座標に変換
- クリック時にアイテムを配置
- ぐるぐる床：踏んだモンスターの向きを90度右回転

**将来拡張**：
- アイテム種類の追加（加速床、減速床、ワープゲートなど）
- Spirit ゲージを消費してアイテムを購入する経済システム

---

### 10.8 Gizmosによるデバッグ描画

開発効率向上のため、視覚的フィードバックを提供：

```rust
pub fn draw_grid_system(mut gizmos: Gizmos) {
    let grid_color = Color::srgb(GRID_COLOR.0, GRID_COLOR.1, GRID_COLOR.2);

    // 縦線・横線を描画
    for i in 0..=FIELD_WIDTH {
        let x = (i as f32 - FIELD_WIDTH as f32 / 2.0) * GRID_SIZE;
        gizmos.line_2d(/*...*/);
    }
}
```

**Bevy 0.17の変更点**：
- `Color::rgb()` は `Color::srgb()` に変更（sRGB色空間の明示化）

---

### 10.9 システムのチェーン実行

Bevy 0.17では `.chain()` でシステムの実行順序を厳密に制御：

```rust
.add_systems(
    Update,
    (
        spawn_monsters_system,
        staging_timer_system,
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

**実行順序の重要性**：
1. `collision_detection_system` で衝突状態を更新
2. `monster_movement_system` で移動（衝突していれば停止）
3. `update_wait_meter_system` で待機時間を計測
4. `despawn_expired_monsters_system` で閾値超過を判定

順序が逆だと1フレーム遅れが発生し、挙動が不自然になる。

---

### 10.10 まとめ

現在の実装では、dev_guideの基本方針に加えて以下の独自システムを導入：

| システム            | 目的                       | 実装場所                   |
| --------------- | ------------------------ | ---------------------- |
| グリッド座標管理        | 座標変換の一貫性                 | `core/types.rs`        |
| 設定の2段階分離        | 技術/バランスの明確化              | `core/config.rs` など    |
| WaitMeter       | モンスター停止時のペナルティ           | `monster/wait.rs`      |
| PlayerGauges    | ゲーム進行管理（魂/虚）             | `player/gauges.rs`     |
| Property/Movement分離 | アイテム効果の適用とリセット           | `monster/components.rs` |
| 衝突検出            | モンスター同士の相互作用             | `monster/collision.rs` |
| アイテム配置          | プレイヤーの戦略的介入              | `item/placement.rs`    |
| Gizmos描画        | デバッグ効率向上                 | `world/grid.rs`        |
| システムチェーン        | 実行順序の厳密な制御（Bevy 0.17）    | 各 `plugin.rs`         |

これらは**プロトタイプフェーズの実装**であり、将来的なデータ駆動化・モンスター種類の拡張に備えた基盤となっている。
