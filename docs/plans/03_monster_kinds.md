# Plan 03: MonsterKind導入とデータ駆動化

**ステータス**: 計画中
**作成日**: 2025-11-14
**関連**: dev_guide.md セクション4「敵キャラクター構成指針」

---

## 目的

現在の実装は単一種類のモンスターのみを扱っているが、
dev_guideで想定されている「30種類以上のモンスター」をサポートするため、
以下を実装する：

1. `MonsterKind` enum の導入
2. `MonsterDefinition` 構造体によるデータ駆動設計
3. RON/JSONファイルからのモンスター定義読み込み
4. 個別モンスター挙動の拡張可能な設計

---

## 現在の実装との差異

### 現状（プロトタイプ）

```rust
// src/feature/monster/components.rs
#[derive(Component)]
pub struct Monster;

#[derive(Component, Debug, Clone, Copy)]
pub struct MonsterProperty {
    pub base_direction: Direction,
    pub base_speed: f32,
}
```

- すべてのモンスターが同じ挙動
- パラメータはハードコード（`level.rs`で定義）
- 見た目は単一色の四角形

### 目標（データ駆動）

```rust
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MonsterKind {
    Kappa,      // 河童 - 標準的な速度と挙動
    Ghost,      // ゴースト - 高速移動、半透明
    Bakeneko,   // 化け猫 - 大型でゆっくり
    // ... 将来的に30種類以上の妖怪に拡張
}

#[derive(Clone, Debug)]
pub struct MonsterDefinition {
    pub kind: MonsterKind,
    pub speed: f32,
    pub size: f32,
    pub color: (f32, f32, f32),  // 暫定（将来的にテクスチャパスに置き換え）
    pub special_behavior: SpecialBehavior,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecialBehavior {
    None,
    SpeedBoost { multiplier: f32 },
    Teleport { interval: f32 },
    Split { count: usize },
}
```

- モンスター種類ごとに異なる挙動
- 外部ファイル（RON）からパラメータを読み込み
- 拡張可能な特殊挙動システム

---

## 実装ステップ

### Phase 1: MonsterKindの基本実装

#### 1.1 MonsterKindとMonsterDefinitionの導入

**ファイル**: `src/feature/monster/definitions.rs` (新規)

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterKind {
    Kappa,      // 河童
    Ghost,      // ゴースト
    Bakeneko,   // 化け猫
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterDefinition {
    pub kind: MonsterKind,
    pub speed: f32,
    pub size: f32,
    pub color: (f32, f32, f32),
    pub wait_threshold: f32,
}

#[derive(Resource, Default)]
pub struct MonsterDefinitions {
    definitions: HashMap<MonsterKind, MonsterDefinition>,
}

impl MonsterDefinitions {
    pub fn get(&self, kind: MonsterKind) -> &MonsterDefinition {
        self.definitions.get(&kind).expect("Monster definition not found")
    }

    pub fn insert(&mut self, def: MonsterDefinition) {
        self.definitions.insert(def.kind, def);
    }
}
```

**依存関係追加**:

`Cargo.toml` に以下を追加：
```toml
[dependencies]
bevy = "0.17.2"
serde = { version = "1.0", features = ["derive"] }
serde_ron = "0.8"
```

**決定事項**:
- ✅ `serde` と `serde_ron` をCargo.tomlに追加する
- ✅ `MonsterKind` は3種類から始める（Kappa, Ghost, Bakeneko）
- ✅ `MonsterKind` は `feature/monster/definitions.rs` に配置（monster機能内に閉じる）
  - 他のfeatureから参照したい情報は、より抽象化したレイヤー（例: イベント経由）で管理

#### 1.2 MonsterコンポーネントへのMonsterKind追加

**ファイル**: `src/feature/monster/components.rs`

```diff
+// src/feature/monster/mod.rs に追加
+mod definitions;
+pub use definitions::{MonsterKind, MonsterDefinition, MonsterDefinitions};
+
+// src/feature/monster/components.rs
 #[derive(Component)]
 pub struct Monster;

 #[derive(Component, Debug, Clone, Copy)]
 pub struct MonsterProperty {
+    pub kind: MonsterKind,
     pub base_direction: Direction,
     pub base_speed: f32,
+    pub base_size: f32,
 }
```

**決定事項**:
- ✅ `MonsterKind` は `feature/monster/definitions.rs` に配置
- ✅ `MonsterKind` は `Component` として使用（entity に直接付与）

#### 1.3 スポーンシステムの更新

**ファイル**: `src/feature/monster/spawn.rs`

```diff
 #[derive(Debug, Clone)]
 pub struct SpawnDefinition {
+    pub kind: MonsterKind,
     pub direction: Direction,
     pub grid_pos: i32,
     pub delay: f32,
 }

 fn create_spawn_definitions() -> Vec<SpawnDefinition> {
     vec![
-        SpawnDefinition { direction: Direction::Right, grid_pos: 5, delay: 0.0 },
+        SpawnDefinition {
+            kind: MonsterKind::Kappa,
+            direction: Direction::Right,
+            grid_pos: 5,
+            delay: 0.0
+        },
+        SpawnDefinition {
+            kind: MonsterKind::Ghost,
+            direction: Direction::Left,
+            grid_pos: 5,
+            delay: 0.0
+        },
         // ...
     ]
 }

 fn spawn_monster(
     commands: &mut Commands,
     spawn_def: &SpawnDefinition,
+    monster_defs: &MonsterDefinitions,
 ) {
+    let def = monster_defs.get(spawn_def.kind);
     let position = get_staging_position(spawn_def.direction, spawn_def.grid_pos);
-    let monster_size = GRID_SIZE * 0.6;
+    let monster_size = GRID_SIZE * def.size;

     commands.spawn((
         Monster,
+        spawn_def.kind,
         MonsterState::Staging,
-        MonsterProperty::new(spawn_def.direction, level::MONSTER_SPEED),
+        MonsterProperty::new(
+            spawn_def.kind,
+            spawn_def.direction,
+            def.speed,
+            def.size
+        ),
         // ...
         Sprite {
-            color: Color::srgb(MONSTER_COLOR.0, MONSTER_COLOR.1, MONSTER_COLOR.2),
+            color: Color::srgb(def.color.0, def.color.1, def.color.2),
             custom_size: Some(Vec2::splat(monster_size)),
             ..default()
         },
         // ...
     ));
 }
```

**MonsterDefinitionsの初期化**:

**ファイル**: `src/feature/monster/plugin.rs`

```rust
impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MonsterSpawnQueue>()
            .init_resource::<MonsterDefinitions>()  // 追加
            .add_message::<MonsterDespawnEvent>()
            .add_systems(Startup, setup_monster_definitions)  // 追加
            .add_systems(
                Update,
                (
                    spawn_monsters_system,
                    // ...
                )
                    .chain()
                    .run_if(in_state(GameState::InGame))
            );
    }
}

