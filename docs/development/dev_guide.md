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
my_game/
├─ Cargo.toml
└─ src/
   ├─ main.rs
   ├─ lib.rs                # AppPlugin (全体組み立て)
   ├─ core/                 # 共通基盤
   │   ├─ types.rs
   │   ├─ config.rs
   │   └─ animation.rs
   ├─ feature/              # ゲーム要素単位
   │   ├─ player/
   │   │   ├─ mod.rs
   │   │   ├─ plugin.rs
   │   │   ├─ components.rs
   │   │   └─ systems.rs
   │   ├─ monster/
   │   │   ├─ mod.rs
   │   │   ├─ plugin.rs
   │   │   ├─ components.rs
   │   │   ├─ config.rs
   │   │   ├─ spawn.rs
   │   │   ├─ ai.rs
   │   │   ├─ movement.rs
   │   │   ├─ attack.rs
   │   │   ├─ death.rs
   │   │   └─ types/
   │   │       ├─ mod.rs
   │   │       ├─ slime.rs
   │   │       ├─ sniper.rs
   │   │       └─ boss_slug.rs
   │   ├─ combat/
   │   │   ├─ plugin.rs
   │   │   ├─ damage.rs
   │   │   ├─ hitbox.rs
   │   │   └─ events.rs
   │   ├─ world/
   │   │   ├─ plugin.rs
   │   │   ├─ tilemap.rs
   │   │   └─ camera.rs
   │   └─ ui/
   │       ├─ plugin.rs
   │       ├─ hud.rs
   │       └─ menu.rs
   └─ assets/
       └─ monster/
           ├─ monsters.ron
           └─ textures/
````

### 構成の考え方

* `core/` は共通型・設定・アニメーション等の「基盤」
* `feature/` 以下は**ゲーム要素単位（ドメイン単位）**
* `monster/` は敵の全要素を集約し、**共通ロジック＋個別拡張**構成
* `assets/` に外部設定ファイル（RON / JSON）を置くことで**データ駆動**化

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
    #[default] MainMenu,
    InGame,
    Paused,
}
```

`OnEnter` / `OnExit` / `run_if(in_state(...))` を用いることで
フェーズに応じた処理を分離する。

---

### 3.3 イベント駆動による疎結合化

複数システム間の通信は、直接参照ではなく **BevyのEventシステム** を用いる。

```rust
pub struct MonsterHitEvent {
    pub entity: Entity,
    pub damage: f32,
}

fn monster_hit_detection_system(
    mut events: EventWriter<MonsterHitEvent>,
    query: Query<(Entity, &Monster, &Hitbox)>,
) {
    for (entity, _, hitbox) in &query {
        if hitbox.collided {
            events.send(MonsterHitEvent { entity, damage: 5.0 });
        }
    }
}
```

別システムで `EventReader` を介して反応し、
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
use my_game::AppPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AppPlugin)
        .run();
}
```

### lib.rs

```rust
use bevy::prelude::*;
pub mod feature;
use feature::{player::PlayerPlugin, monster::MonsterPlugin, ui::UIPlugin};

#[derive(States, Default, Debug, PartialEq, Eq, Hash, Clone)]
pub enum GameState {
    #[default] MainMenu,
    InGame,
    Paused,
}

pub struct AppPlugin;
impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_plugin(PlayerPlugin)
            .add_plugin(MonsterPlugin)
            .add_plugin(UIPlugin);
    }
}
```

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
