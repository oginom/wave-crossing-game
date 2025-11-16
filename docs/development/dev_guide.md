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
- イベント定義は `#[derive(Message)]` を使用

```rust
#[derive(Message)]
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

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterKind {
    Kappa,      // 河童 - 標準的な速度と挙動
    Ghost,      // ゴースト - 高速移動、すり抜け特性
    Bakeneko,   // 化け猫 - 大型でゆっくり、マイペース挙動
}
```

**現在の実装**: 本プロジェクトでは体力（Health）システムは未実装。
モンスターは待機時間（WaitMeter）が閾値を超えると消滅する仕組みを採用。

### 4.2 コンフィグ駆動設計

敵定義は `MonsterDefinition` に集約し、RONファイルで外部化。

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterDefinition {
    pub kind: MonsterKind,
    pub speed: f32,
    pub size: f32,
    pub color: (f32, f32, f32),  // デバッグ用（テクスチャがない場合のフォールバック）
    pub wait_threshold: f32,
    pub special_behavior: SpecialBehavior,
    pub texture_path: String,  // テクスチャファイルのパス
}
```

> 各敵の行動・ステータス・見た目を外部ファイルで定義し、
> **「コードを増やさずに敵を増やせる」構造**を目指す。

---

### 4.3 特殊挙動システム

本ゲームでは複雑なAI状態機械ではなく、**特殊挙動（SpecialBehavior）** による拡張方式を採用。

```rust
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SpecialBehavior {
    /// 特殊挙動なし（標準的な挙動のみ）
    None,

    /// すり抜け: 他のモンスターと相互衝突せずにすれ違う
    PassThrough,

    /// マイペース: 時々一定時間立ち止まる
    MyPace {
        stop_interval: f32,  // 立ち止まる間隔（秒）
        stop_duration: f32,  // 立ち止まる時間（秒）
    },
}
```

**設計思想**:
- 基本挙動は「直進」のみ
- 特殊な動きが必要な場合のみ `SpecialBehavior` で拡張
- 各挙動は独立したシステムで処理（例: `my_pace_system`, 衝突判定でPassThroughをチェック）

**メリット**:
- シンプルで理解しやすい
- 個別のシステムとして実装できるため、デバッグが容易
- 新しい挙動の追加が簡単（enum variantとシステムを追加するだけ）

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
/// モンスターのスポーンキュー（リソース）
#[derive(Resource)]
pub struct MonsterSpawnQueue {
    pub spawns: Vec<SpawnDefinition>,
    pub waves: Vec<WaveDefinition>,
    pub timer: f32,
    pub processed_wave_indices: Vec<usize>,  // 処理済みWaveのインデックス
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterState {
    Staging,   // 画面端で待機中
    Moving,    // 移動中
    Reached,   // 到達（消滅待ち）
}

/// Wave定義（特定の時刻に出現するモンスターのグループ）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveDefinition {
    /// Wave開始時間（ゲーム開始からの経過時間・秒）
    pub start_time: f32,
    /// このWaveでスポーンするモンスターのリスト
    pub monsters: Vec<SpawnDefinition>,
}

/// モンスターのスポーン定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnDefinition {
    pub kind: MonsterKind,
    pub direction: Direction,
    /// スポーン位置（進行方向に垂直な軸の座標）
    /// Right/Leftの場合はy座標、Up/Downの場合はx座標を指定
    pub grid_pos: i32,
    pub delay: f32,  // Wave開始時刻からの相対遅延（秒）
}
```

**スポーンシステムの流れ**

1. **Wave定義の読み込み**（`Startup`システム）
   - RONファイルから Wave 構成を非同期ロード
   - アセットが読み込まれたら `MonsterSpawnQueue` リソースに登録

2. **ステージングフェーズ**（`MonsterState::Staging`）
   - モンスターを画面端（フィールド外）に配置
   - `StagingTimer` で待機時間を管理
   - 待機時間：定数 `STAGING_DURATION`（通常2秒程度）

