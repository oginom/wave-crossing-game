use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::{Direction, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT, grid_to_world, GridPosition};
use crate::core::level;
use super::components::*;
use super::definitions::{MonsterDefinitions, MonsterKind};
use super::special_behavior::{SpecialBehavior, MyPaceTimer};

/// モンスターのスポーン定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnDefinition {
    pub kind: MonsterKind,
    pub direction: Direction,
    /// スポーン位置（進行方向に垂直な軸の座標）
    /// Right/Leftの場合はy座標、Up/Downの場合はx座標を指定
    pub grid_pos: i32,
    pub delay: f32,
}

/// Wave定義（特定の時刻に出現するモンスターのグループ）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveDefinition {
    /// Wave開始時間（ゲーム開始からの経過時間・秒）
    pub start_time: f32,
    /// このWaveでスポーンするモンスターのリスト
    pub monsters: Vec<SpawnDefinition>,
}

/// ステージレベル定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageLevel {
    pub stage: u32,
    pub level: u32,
    pub waves: Vec<WaveDefinition>,
}

/// ステージレベルファイルの構造
#[derive(Deserialize)]
struct StageLevelFile {
    stage: u32,
    level: u32,
    waves: Vec<WaveDefinition>,
}

impl StageLevel {
    /// RONファイルからステージレベル定義を読み込む
    pub fn from_file(path: &str) -> Self {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read stage level file '{}': {}", path, e));

        let file: StageLevelFile = ron::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse stage level file '{}': {}", path, e));

        Self {
            stage: file.stage,
            level: file.level,
            waves: file.waves,
        }
    }
}

/// モンスターのスポーンキュー（リソース）
#[derive(Resource)]
pub struct MonsterSpawnQueue {
    pub spawns: Vec<SpawnDefinition>,
    pub waves: Vec<WaveDefinition>,
    pub timer: f32,
    pub processed_wave_indices: Vec<usize>,  // 処理済みWaveのインデックス
}

impl Default for MonsterSpawnQueue {
    fn default() -> Self {
        // RONファイルからWave定義を読み込む
        let stage_level = StageLevel::from_file("assets/stages/stage1_level1.ron");

        Self {
            spawns: Vec::new(),
            waves: stage_level.waves,
            timer: 0.0,
            processed_wave_indices: Vec::new(),
        }
    }
}


/// 画面端の待機位置を取得
fn get_staging_position(direction: Direction, grid_pos: i32) -> Vec3 {
    let field_width = FIELD_WIDTH as f32 * GRID_SIZE;
    let field_height = FIELD_HEIGHT as f32 * GRID_SIZE;
    let margin = GRID_SIZE * 1.5;

    match direction {
        Direction::Right => {
            // 左端から右に向かって進入、grid_posはy座標
            let grid_position = GridPosition::new(0, grid_pos);
            let world_pos = grid_to_world(grid_position, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT);
            Vec3::new(-field_width / 2.0 - margin, world_pos.y, 0.0)
        }
        Direction::Left => {
            // 右端から左に向かって進入、grid_posはy座標
            let grid_position = GridPosition::new(0, grid_pos);
            let world_pos = grid_to_world(grid_position, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT);
            Vec3::new(field_width / 2.0 + margin, world_pos.y, 0.0)
        }
        Direction::Up => {
            // 下端から上に向かって進入、grid_posはx座標
            let grid_position = GridPosition::new(grid_pos, 0);
            let world_pos = grid_to_world(grid_position, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT);
            Vec3::new(world_pos.x, -field_height / 2.0 - margin, 0.0)
        }
        Direction::Down => {
            // 上端から下に向かって進入、grid_posはx座標
            let grid_position = GridPosition::new(grid_pos, 0);
            let world_pos = grid_to_world(grid_position, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT);
            Vec3::new(world_pos.x, field_height / 2.0 + margin, 0.0)
        }
    }
}

/// モンスターをスポーンするシステム
pub fn spawn_monsters_system(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_queue: ResMut<MonsterSpawnQueue>,
    monster_defs: Res<MonsterDefinitions>,
    asset_server: Res<AssetServer>,
) {
    spawn_queue.timer += time.delta_secs();

    // Wave開始時間を確認して、新しいWaveのモンスターをスポーンキューに追加
    let mut newly_processed_waves = Vec::new();
    let mut new_spawns = Vec::new();

    for (index, wave) in spawn_queue.waves.iter().enumerate() {
        // すでに処理済みのWaveはスキップ
        if spawn_queue.processed_wave_indices.contains(&index) {
            continue;
        }

        // Wave開始時間に達したか確認
        if spawn_queue.timer >= wave.start_time {
            // このWaveのモンスターをスポーンキューに追加
            for monster_spawn in &wave.monsters {
                let mut spawn_def = monster_spawn.clone();
                // delayはWave開始時間からの相対時間なので、絶対時間に変換
                spawn_def.delay = wave.start_time + monster_spawn.delay;
                new_spawns.push(spawn_def);
            }
            newly_processed_waves.push(index);
            info!("Wave {} started at {:.2}s", index, spawn_queue.timer);
        }
    }

    // 新しいスポーンをキューに追加
    spawn_queue.spawns.extend(new_spawns);

    // 処理済みWaveをマーク
    spawn_queue.processed_wave_indices.extend(newly_processed_waves);

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

/// モンスターをスポーン
fn spawn_monster(
    commands: &mut Commands,
    spawn_def: &SpawnDefinition,
    monster_defs: &MonsterDefinitions,
    asset_server: &AssetServer,
) {
    let def = monster_defs.get(spawn_def.kind);
    let position = get_staging_position(spawn_def.direction, spawn_def.grid_pos);
    let monster_size = GRID_SIZE * def.size;

    // テクスチャを読み込む
    let texture_handle: Handle<Image> = asset_server.load(&def.texture_path);

    let mut entity_commands = commands.spawn((
        Monster,
        spawn_def.kind,
        MonsterState::Staging,
        MonsterProperty::new(
            spawn_def.kind,
            spawn_def.direction,
            def.speed,
            def.size,
            def.color,
        ),
        Movement::new(spawn_def.direction, def.speed),
        StagingTimer::new(level::STAGING_DURATION),
        CollisionBox::new(Vec2::splat(monster_size)),
        CollisionState::new(),
        WaitMeter::new(def.wait_threshold),
        def.special_behavior.clone(),  // SpecialBehaviorコンポーネントを追加
        Sprite {
            image: texture_handle,
            color: Color::WHITE,  // テクスチャ本来の色を表示（乗算で白=そのまま表示）
            custom_size: Some(Vec2::splat(monster_size)),
            ..default()
        },
        Transform::from_translation(position),
    ));

    // MyPace挙動の場合は、MyPaceTimerコンポーネントを追加
    if let SpecialBehavior::MyPace { stop_interval, stop_duration } = def.special_behavior {
        entity_commands.insert(MyPaceTimer::new(stop_interval, stop_duration));
    }

    info!(
        "Spawned {:?} at {:?} (grid: {}) facing {:?}",
        spawn_def.kind, position, spawn_def.grid_pos, spawn_def.direction
    );
}
