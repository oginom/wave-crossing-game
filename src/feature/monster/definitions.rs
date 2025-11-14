use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// モンスターの種類
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterKind {
    Kappa,      // 河童 - 標準的な速度と挙動
    Ghost,      // ゴースト - 高速移動
    Bakeneko,   // 化け猫 - 大型でゆっくり
}

/// モンスターの定義（種類ごとのパラメータ）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterDefinition {
    pub kind: MonsterKind,
    pub speed: f32,
    pub size: f32,
    pub color: (f32, f32, f32),
    pub wait_threshold: f32,
}

/// モンスター定義を管理するリソース
#[derive(Resource, Default)]
pub struct MonsterDefinitions {
    definitions: HashMap<MonsterKind, MonsterDefinition>,
}

impl MonsterDefinitions {
    /// 指定された種類のモンスター定義を取得
    pub fn get(&self, kind: MonsterKind) -> &MonsterDefinition {
        self.definitions
            .get(&kind)
            .expect("Monster definition not found")
    }

    /// モンスター定義を登録
    pub fn insert(&mut self, def: MonsterDefinition) {
        self.definitions.insert(def.kind, def);
    }
}
