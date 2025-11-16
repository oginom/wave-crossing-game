# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a 2D grid-based game built with **Bevy 0.17.2** where players navigate monsters across intersections. The game is an entry for GitHub Game Off 2025 (theme: WAVES). It uses an ECS architecture with a data-driven design for monster definitions and stage configurations loaded from RON files.

## Development Commands

### Running the Game

```bash
# Development mode (with dynamic linking for faster compilation)
just dev

# Standard cargo run
cargo run
```

### WebGL/WASM Builds

```bash
# Build for WebGL (optimized for size)
just build-wasm

# Build for WebGL (debug, faster build)
just build-wasm-dev

# Build and serve locally at http://localhost:8000
just serve-wasm
```

### Standard Cargo Commands

```bash
# Build
cargo build

# Run tests
cargo test

# Check compilation
cargo check
```

## Architecture

### Plugin-Based Structure

The codebase follows a strict plugin-oriented design. All features are isolated plugins registered in [lib.rs](src/lib.rs):

- **StageAssetPlugin**: Custom asset loaders for RON files
- **WorldPlugin**: Grid rendering and field setup
- **MonsterPlugin**: Monster spawning, movement, collision, and lifecycle
- **ItemPlugin**: Item placement and effects (e.g., rotation tiles)
- **PlayerPlugin**: Player gauges (Spirit/Void)

### Directory Layout

```
src/
├── core/           # Shared infrastructure
│   ├── types.rs    # GridPosition, Direction, coordinate conversion
│   ├── config.rs   # Technical constants (GRID_SIZE, FIELD_WIDTH)
│   ├── level.rs    # Game balance values (MONSTER_SPEED, WAIT_THRESHOLD)
│   └── stage_asset.rs  # Custom RON asset loaders
└── feature/        # Game features (domain-driven)
    ├── world/      # Grid and field rendering
    ├── monster/    # Monster system (see below)
    ├── item/       # Item placement and effects
    ├── player/     # Player gauges
    └── ui/         # UI rendering
```

### Monster System Components

Located in `src/feature/monster/`:

- **components.rs**: Core components (Monster, MonsterKind, MonsterState, MonsterProperty, Movement)
- **spawn.rs**: Spawning from wave definitions
- **staging.rs**: Pre-movement waiting phase at field edges
- **movement.rs**: Movement system
- **collision.rs**: AABB collision detection between monsters
- **wait.rs**: WaitMeter for tracking stopped monsters
- **special_behavior.rs**: MyPace and PassThrough behaviors
- **despawn.rs**: Monster removal and event handling
- **definitions.rs**: MonsterDefinition loading from RON

## Core Design Patterns

### Data-Driven Configuration

Monster definitions and stage layouts are loaded from RON files in [assets/](assets/):

- [assets/monsters.ron](assets/monsters.ron): Monster definitions (kind, speed, size, special_behavior, texture_path)
- [assets/stages/stage1_level1.ron](assets/stages/stage1_level1.ron): Wave-based spawn configurations

When adding new monsters or stages, edit these files rather than modifying code.

### Event-Driven Communication

**Important**: Bevy 0.17 uses `MessageWriter`/`MessageReader` instead of `EventWriter`/`EventReader`.

- Events are registered with `.add_message::<EventType>()`
- Event types use `#[derive(Message)]` instead of `#[derive(Event)]`
- Use messages to communicate between systems (e.g., `MonsterDespawnEvent`)

Example:
```rust
#[derive(Message)]
pub struct MonsterDespawnEvent {
    pub entity: Entity,
    pub cause: DespawnCause,
}
```

### State Management

The game uses `GameState` enum ([lib.rs](src/lib.rs)):
- `InGame`: Main gameplay
- `GameOver`: When void gauge fills

Systems use `.run_if(in_state(GameState::InGame))` for conditional execution.

### Grid-Based Coordinates

All positions use `GridPosition` from [core/types.rs](src/core/types.rs):

- `grid_to_world()`: Convert grid coordinates to world coordinates
- `world_to_grid()`: Convert world coordinates to grid coordinates
- `is_valid_grid_position()`: Check if coordinates are within field bounds

The field is a 10x10 grid with `GRID_SIZE = 64.0` pixels per cell.