fn setup_monster_definitions(mut definitions: ResMut<MonsterDefinitions>) {
    // Phase 1: ハードコードで定義を登録
    definitions.insert(MonsterDefinition {
        kind: MonsterKind::Kappa,
        speed: 100.0,
        size: 0.6,
        color: (0.2, 0.8, 0.5),
        wait_threshold: 10.0,
    });

    definitions.insert(MonsterDefinition {
        kind: MonsterKind::Ghost,
        speed: 150.0,
        size: 0.5,
        color: (0.9, 0.9, 1.0),
        wait_threshold: 8.0,
    });

    definitions.insert(MonsterDefinition {
        kind: MonsterKind::Bakeneko,
        speed: 70.0,
        size: 0.8,
        color: (0.3, 0.2, 0.4),
        wait_threshold: 15.0,
    });
}

fn spawn_monsters_system(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_queue: ResMut<MonsterSpawnQueue>,
    monster_defs: Res<MonsterDefinitions>,  // 追加
) {
    // 既存のロジック + monster_defs を spawn_monster() に渡す
}
```

**決定事項**:
- ✅ `MonsterDefinitions` を `init_resource()` でリソースとして初期化
- ✅ `Startup` システムでハードコードの定義を登録（Phase 1）
- ✅ Phase 2でRONファイル読み込みに置き換え予定
- ✅ `spawn_monsters_system` に `Res<MonsterDefinitions>` パラメータを追加

---

### Phase 2: RONファイルによるデータ駆動化

#### 2.1 RONファイルの作成

**ファイル**: `assets/monsters.ron` (新規)

```ron
(
    definitions: [
        (
            kind: Kappa,
            speed: 100.0,
            size: 0.6,
            color: (0.2, 0.8, 0.5),  // 緑色（河童）
            wait_threshold: 10.0,
        ),
        (
            kind: Ghost,
            speed: 150.0,
            size: 0.5,
            color: (0.9, 0.9, 1.0),  // 白っぽい（ゴースト）
            wait_threshold: 8.0,
        ),
        (
            kind: Bakeneko,
            speed: 70.0,
            size: 0.8,
            color: (0.3, 0.2, 0.4),  // 紫がかった色（化け猫）
            wait_threshold: 15.0,
        ),
    ]
)
```

**決定事項**:
- ✅ RONファイルのスキーマは暫定案の通りで確定
- ✅ `assets/` ディレクトリは作成済み
- ✅ バリデーション処理は不要（エラー時はパニックで終了）

#### 2.2 RONファイル読み込みシステム

**ファイル**: `src/feature/monster/definitions.rs` (既存ファイルに追加)

```rust
#[derive(Deserialize)]
struct MonsterDefinitionsFile {
    definitions: Vec<MonsterDefinition>,
}

