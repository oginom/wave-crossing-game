use bevy::prelude::*;
use super::components::*;

/// プレイヤーキャラクターをスポーンする
pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        CircularMotion::default(),
        Sprite {
            color: Color::srgb(0.2, 0.7, 0.9),
            custom_size: Some(Vec2::new(30.0, 30.0)),
            ..default()
        },
        Transform::from_xyz(200.0, 0.0, 0.0),
    ));
}

/// 円運動を更新するシステム
pub fn circular_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CircularMotion), With<Player>>,
) {
    for (mut transform, mut motion) in &mut query {
        // 角度を更新
        motion.current_angle += motion.angular_velocity * time.delta_secs();

        // 2πを超えたら0に戻す（オプション、精度維持のため）
        if motion.current_angle > std::f32::consts::TAU {
            motion.current_angle -= std::f32::consts::TAU;
        }

        // 円運動の座標を計算
        let x = motion.center.x + motion.radius * motion.current_angle.cos();
        let y = motion.center.y + motion.radius * motion.current_angle.sin();

        transform.translation.x = x;
        transform.translation.y = y;
    }
}
