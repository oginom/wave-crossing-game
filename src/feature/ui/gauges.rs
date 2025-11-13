use bevy::prelude::*;
use crate::feature::player::PlayerGauges;

// ゲージUI用のコンポーネント
#[derive(Component)]
pub struct GaugesPanel;

#[derive(Component)]
pub struct SpiritGaugeBar;

#[derive(Component)]
pub struct VoidGaugeBar;

#[derive(Component)]
pub struct SpiritGaugeText;

#[derive(Component)]
pub struct VoidGaugeText;

// ゲージのサイズとマージン
const GAUGE_WIDTH: f32 = 200.0;
const GAUGE_HEIGHT: f32 = 30.0;
const GAUGE_MARGIN: f32 = 10.0;

/// 画面右下にゲージUIを生成
pub fn setup_gauges_ui_system(mut commands: Commands) {
    commands
        .spawn((
            GaugesPanel,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                bottom: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(GAUGE_MARGIN),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Spirit ゲージ
            create_gauge_row(
                parent,
                "Spirit",
                SpiritGaugeBar,
                SpiritGaugeText,
                Color::srgb(0.2, 0.8, 1.0), // 青系
            );

            // Void ゲージ
            create_gauge_row(
                parent,
                "Void",
                VoidGaugeBar,
                VoidGaugeText,
                Color::srgb(0.8, 0.2, 0.8), // 紫系
            );
        });
}

fn create_gauge_row<T: Component, U: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    bar_marker: T,
    text_marker: U,
    color: Color,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|row| {
            // ラベル
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // ゲージ背景
            row.spawn((
                Node {
                    width: Val::Px(GAUGE_WIDTH),
                    height: Val::Px(GAUGE_HEIGHT),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .with_children(|bg| {
                // ゲージバー
                bg.spawn((
                    bar_marker,
                    Node {
                        width: Val::Percent(0.0), // 初期値
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(color),
                ));
            });

            // 数値表示
            row.spawn((
                text_marker,
                Text::new("0/100"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Spiritゲージの表示を更新（バーと数値の両方）
pub fn update_spirit_gauge_ui_system(
    gauges: Res<PlayerGauges>,
    mut bar_query: Query<&mut Node, With<SpiritGaugeBar>>,
    mut text_query: Query<&mut Text, With<SpiritGaugeText>>,
) {
    // ゲージバーの更新
    for mut node in bar_query.iter_mut() {
        node.width = Val::Percent(gauges.spirit.ratio() * 100.0);
    }

    // 数値テキストの更新
    for mut text in text_query.iter_mut() {
        **text = format!("{:.0}/{:.0}", gauges.spirit.current, gauges.spirit.max);
    }
}

/// Voidゲージの表示を更新（バーと数値の両方）
pub fn update_void_gauge_ui_system(
    gauges: Res<PlayerGauges>,
    mut bar_query: Query<&mut Node, With<VoidGaugeBar>>,
    mut text_query: Query<&mut Text, With<VoidGaugeText>>,
) {
    // ゲージバーの更新
    for mut node in bar_query.iter_mut() {
        node.width = Val::Percent(gauges.void.ratio() * 100.0);
    }

    // 数値テキストの更新
    for mut text in text_query.iter_mut() {
        **text = format!("{:.0}/{:.0}", gauges.void.current, gauges.void.max);
    }
}