impl MonsterDefinitions {
    /// RONファイルからモンスター定義を読み込む
    /// ファイルが存在しない、または形式が不正な場合はパニックする
    pub fn from_file(path: &str) -> Self {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read monster definitions file '{}': {}", path, e));

        let file: MonsterDefinitionsFile = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse monster definitions file '{}': {}", path, e));

        let mut definitions = HashMap::new();
        for def in file.definitions {
            definitions.insert(def.kind, def);
        }

        Self { definitions }
    }
}
```

**plugin.rs の変更**:

```rust
fn setup_monster_definitions(mut definitions: ResMut<MonsterDefinitions>) {
    // Phase 2: RONファイルから定義を読み込む
    let loaded = MonsterDefinitions::from_file("assets/monsters.ron");
    *definitions = loaded;
}
```

**決定事項**:
- ✅ `std::fs::read_to_string()` を使用してシンプルに読み込む
- ✅ `serde_ron::from_str()` でパースする
- ✅ ロード失敗時は `panic!()` でアプリを終了する
- ✅ Bevyのアセットシステムは使用しない（起動時の同期読み込みで十分）

---

### Phase 3: 特殊挙動システム

#### 3.1 SpecialBehaviorの設計

**ファイル**: `src/feature/monster/special_behavior.rs` (新規)

```rust
#[derive(Component, Clone, Debug, PartialEq)]
pub enum SpecialBehavior {
    None,
    SpeedBoost {
        multiplier: f32,
        activation_distance: f32,  // ゴールまでの距離
    },
    Teleport {
        interval: f32,
        distance: f32,
    },
    Split {
        count: usize,
        trigger_on_collision: bool,
    },
}