3. **移動開始**（`MonsterState::Staging` → `MonsterState::Moving`）
   - `StagingTimer` 経過で `MonsterState::Moving` に遷移
   - 移動システムが動作開始（衝突判定・移動・待機メーター更新）

**サンプルコード**

```rust
/// モンスターをスポーンするシステム
pub fn spawn_monsters_system(
    mut commands: Commands,
    time: Res<Time>,
    spawn_queue: Option<ResMut<MonsterSpawnQueue>>,
    monster_defs: Res<MonsterDefinitions>,
    asset_server: Res<AssetServer>,
) {
    // MonsterSpawnQueueが初期化されるまで待機
    let Some(mut spawn_queue) = spawn_queue else {
        return;
    };
    spawn_queue.timer += time.delta_secs();

    // Wave開始時間を確認して、新しいWaveのモンスターをスポーンキューに追加
    for (index, wave) in spawn_queue.waves.iter().enumerate() {
        if spawn_queue.processed_wave_indices.contains(&index) {
            continue;
        }

        if spawn_queue.timer >= wave.start_time {
            for monster_spawn in &wave.monsters {
                let mut spawn_def = monster_spawn.clone();
                // delayはWave開始時間からの相対時間なので、絶対時間に変換
                spawn_def.delay = wave.start_time + monster_spawn.delay;
                spawn_queue.spawns.push(spawn_def);
            }
            spawn_queue.processed_wave_indices.push(index);
        }
    }

    // スポーン予定のモンスターをチェック
    let mut spawned_indices = Vec::new();
    for (index, spawn_def) in spawn_queue.spawns.iter().enumerate() {
        if spawn_queue.timer >= spawn_def.delay {
            spawn_monster(&mut commands, spawn_def, &monster_defs, &asset_server);
            spawned_indices.push(index);
        }
    }

    // スポーン済みの定義を削除
    for index in spawned_indices.iter().rev() {
        spawn_queue.spawns.remove(*index);
    }
}

/// ステージングタイマーを更新し、時間経過でMoving状態に遷移
pub fn staging_timer_system(
    time: Res<Time>,
    mut query: Query<(&mut StagingTimer, &mut MonsterState), With<Monster>>,
) {
    for (mut timer, mut state) in &mut query {
        if *state == MonsterState::Staging {
            timer.remaining -= time.delta_secs();
            if timer.remaining <= 0.0 {
                *state = MonsterState::Moving;
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

## 10. カスタムアセットローダー

本プロジェクトでは、RONファイルからステージ定義とモンスター定義を読み込むための
**カスタムアセットローダー**を実装している。

### 10.1 StageAssetPlugin

`core/stage_asset.rs` でカスタムアセットローダーを定義：

```rust
pub struct StageAssetPlugin;

