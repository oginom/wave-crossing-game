# 02 Game States: Wait System & Spirit/Void Gauges

## 概要

モンスターの待機システムと、プレイヤーの状態を管理する2つのゲージ（魂/虚）を実装する。
モンスターが長時間停止すると消滅し、ゴール到達と消滅がプレイヤーのゲージに影響を与える。

## 実装要素

### 1. モンスター待機システム

#### 1.1 Wait値の管理

**コンポーネント追加**

```rust
// src/feature/monster/components.rs

#[derive(Component, Default)]
pub struct WaitMeter {
    /// 現在の待機時間（秒）
    pub current: f32,
    /// 消滅する待機時間の閾値（秒）
    pub threshold: f32,
    /// 前フレームでの移動状態（速度がゼロかどうか）
    pub was_stopped: bool,
}

impl WaitMeter {
    pub fn new(threshold: f32) -> Self {
        Self {
            current: 0.0,
            threshold,
            was_stopped: false,
        }
    }

    pub fn progress_ratio(&self) -> f32 {
        (self.current / self.threshold).min(1.0)
    }

    pub fn is_expired(&self) -> bool {
        self.current >= self.threshold
    }
}
```

#### 1.2 Wait値の更新システム

```rust
// src/feature/monster/wait.rs

use bevy::prelude::*;
use super::components::{Monster, WaitMeter};
use super::movement::Velocity;

const WAIT_SPEED_THRESHOLD: f32 = 0.1; // この速度以下で「停止」とみなす

/// モンスターの待機時間を更新
pub fn update_wait_meter_system(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut WaitMeter), With<Monster>>,
) {
    for (velocity, mut wait_meter) in query.iter_mut() {
        let is_stopped = velocity.x.abs() < WAIT_SPEED_THRESHOLD
                      && velocity.y.abs() < WAIT_SPEED_THRESHOLD;

        if is_stopped {
            // 停止中：wait値を増加
            wait_meter.current += time.delta_seconds();
        } else {
            // 移動中：wait値をリセット
            if wait_meter.current > 0.0 {
                wait_meter.current = 0.0;
            }
        }

        wait_meter.was_stopped = is_stopped;
    }
}
```

#### 1.3 色の変化システム

```rust
// src/feature/monster/wait.rs

/// wait値に応じてモンスターの色を変化させる
pub fn update_monster_color_system(
    mut query: Query<(&WaitMeter, &mut Sprite), With<Monster>>,
) {
    for (wait_meter, mut sprite) in query.iter_mut() {
        let ratio = wait_meter.progress_ratio();

        // 待機時間が進むほど黒くなる（RGB値を減少）
        let brightness = 1.0 - (ratio * 0.7); // 最大で30%の明るさまで落ちる
        sprite.color = Color::srgb(brightness, brightness, brightness);
    }
}
```

#### 1.4 消滅イベントと処理

```rust
// src/feature/monster/events.rs

use bevy::prelude::*;

#[derive(Event)]
pub struct MonsterDespawnEvent {
    pub entity: Entity,
    pub cause: DespawnCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DespawnCause {
    ReachedGoal,
    WaitExpired,
}
```

```rust
// src/feature/monster/wait.rs

use super::events::{MonsterDespawnEvent, DespawnCause};

/// wait値が閾値を超えたモンスターを消滅させる
pub fn despawn_expired_monsters_system(
    mut commands: Commands,
    query: Query<(Entity, &WaitMeter), With<Monster>>,
    mut despawn_events: EventWriter<MonsterDespawnEvent>,
) {
    for (entity, wait_meter) in query.iter() {
        if wait_meter.is_expired() {
            // イベントを発行
            despawn_events.send(MonsterDespawnEvent {
                entity,
                cause: DespawnCause::WaitExpired,
            });

            // エンティティを削除
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

---

### 2. プレイヤーゲージシステム

#### 2.1 ゲージリソース

```rust
// src/feature/player/gauges.rs

use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct PlayerGauges {
    pub spirit: SpiritGauge,
    pub void: VoidGauge,
}

