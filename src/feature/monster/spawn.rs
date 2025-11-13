use bevy::prelude::*;
use crate::core::{Direction, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT, MONSTER_SPEED, MONSTER_COLOR, grid_to_world, GridPosition};
use super::components::*;

/// モンスターのスポーン定義
#[derive(Debug, Clone)]
pub struct SpawnDefinition {
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

/// スポーン定義を作成（10匹のモンスター）
/// 衝突テスト用に配置を調整
fn create_spawn_definitions() -> Vec<SpawnDefinition> {
    vec![
        // 正面衝突テスト: 同じy座標5で左右から
        SpawnDefinition { direction: Direction::Right, grid_pos: 5, delay: 0.0 },
        SpawnDefinition { direction: Direction::Left, grid_pos: 5, delay: 0.0 },

        // 直角衝突テスト: 異なるタイミングで交差
        SpawnDefinition { direction: Direction::Down, grid_pos: 3, delay: 1.0 },
        SpawnDefinition { direction: Direction::Right, grid_pos: 3, delay: 1.5 },

        // 正面衝突テスト2: 同じx座標7で上下から
        SpawnDefinition { direction: Direction::Up, grid_pos: 7, delay: 2.0 },
        SpawnDefinition { direction: Direction::Down, grid_pos: 7, delay: 2.0 },

        // 単独移動（衝突なし）
        SpawnDefinition { direction: Direction::Right, grid_pos: 1, delay: 3.0 },
        SpawnDefinition { direction: Direction::Left, grid_pos: 9, delay: 3.5 },
        SpawnDefinition { direction: Direction::Up, grid_pos: 2, delay: 4.0 },
        SpawnDefinition { direction: Direction::Down, grid_pos: 8, delay: 4.5 },
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
) {
    spawn_queue.timer += time.delta_secs();

    // スポーン予定のモンスターをチェック
    let mut spawned_indices = Vec::new();
    for (index, spawn_def) in spawn_queue.spawns.iter().enumerate() {
        if spawn_queue.timer >= spawn_def.delay {
            spawn_monster(&mut commands, spawn_def);
            spawned_indices.push(index);
        }
    }

    // スポーン済みの定義を削除
    for index in spawned_indices.iter().rev() {
        spawn_queue.spawns.remove(*index);
    }
}

/// モンスターをスポーン
fn spawn_monster(commands: &mut Commands, spawn_def: &SpawnDefinition) {
    let position = get_staging_position(spawn_def.direction, spawn_def.grid_pos);
    let monster_size = GRID_SIZE * 0.6;

    commands.spawn((
        Monster,
        MonsterState::Staging,
        MonsterProperty::new(spawn_def.direction, MONSTER_SPEED),
        Movement::new(spawn_def.direction, MONSTER_SPEED),
        StagingTimer::new(2.0), // 2秒待機
        CollisionBox::new(Vec2::splat(monster_size)),
        CollisionState::new(),
        WaitMeter::new(10.0), // 10秒で消滅
        Sprite {
            color: Color::srgb(MONSTER_COLOR.0, MONSTER_COLOR.1, MONSTER_COLOR.2),
            custom_size: Some(Vec2::splat(monster_size)),
            ..default()
        },
        Transform::from_translation(position),
    ));

    info!("Spawned monster at {:?} (grid: {}) facing {:?}", position, spawn_def.grid_pos, spawn_def.direction);
}