impl Plugin for StageAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StageLevelAsset>()
            .init_asset_loader::<StageLevelAssetLoader>()
            .init_asset::<MonsterDefinitionsAsset>()
            .init_asset_loader::<MonsterDefinitionsAssetLoader>();
    }
}
```

### 10.2 アセット定義

**ステージレベルアセット**:
```rust
#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct StageLevelAsset {
    pub stage: u32,
    pub level: u32,
    pub waves: Vec<WaveDefinition>,
}
```

**モンスター定義アセット**:
```rust
#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct MonsterDefinitionsAsset {
    pub definitions: Vec<MonsterDefinition>,
}
```

### 10.3 非同期ロードの仕組み

1. **Startupシステムでロード開始**:
   ```rust
   fn load_monster_definitions_system(mut commands: Commands, asset_server: Res<AssetServer>) {
       let handle: Handle<MonsterDefinitionsAsset> = asset_server.load("monsters.ron");
       commands.insert_resource(MonsterDefinitionsLoader {
           handle,
           loaded: false,
       });
   }
   ```

2. **Updateシステムでロード完了を監視**:
   ```rust
   fn initialize_monster_definitions_system(
       mut loader: ResMut<MonsterDefinitionsLoader>,
       monster_def_assets: Res<Assets<MonsterDefinitionsAsset>>,
       mut definitions: ResMut<MonsterDefinitions>,
   ) {
       if loader.loaded {
           return;
       }

       if let Some(monster_def_asset) = monster_def_assets.get(&loader.handle) {
           *definitions = MonsterDefinitions::from_hashmap(monster_def_asset.to_hashmap());
           loader.loaded = true;
       }
   }
   ```

### 10.4 RONファイルの例

**assets/monsters.ron**:
```ron
(
    definitions: [
        (
            kind: Kappa,
            speed: 100.0,
            size: 0.6,
            color: (0.2, 0.8, 0.5),
            wait_threshold: 10.0,
            special_behavior: None,
            texture_path: "img/kappa.png",
        ),
        // ... 他のモンスター定義
    ]
)
```

**assets/stages/stage1_level1.ron**:
```ron
(
    stage: 1,
    level: 1,
    waves: [
        (
            start_time: 0.0,
            monsters: [
                (kind: Kappa, direction: Right, grid_pos: 5, delay: 0.0),
                // ... 他のスポーン定義
            ],
        ),
    ]
)
```

### 10.5 利点

- **データ駆動**: コードを変更せずにゲームバランスを調整可能
- **非同期ロード**: ゲーム起動時の読み込み時間を最小化
- **型安全**: RONのデシリアライズ時に構造の検証が行われる
- **拡張性**: 新しいステージやモンスターの追加が容易

---

## 11. 現在の実装における独自システム

本セクションでは、dev_guideの基本方針には記載されていないが、
現在の実装で導入されている独自のゲームシステムを説明する。

---

### 11.1 グリッドベース座標管理

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

### 11.2 設定ファイルの2段階分離

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

### 11.3 待機メーター（WaitMeter）システム

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

### 11.4 プレイヤーゲージシステム（魂と虚）

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

### 11.5 MonsterPropertyとMovementの分離

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
    /// 移動が有効かどうか（特殊挙動で一時停止する場合に使用）
    pub enabled: bool,
}
```

**設計意図**：
- アイテム効果（例：ぐるぐる床で方向転換）を適用しても、元の状態にリセット可能
- デバッグ時に「本来の挙動」と「現在の挙動」を比較可能
- **`enabled`フィールド**: 特殊挙動（MyPace等）で一時的に移動を停止する際に使用
  - `enabled = false` にすると、移動システムが速度を0として扱う
  - 衝突判定による停止とは別の、意図的な停止を実現
- 将来的なバフ/デバフシステムの基盤

---

### 11.6 衝突検出システム

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

### 11.7 アイテム配置システム

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

### 11.8 Gizmosによるデバッグ描画

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

### 11.9 システムのチェーン実行

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

### 11.10 特殊挙動システムの詳細

セクション4.3で紹介した特殊挙動システムの実装詳細を説明する。

#### MyPace挙動の実装

**コンポーネント定義**:
```rust
/// マイペース挙動用のタイマーコンポーネント
#[derive(Component)]
pub struct MyPaceTimer {
    pub interval_timer: Timer,
    pub stop_timer: Timer,
    pub is_stopped: bool,
}
```

**システム実装** (`monster/special_behavior.rs`):
```rust
pub fn my_pace_system(
    time: Res<Time>,
    mut query: Query<(&SpecialBehavior, &mut MyPaceTimer, &mut Movement)>,
) {
    for (behavior, mut timer, mut movement) in &mut query {
        if let SpecialBehavior::MyPace { .. } = behavior {
            if timer.is_stopped {
                // 立ち止まり中
                timer.stop_timer.tick(time.delta());
                if timer.stop_timer.is_finished() {
                    timer.is_stopped = false;
                    movement.enabled = true;  // 移動再開
                }
            } else {
                // 通常移動中
                timer.interval_timer.tick(time.delta());
                if timer.interval_timer.is_finished() {
                    timer.is_stopped = true;
                    timer.stop_timer.reset();
                    movement.enabled = false;  // 移動停止
                }
            }
        }
    }
}
```

