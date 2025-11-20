use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
};
use serde::Deserialize;
use std::collections::HashMap;
use crate::feature::monster::{StageLevel, WaveDefinition, MonsterDefinition, MonsterKind};
use crate::feature::obstacle::ObstacleDefinition;

/// ステージレベルファイルの構造
#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct StageLevelAsset {
    pub stage: u32,
    pub level: u32,
    pub waves: Vec<WaveDefinition>,
    #[serde(default)]
    pub obstacles: Vec<ObstacleDefinition>,
}

impl StageLevelAsset {
    pub fn to_stage_level(&self) -> StageLevel {
        StageLevel {
            stage: self.stage,
            level: self.level,
            waves: self.waves.clone(),
        }
    }
}

#[derive(Default)]
pub struct StageLevelAssetLoader;

impl AssetLoader for StageLevelAssetLoader {
    type Asset = StageLevelAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let asset: StageLevelAsset = ron::from_str(content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// モンスター定義ファイルの構造
#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct MonsterDefinitionsAsset {
    pub definitions: Vec<MonsterDefinition>,
}

impl MonsterDefinitionsAsset {
    pub fn to_hashmap(&self) -> HashMap<MonsterKind, MonsterDefinition> {
        let mut map = HashMap::new();
        for def in &self.definitions {
            map.insert(def.kind, def.clone());
        }
        map
    }
}

#[derive(Default)]
pub struct MonsterDefinitionsAssetLoader;

impl AssetLoader for MonsterDefinitionsAssetLoader {
    type Asset = MonsterDefinitionsAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let asset: MonsterDefinitionsAsset = ron::from_str(content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

pub struct StageAssetPlugin;

impl Plugin for StageAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<StageLevelAsset>()
            .init_asset_loader::<StageLevelAssetLoader>()
            .init_asset::<MonsterDefinitionsAsset>()
            .init_asset_loader::<MonsterDefinitionsAssetLoader>();
    }
}