### Component Separation Pattern

Monsters use two movement-related components:

- **MonsterProperty**: Base/original parameters (base_direction, base_speed)
- **Movement**: Current parameters affected by items/effects (direction, speed, enabled)

This allows temporary modifications (e.g., rotation tiles) while preserving original values for reset.

## Special Systems

### Wave-Based Spawning

Monsters spawn in waves defined in stage RON files. Each wave has:
- `start_time`: Seconds from game start
- `monsters`: List of spawn definitions (kind, direction, grid_pos, delay)

The spawn system ([monster/spawn.rs](src/feature/monster/spawn.rs)) processes waves and creates monsters in `Staging` state.

### Staging System

Monsters appear at field edges and wait 2 seconds (STAGING_DURATION) before moving:
1. Spawn at edge in `MonsterState::Staging`
2. `StagingTimer` counts down
3. Transition to `MonsterState::Moving`

### Collision Detection

Monsters detect collisions with each other using AABB in [monster/collision.rs](src/feature/monster/collision.rs):
- Colliding monsters have their `Movement.speed` set to 0
- `WaitMeter` increases while stopped
- When `WaitMeter` exceeds threshold, monster despawns and void gauge increases

### Special Behaviors

Defined in `SpecialBehavior` enum:

- **None**: Standard movement
- **PassThrough**: Ghost-like, ignores collision with other monsters
- **MyPace { stop_interval, stop_duration }**: Periodically stops (uses `Movement.enabled = false`)

New behaviors are added as enum variants with corresponding system implementations.

### Player Gauges

Two gauges track game progress ([player/gauges.rs](src/feature/player/gauges.rs)):

- **SpiritGauge**: Increases when monsters reach goal (success)
- **VoidGauge**: Increases when monsters despawn from waiting too long (failure, triggers game over when full)

## Bevy 0.17 Specifics

Key API changes from earlier Bevy versions:

- `.add_plugin()` → `.add_plugins()`
- `.add_state()` → `.init_state()`
- `EventWriter`/`EventReader` → `MessageWriter`/`MessageReader`
- `Style` → `Node` component
- `Color::rgb()` → `Color::srgb()`
- `TextBundle` → separate `Text`, `TextFont`, `TextColor` components

### System Ordering

Use `.chain()` to enforce system execution order:

```rust
.add_systems(
    Update,
    (
        collision_detection_system,
        monster_movement_system,
        update_wait_meter_system,
        despawn_expired_monsters_system,
    )
        .chain()
        .run_if(in_state(GameState::InGame))
)
```

Order matters: collision must update before movement, movement before wait meter, etc.

## Asset Loading

Custom asset loaders in [core/stage_asset.rs](src/core/stage_asset.rs):

- `StageLevelAssetLoader`: Loads `.ron` files as `StageLevelAsset`
- `MonsterDefinitionsAssetLoader`: Loads monster definitions

Loading is asynchronous:
1. `Startup` system loads asset handle
2. `Update` system checks if asset is ready
3. When ready, converts to runtime resource (e.g., `MonsterDefinitions`)

## Configuration Files

### Technical Constants ([core/config.rs](src/core/config.rs))

Do not change these without considering visual/physics implications:
- `GRID_SIZE`, `FIELD_WIDTH`, `FIELD_HEIGHT`
- `COLLISION_THRESHOLD`
- Debug colors

### Game Balance ([core/level.rs](src/core/level.rs))

Safe to adjust for game balance:
- `MONSTER_SPEED`, `STAGING_DURATION`
- `WAIT_THRESHOLD`
- `SPIRIT_*`, `VOID_*` gauge parameters

## Code Standards

- **Module organization**: Each plugin in its own directory with `plugin.rs`, `components.rs`, and system files
- **System naming**: Use descriptive names ending with `_system`
- **Documentation**: Add `///` doc comments to all public items
- **Event-driven**: No direct system dependencies; use messages for communication
- **Data-driven**: Prefer RON configuration over hardcoded values
- **Single Responsibility**: Each system handles one specific task

## Language Note

The comprehensive development guide ([docs/development/dev_guide.md](docs/development/dev_guide.md)) is written in Japanese and contains extensive architectural documentation, design rationale, and implementation patterns. Refer to it for deep architectural understanding.