fn apply_special_behavior_system(
    mut query: Query<(&MonsterKind, &mut Movement, &Transform, &SpecialBehavior)>,
    monster_defs: Res<MonsterDefinitions>,
    time: Res<Time>,
) {
    for (kind, mut movement, transform, behavior) in &mut query {
        match behavior {
            SpecialBehavior::None => {},
            SpecialBehavior::SpeedBoost { multiplier, activation_distance } => {
                // ゴールまでの距離を計算し、閾値を下回ったら加速
                // TODO: ゴール位置の取得方法
            },
            SpecialBehavior::Teleport { interval, distance } => {
                // 一定間隔でテレポート
                // TODO: タイマー管理
            },
            SpecialBehavior::Split { count, trigger_on_collision } => {
                // 衝突時に分裂
                // TODO: 新しいモンスターのスポーン
            },
        }
    }
}
```

**TODO**:
- [ ] 特殊挙動を何種類実装するか決定（最初は3種類程度？）
- [ ] 各挙動の具体的なゲームデザインを詰める
  - SpeedBoost: どのタイミングで発動？加速率は？
  - Teleport: テレポート先はランダム？固定方向？
  - Split: 分裂後のモンスターの種類は？
- [ ] `SpecialBehavior` を `MonsterDefinition` に含めるか、別コンポーネントにするか
  - **提案**: 別コンポーネント（動的に変更可能にするため）

#### 3.2 個別モンスター挙動の分離

**ファイル**: `src/feature/monster/types/` (新規ディレクトリ)

```text
monster/types/
├─ mod.rs
├─ kappa.rs
├─ ghost.rs
└─ bakeneko.rs
```

各ファイルには、そのモンスター固有のシステムを実装。

```rust
// kappa.rs
pub fn kappa_behavior_system(
    query: Query<&MonsterKind, With<Monster>>,
) {
    // 河童固有の挙動（例：水辺で速度アップ）
}
```

**TODO**:
- [ ] 個別挙動が必要なモンスターをリストアップ
- [ ] `types/` ディレクトリを作るタイミング（Phase 3で必須か？Phase 4以降でもよいか？）
- [ ] 個別挙動システムの登録方法（MonsterPluginで全て登録？動的登録？）

---

### Phase 4: スケールアップと最適化

#### 4.1 モンスター種類の大量追加

**目標**: 30種類以上のモンスターを定義

**作業内容**:
1. `MonsterKind` に30個のバリアントを追加
2. `assets/monsters.ron` に30個の定義を記述
3. 各モンスターのゲームバランスを調整

**TODO**:
- [ ] 30種類のモンスターのコンセプトリストを作成
- [ ] 挙動のバリエーション（速度、サイズ、特殊能力）をどう差別化するか
- [ ] テクスチャ/スプライトの導入（現在は色だけ）

#### 4.2 Wave定義のデータ駆動化

**ファイル**: `assets/waves.ron` (新規)

```ron
(
    waves: [
        (
            monsters: [
                (kind: Kappa, direction: Right, grid_pos: 5, delay: 0.0),
                (kind: Kappa, direction: Left, grid_pos: 5, delay: 0.0),
            ],
        ),
        (
            monsters: [
                (kind: Ghost, direction: Down, grid_pos: 3, delay: 0.0),
                (kind: Bakeneko, direction: Up, grid_pos: 7, delay: 1.0),
            ],
        ),
    ]
)
```

**TODO**:
- [ ] Wave定義のスキーマ設計
- [ ] 現在の `create_spawn_definitions()` をRONファイル読み込みに置き換え
- [ ] ステージ/レベル概念の導入（複数のWaveファイル？）

#### 4.3 パフォーマンス最適化

**検討事項**:
- オブジェクトプール（dev_guide 9.2参照）
- 空間分割（100体以上の同時表示時）
- システム並列化（現在は`.chain()`で直列実行）

**TODO**:
- [ ] パフォーマンス測定基準の設定（目標FPS、最大モンスター数）
- [ ] プロファイリング実施（`bevy_diagnostic`の活用）
- [ ] 最適化の優先順位付け

---

## 技術的課題とTODO

### 依存関係の追加

**Cargo.toml**:
```toml
[dependencies]
bevy = "0.17.2"
serde = { version = "1.0", features = ["derive"] }
serde_ron = "0.8"  # RONファイル読み込み用
```

**決定事項**:
- ✅ `serde = { version = "1.0", features = ["derive"] }` を使用（Bevy 0.17と互換性あり）
- ✅ `ron = "0.8"` を使用（最新の安定版）

### Bevy 0.17でのアセット読み込み

**決定事項**:
- ✅ Bevyのアセットシステムは使用せず、`std::fs::read_to_string()` で直接読み込む
- ✅ `AssetLoader` トレイトの実装は不要
- ✅ 起動時に同期読み込みで十分（ゲームプレイに影響なし）
- ✅ エラーハンドリングは `panic!()` でアプリ終了

### MonsterKindの配置場所

**決定事項**:
- ✅ `feature/monster/definitions.rs` に配置（monster機能内に閉じる）
- ✅ 他のfeatureからの参照が必要になった場合は、イベント経由で情報を渡す設計を採用

### 特殊挙動の実装順序

**優先度**:
1. **High**: `SpecialBehavior::None` - 基本実装（Phase 1で対応）
2. **Medium**: `SpeedBoost` - 実装が比較的シンプル
3. **Low**: `Teleport`, `Split` - 複雑な処理が必要

**TODO**:
- [ ] Phase 3で実装する挙動を1〜2種類に絞る
- [ ] 残りはPhase 4以降に延期

### テスト戦略

**検討事項**:
- 単体テスト：`MonsterDefinitions` のロード処理
- 統合テスト：各MonsterKindが正しくスポーンされるか
- デバッグ機能：特定のMonsterKindを強制スポーンするコマンド

**TODO**:
- [ ] テストの必要性を検討（小規模プロジェクトでどこまでやるか）
- [ ] デバッグ用のスポーンコマンド（F1キーで特定モンスター生成など）

---

## スケジュールと優先度

### 推奨実装順序

1. **Phase 1** (高優先度)
   - MonsterKindの基本実装
   - ハードコードで3種類のモンスターを動かす
   - 見た目の差別化（色、サイズ）

2. **Phase 2** (中優先度)
   - RONファイルからの読み込み
   - `assets/monsters.ron` の作成
   - データ駆動化の実現

3. **Phase 3** (中優先度)
   - 特殊挙動システムの基盤構築
   - 1〜2種類の特殊挙動を実装

4. **Phase 4** (低優先度)
   - 30種類へのスケールアップ
   - Wave定義のデータ駆動化
   - パフォーマンス最適化

### マイルストーン

- **Milestone 1**: Phase 1完了 - 3種類のモンスターが異なる見た目・速度で動く
- **Milestone 2**: Phase 2完了 - RONファイルから定義を読み込める
- **Milestone 3**: Phase 3完了 - 特殊挙動を持つモンスターが1種類以上動く
- **Milestone 4**: Phase 4完了 - 30種類のモンスターとデータ駆動Wave管理

---

## 参考資料

- [dev_guide.md セクション4](../development/dev_guide.md#4-敵キャラクター構成指針)
- [dev_guide.md セクション9](../development/dev_guide.md#9-大量モンスター管理方針)
- Bevy公式ドキュメント: https://docs.rs/bevy/0.17.2/
- RON形式仕様: https://github.com/ron-rs/ron

---

## 変更履歴

- 2025-11-14: 初版作成
