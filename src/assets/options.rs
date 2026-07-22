use super::*;

use serde::Deserializer;

fn ok_or_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned + Default,
    D: Deserializer<'de>,
{
    Ok(T::deserialize(deserializer).unwrap_or_else(|err| {
        log::error!(
            "failed to deserialize type {}, using default, error: {:?}",
            std::any::type_name::<T>(),
            err
        );
        T::default()
    }))
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Options {
    #[serde(deserialize_with = "ok_or_default")]
    pub volume: VolumeOptions,
    #[serde(deserialize_with = "ok_or_default")]
    pub graphics: GraphicsOptions,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GraphicsOptions {
    pub crt: GraphicsCrtOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GraphicsCrtOptions {
    pub enabled: bool,
    #[serde(skip)]
    pub curvature: f32,
    #[serde(skip)]
    pub vignette: f32,
    #[serde(skip)]
    pub scanlines: f32,
}

impl Default for GraphicsCrtOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            curvature: 0.05,
            vignette: 0.2,
            scanlines: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct VolumeOptions {
    /// Volume in range `0.0..=1.0`.
    pub volume_master: f32, // TODO: range should be part of the type
    pub volume_music: f32,
    pub volume_sfx: f32,
}

impl VolumeOptions {
    pub fn master(&self) -> f32 {
        self.volume_master
    }

    pub fn music(&self) -> f32 {
        self.master() * self.volume_music / 100.0
    }

    pub fn sfx(&self) -> f32 {
        self.master() * self.volume_sfx / 100.0
    }
}

impl Default for VolumeOptions {
    fn default() -> Self {
        Self {
            volume_master: 0.5,
            volume_music: 1.0,
            volume_sfx: 1.0,
        }
    }
}
