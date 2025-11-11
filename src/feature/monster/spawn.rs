use bevy::prelude::*;
use crate::core::{Direction, GRID_SIZE, FIELD_WIDTH, FIELD_HEIGHT, MONSTER_SPEED, MONSTER_COLOR};
use super::components::*;

/// モンスターのスポーン定義
#[derive(Debug, Clone)]
pub struct SpawnDefinition {
    pub direction: Direction,
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
fn create_spawn_definitions() -> Vec<SpawnDefinition> {
    vec![
        SpawnDefinition { direction: Direction::Right, delay: 0.0 },
        SpawnDefinition { direction: Direction::Left, delay: 1.0 },
        SpawnDefinition { direction: Direction::Down, delay: 2.0 },
        SpawnDefinition { direction: Direction::Up, delay: 2.5 },
        SpawnDefinition { direction: Direction::Right, delay: 3.0 },
        SpawnDefinition { direction: Direction::Left, delay: 3.5 },
        SpawnDefinition { direction: Direction::Down, delay: 4.0 },
        SpawnDefinition { direction: Direction::Up, delay: 4.5 },
        SpawnDefinition { direction: Direction::Right, delay: 5.0 },
        SpawnDefinition { direction: Direction::Left, delay: 5.5 },
    ]
}

/// 画面端の待機位置を取得
fn get_staging_position(direction: Direction) -> Vec3 {
    let field_width = FIELD_WIDTH as f32 * GRID_SIZE;
    let field_height = FIELD_HEIGHT as f32 * GRID_SIZE;
    let margin = GRID_SIZE * 1.5;

    match direction {
        Direction::Right => Vec3::new(-field_width / 2.0 - margin, 0.0, 0.0),
        Direction::Left => Vec3::new(field_width / 2.0 + margin, 0.0, 0.0),
        Direction::Up => Vec3::new(0.0, -field_height / 2.0 - margin, 0.0),
        Direction::Down => Vec3::new(0.0, field_height / 2.0 + margin, 0.0),
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
    let position = get_staging_position(spawn_def.direction);
    let monster_size = GRID_SIZE * 0.6;

    commands.spawn((
        Monster,
        MonsterState::Staging,
        Movement::new(spawn_def.direction, MONSTER_SPEED),
        StagingTimer::new(2.0), // 2秒待機
        CollisionBox::new(Vec2::splat(monster_size)),
        Sprite {
            color: Color::srgb(MONSTER_COLOR.0, MONSTER_COLOR.1, MONSTER_COLOR.2),
            custom_size: Some(Vec2::splat(monster_size)),
            ..default()
        },
        Transform::from_translation(position),
    ));

    info!("Spawned monster at {:?} facing {:?}", position, spawn_def.direction);
}