**特徴**:
- `Movement.enabled` フィールドを制御して移動のON/OFFを切り替え
- 衝突判定とは独立した停止メカニズム
- タイマーベースのシンプルな実装

#### PassThrough挙動の実装

衝突検出システム内で特殊挙動をチェック：

```rust
pub fn collision_detection_system(
    mut query: Query<(..., Option<&SpecialBehavior>), With<Monster>>,
) {
    for (entity, ..., special_behavior) in &mut query {
        // PassThrough挙動を持つモンスターは衝突判定をスキップ
        if matches!(special_behavior, Some(SpecialBehavior::PassThrough)) {
            collision_state.is_colliding = false;
            continue;
        }
        // ... 通常の衝突判定
    }
}
```

**特徴**:
- 既存の衝突検出システムに条件分岐を追加するだけ
- 新しいシステムを追加する必要がない
- コンポーネントの有無で挙動を切り替え

---

### 11.11 ゲージUI実装（Bevy 0.15+新UI）

Bevy 0.15以降の新UIシステム（`Node`, `Text`, `BackgroundColor`等）を使用したゲージ表示。

**UI構造**:
```rust
pub fn setup_gauges_ui_system(mut commands: Commands) {
    commands
        .spawn((
            GaugesPanel,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                bottom: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // ゲージ行を作成
            create_gauge_row(parent, "Spirit", ...);
            create_gauge_row(parent, "Void", ...);
        });
}
```

**プログレスバー更新**:
```rust
pub fn update_spirit_gauge_ui_system(
    gauges: Res<PlayerGauges>,
    mut bar_query: Query<&mut Node, With<SpiritGaugeBar>>,
    mut text_query: Query<&mut Text, With<SpiritGaugeText>>,
) {
    for mut node in bar_query.iter_mut() {
        node.width = Val::Percent(gauges.spirit.ratio() * 100.0);
    }

    for mut text in text_query.iter_mut() {
        **text = format!("{:.0}/{:.0}", gauges.spirit.current, gauges.spirit.max);
    }
}
```

**Bevy 0.15+の変更点**:
- `Style` → `Node` コンポーネントに変更
- `TextBundle` → `Text` コンポーネント + `TextFont` + `TextColor` に分離
- `Color::rgb()` → `Color::srgb()` に変更

---

### 11.12 まとめ

現在の実装では、dev_guideの基本方針に加えて以下の独自システムを導入：

| システム                     | 目的                       | 実装場所                      |
| ------------------------ | ------------------------ | ------------------------- |
| カスタムアセットローダー             | RONファイルからのデータ駆動読み込み      | `core/stage_asset.rs`     |
| グリッド座標管理                 | 座標変換の一貫性                 | `core/types.rs`           |
| 設定の2段階分離                 | 技術/バランスの明確化              | `core/config.rs` など       |
| WaitMeter                | モンスター停止時のペナルティ           | `monster/wait.rs`         |
| PlayerGauges             | ゲーム進行管理（魂/虚）             | `player/gauges.rs`        |
| Property/Movement分離      | アイテム効果の適用とリセット           | `monster/components.rs`   |
| Movement.enabled         | 特殊挙動による一時停止              | `monster/components.rs`   |
| SpecialBehavior          | モンスター個別挙動の拡張             | `monster/special_behavior.rs` |
| SpawnDefinition          | 柔軟なモンスター配置               | `monster/spawn.rs`        |
| 衝突検出                     | モンスター同士の相互作用             | `monster/collision.rs`    |
| アイテム配置                   | プレイヤーの戦略的介入              | `item/placement.rs`       |
| ゲージUI（Bevy 0.15+新UI）     | プレイヤーへの視覚的フィードバック        | `ui/gauges.rs`            |
| Gizmos描画                 | デバッグ効率向上                 | `world/grid.rs`           |
| システムチェーン                 | 実行順序の厳密な制御（Bevy 0.17）    | 各 `plugin.rs`            |

これらは**プロトタイプフェーズの実装**であり、将来的なデータ駆動化・モンスター種類の拡張に備えた基盤となっている。