impl Default for PlayerGauges {
    fn default() -> Self {
        Self {
            spirit: SpiritGauge::new(100.0, 50.0), // max=100, initial=50
            void: VoidGauge::new(100.0),           // max=100, initial=0
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpiritGauge {
    pub current: f32,
    pub max: f32,
}

impl SpiritGauge {
    pub fn new(max: f32, initial: f32) -> Self {
        Self {
            current: initial.min(max),
            max,
        }
    }

    pub fn add(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn consume(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    pub fn ratio(&self) -> f32 {
        self.current / self.max
    }
}

#[derive(Debug, Clone)]
pub struct VoidGauge {
    pub current: f32,
    pub max: f32,
}

impl VoidGauge {
    pub fn new(max: f32) -> Self {
        Self {
            current: 0.0,
            max,
        }
    }

    pub fn add(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    pub fn ratio(&self) -> f32 {
        self.current / self.max
    }
}
```

#### 2.2 ゲージ更新システム

```rust
// src/feature/player/gauges.rs

use crate::feature::monster::events::{MonsterDespawnEvent, DespawnCause};

const SPIRIT_GAIN_PER_GOAL: f32 = 10.0;
const VOID_GAIN_PER_DESPAWN: f32 = 5.0;

/// モンスター消滅イベントを受けてゲージを更新
pub fn update_gauges_on_monster_event_system(
    mut gauges: ResMut<PlayerGauges>,
    mut events: EventReader<MonsterDespawnEvent>,
) {
    for event in events.read() {
        match event.cause {
            DespawnCause::ReachedGoal => {
                gauges.spirit.add(SPIRIT_GAIN_PER_GOAL);
                info!("Spirit +{}: {:.1}/{:.1}",
                      SPIRIT_GAIN_PER_GOAL,
                      gauges.spirit.current,
                      gauges.spirit.max);
            }
            DespawnCause::WaitExpired => {
                gauges.void.add(VOID_GAIN_PER_DESPAWN);
                info!("Void +{}: {:.1}/{:.1}",
                      VOID_GAIN_PER_DESPAWN,
                      gauges.void.current,
                      gauges.void.max);
            }
        }
    }
}
```

#### 2.3 ゲームオーバー判定

```rust
// src/feature/player/gauges.rs

use crate::GameState;

/// Voidゲージが満タンになったらゲームオーバー
pub fn check_game_over_system(
    gauges: Res<PlayerGauges>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if gauges.void.is_full() {
        warn!("Game Over: Void gauge is full");
        next_state.set(GameState::GameOver);
    }
}
```

---

### 3. UI表示

#### 3.1 ゲージUI構造

```rust
// src/feature/ui/gauges.rs

use bevy::prelude::*;

#[derive(Component)]
pub struct GaugesPanel;

#[derive(Component)]
pub struct SpiritGaugeBar;

#[derive(Component)]
pub struct VoidGaugeBar;

#[derive(Component)]
pub struct GaugeText;
```

#### 3.2 UI生成

```rust
// src/feature/ui/gauges.rs

const GAUGE_WIDTH: f32 = 200.0;
const GAUGE_HEIGHT: f32 = 30.0;
const GAUGE_MARGIN: f32 = 10.0;

/// 画面右下にゲージUIを生成
pub fn setup_gauges_ui_system(mut commands: Commands) {
    commands
        .spawn((
            GaugesPanel,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(GAUGE_MARGIN),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // Spirit ゲージ
            create_gauge_row(
                parent,
                "Spirit",
                SpiritGaugeBar,
                Color::srgb(0.2, 0.8, 1.0), // 青系
            );

            // Void ゲージ
            create_gauge_row(
                parent,
                "Void",
                VoidGaugeBar,
                Color::srgb(0.8, 0.2, 0.8), // 紫系
            );
        });
}

fn create_gauge_row<T: Component>(
    parent: &mut ChildBuilder,
    label: &str,
    marker: T,
    color: Color,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            ..default()
        })
        .with_children(|row| {
            // ラベル
            row.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            // ゲージ背景
            row.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(GAUGE_WIDTH),
                    height: Val::Px(GAUGE_HEIGHT),
                    ..default()
                },
                background_color: Color::srgb(0.2, 0.2, 0.2).into(),
                ..default()
            })
            .with_children(|bg| {
                // ゲージバー
                bg.spawn((
                    marker,
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(0.0), // 初期値
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        background_color: color.into(),
                        ..default()
                    },
                ));
            });

            // 数値表示
            row.spawn((
                GaugeText,
                TextBundle::from_section(
                    "0/100",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
            ));
        });
}
```

#### 3.3 ゲージUI更新

```rust
// src/feature/ui/gauges.rs

use crate::feature::player::gauges::PlayerGauges;

/// Spiritゲージの表示を更新
pub fn update_spirit_gauge_ui_system(
    gauges: Res<PlayerGauges>,
    mut query: Query<&mut Style, With<SpiritGaugeBar>>,
) {
    for mut style in query.iter_mut() {
        style.width = Val::Percent(gauges.spirit.ratio() * 100.0);
    }
}

/// Voidゲージの表示を更新
pub fn update_void_gauge_ui_system(
    gauges: Res<PlayerGauges>,
    mut query: Query<&mut Style, With<VoidGaugeBar>>,
) {
    for mut style in query.iter_mut() {
        style.width = Val::Percent(gauges.void.ratio() * 100.0);
    }
}
```

---

### 4. プラグイン統合

#### 4.1 MonsterPlugin更新

```rust
// src/feature/monster/plugin.rs

use super::wait::*;
use super::events::MonsterDespawnEvent;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MonsterDespawnEvent>()
            .add_systems(
                Update,
                (
                    update_wait_meter_system,
                    update_monster_color_system,
                    despawn_expired_monsters_system,
                    // ... 既存システム
                )
                .run_if(in_state(GameState::InGame)),
            );
    }
}
```

#### 4.2 PlayerPlugin拡張

```rust
// src/feature/player/plugin.rs

