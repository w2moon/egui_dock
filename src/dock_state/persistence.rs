//! Versioned layout save/load helpers (requires `serde` feature).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::DockState;

pub const DOCK_LAYOUT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct DockLayoutFile<Tab> {
    pub version: u32,
    pub dock_state: DockState<Tab>,
}

impl<Tab> DockLayoutFile<Tab> {
    pub fn new(dock_state: DockState<Tab>) -> Self {
        Self {
            version: DOCK_LAYOUT_VERSION,
            dock_state,
        }
    }
}

#[derive(Debug)]
pub enum DockLayoutError {
    UnsupportedVersion { found: u32, expected: u32 },
    Json(serde_json::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for DockLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedVersion { found, expected } => {
                write!(f, "unsupported layout version {found} (expected {expected})")
            }
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for DockLayoutError {}

#[derive(Serialize)]
struct DockLayoutFileRef<'a, Tab> {
    version: u32,
    dock_state: &'a DockState<Tab>,
}

impl<Tab> DockState<Tab>
where
    Tab: Serialize,
{
    /// Serialize the full dock layout (including window surfaces) to JSON.
    pub fn to_layout_json(&self) -> Result<String, DockLayoutError> {
        let file = DockLayoutFileRef {
            version: DOCK_LAYOUT_VERSION,
            dock_state: self,
        };
        serde_json::to_string_pretty(&file).map_err(DockLayoutError::Json)
    }

    /// Save layout JSON to a file.
    pub fn save_layout_json_to_file(&self, path: impl AsRef<Path>) -> Result<(), DockLayoutError> {
        fs::write(path, self.to_layout_json()?).map_err(DockLayoutError::Io)
    }
}

impl<Tab> DockLayoutFile<Tab>
where
    Tab: for<'de> Deserialize<'de>,
{
    pub fn from_json_str(json: &str) -> Result<Self, DockLayoutError> {
        serde_json::from_str(json).map_err(DockLayoutError::Json)
    }

    pub fn into_dock_state(self) -> Result<DockState<Tab>, DockLayoutError> {
        if self.version != DOCK_LAYOUT_VERSION {
            return Err(DockLayoutError::UnsupportedVersion {
                found: self.version,
                expected: DOCK_LAYOUT_VERSION,
            });
        }
        Ok(self.dock_state)
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<DockState<Tab>, DockLayoutError> {
        let text = fs::read_to_string(path).map_err(DockLayoutError::Io)?;
        Self::from_json_str(&text)?.into_dock_state()
    }
}
