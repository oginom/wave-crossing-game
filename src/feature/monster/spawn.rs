use bevy::prelude::*;
use crate::core::{Direction, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT, grid_to_world, GridPosition};
use crate::core::level;
use super::components::*;
use super::definitions::{MonsterDefinitions, MonsterKind};

/// モンスターのスポーン定義
#[derive(Debug, Clone)]
pub struct SpawnDefinition {
    pub kind: MonsterKind,
    pub direction: Direction,
    /// スポーン位置（進行方向に垂直な軸の座標）
    /// Right/Leftの場合はy座標、Up/Downの場合はx座標を指定
    pub grid_pos: i32,
    pub delay: f32,
}

/// モンスターのスポーンキュー（リソース）
#[derive(Resource)]
pub struct MonsterSpawnQueue {
    pub spawns: Vec<SpawnDefinition>,
    pub timer: f32,
}

impl Default for MonsterSpawnQueue {
    fn default() -> Self {
        Self {
            spawns: create_spawn_definitions(),
            timer: 0.0,
        }
    }
}

/// スポーン定義を作成（10匹のモンスター、3種類の妖怪）
/// 衝突テスト用に配置を調整
fn create_spawn_definitions() -> Vec<SpawnDefinition> {
    vec![
        // 河童: 正面衝突テスト: 同じy座標5で左右から
        SpawnDefinition { kind: MonsterKind::Kappa, direction: Direction::Right, grid_pos: 5, delay: 0.0 },
        SpawnDefinition { kind: MonsterKind::Kappa, direction: Direction::Left, grid_pos: 5, delay: 0.0 },

        // ゴースト: 直角衝突テスト: 異なるタイミングで交差
        SpawnDefinition { kind: MonsterKind::Ghost, direction: Direction::Down, grid_pos: 3, delay: 1.0 },
        SpawnDefinition { kind: MonsterKind::Ghost, direction: Direction::Right, grid_pos: 3, delay: 1.5 },

        // 化け猫: 正面衝突テスト2: 同じx座標7で上下から
        SpawnDefinition { kind: MonsterKind::Bakeneko, direction: Direction::Up, grid_pos: 7, delay: 2.0 },
        SpawnDefinition { kind: MonsterKind::Bakeneko, direction: Direction::Down, grid_pos: 7, delay: 2.0 },

        // 単独移動（衝突なし）- 様々な種類をテスト
        SpawnDefinition { kind: MonsterKind::Kappa, direction: Direction::Right, grid_pos: 1, delay: 3.0 },
        SpawnDefinition { kind: MonsterKind::Ghost, direction: Direction::Left, grid_pos: 9, delay: 3.5 },
        SpawnDefinition { kind: MonsterKind::Bakeneko, direction: Direction::Up, grid_pos: 2, delay: 4.0 },
        SpawnDefinition { kind: MonsterKind::Kappa, direction: Direction::Down, grid_pos: 8, delay: 4.5 },
    ]
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
) {
    spawn_queue.timer += time.delta_secs();

    // スポーン予定のモンスターをチェック
    let mut spawned_indices = Vec::new();
    for (index, spawn_def) in spawn_queue.spawns.iter().enumerate() {
        if spawn_queue.timer >= spawn_def.delay {
            spawn_monster(&mut commands, spawn_def, &monster_defs);
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
) {
    let def = monster_defs.get(spawn_def.kind);
    let position = get_staging_position(spawn_def.direction, spawn_def.grid_pos);
    let monster_size = GRID_SIZE * def.size;

    commands.spawn((
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
        Sprite {
            color: Color::srgb(def.color.0, def.color.1, def.color.2),
            custom_size: Some(Vec2::splat(monster_size)),
            ..default()
        },
        Transform::from_translation(position),
    ));

    info!(
        "Spawned {:?} at {:?} (grid: {}) facing {:?}",
        spawn_def.kind, position, spawn_def.grid_pos, spawn_def.direction
    );
}