use super::gauges::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerGauges>()
            .add_systems(
                Update,
                (
                    update_gauges_on_monster_event_system,
                    check_game_over_system,
                )
                .run_if(in_state(GameState::InGame)),
            );
    }
}
```

#### 4.3 UIPlugin拡張

```rust
// src/feature/ui/plugin.rs

use super::gauges::*;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), setup_gauges_ui_system)
            .add_systems(
                Update,
                (
                    update_spirit_gauge_ui_system,
                    update_void_gauge_ui_system,
                )
                .run_if(in_state(GameState::InGame)),
            );
    }
}
```

---

### 5. GameState拡張

```rust
// src/lib.rs または src/core/state.rs

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
    Paused,
    GameOver, // 追加
}
```

---

## 実装順序

1. **Phase 1: Wait System** ✅ **完了**
   - [x] `WaitMeter` コンポーネント追加
   - [x] `update_wait_meter_system` 実装
   - [x] `update_monster_color_system` 実装
   - [x] `MonsterDespawnEvent` 定義
   - [x] `despawn_expired_monsters_system` 実装

2. **Phase 2: Gauges Logic** ✅ **完了**
   - [x] `PlayerGauges` リソース実装
   - [x] `SpiritGauge` / `VoidGauge` 実装
   - [x] `update_gauges_on_monster_event_system` 実装
   - [x] ゴール到達時のイベント発行処理追加（既存システムに統合）
   - [x] `check_game_over_system` 実装
   - [x] `GameState::GameOver` 追加

3. **Phase 3: UI Display**
   - [ ] `setup_gauges_ui_system` 実装
   - [ ] `update_spirit_gauge_ui_system` 実装
   - [ ] `update_void_gauge_ui_system` 実装

4. **Phase 4: Integration & Testing**
   - [x] プラグインに全システムを登録（Phase 1: Wait System）
   - [x] プラグインに全システムを登録（Phase 2: Gauges Logic）
   - [ ] プラグインに全システムを登録（Phase 3: UI Display）
   - [x] 動作確認（wait値の増減、色変化、消滅）
   - [ ] ゲージ表示の確認
   - [ ] ゲームオーバー遷移の確認

---

## 調整パラメータ

```rust
// src/feature/monster/config.rs

pub const WAIT_THRESHOLD: f32 = 10.0; // 消滅までの秒数
pub const WAIT_SPEED_THRESHOLD: f32 = 0.1; // 停止判定の速度閾値

// src/feature/player/gauges.rs

pub const SPIRIT_MAX: f32 = 100.0;
pub const SPIRIT_INITIAL: f32 = 50.0;
pub const SPIRIT_GAIN_PER_GOAL: f32 = 10.0;

pub const VOID_MAX: f32 = 100.0;
pub const VOID_GAIN_PER_DESPAWN: f32 = 5.0;
```

---

## 注意点

- **衝突判定との連携**: モンスターが停止する条件は、衝突検出システムで速度がゼロになることを前提とする
- **ゴール判定**: 既存のゴール判定システムで `MonsterDespawnEvent` を発行する必要がある
- **パフォーマンス**: 色変化システムは全モンスターに対して毎フレーム実行されるため、大量のモンスターが存在する場合は最適化を検討
- **UI座標系**: Bevy 0.17+ では UI座標系が変更されているため、適切な座標指定が必要

---

## 将来の拡張

- アイテム使用時の魂消費処理
- ゲージに応じたビジュアルエフェクト（警告表示など）
- ゲームオーバー画面の実装
- wait値が溜まる際のSE/パーティクルエフェクト
- ゲージ回復アイテムの追加