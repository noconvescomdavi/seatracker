use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    Serial, Network, Filesystem, Gps, Bluetooth, Usb, ChartData, NavigationState
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub api_version: u32,
    pub permissions: Vec<Permission>,
    pub capabilities: Vec<String>,
}

pub trait SeaTrackerPlugin {
    fn manifest(&self) -> &PluginManifest;
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
}

pub fn permissions_granted(requested: &[Permission], granted: &[Permission]) -> bool {
    requested.iter().all(|p| granted.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn permissions_are_explicit() {
        assert!(!permissions_granted(&[Permission::Gps,Permission::Network], &[Permission::Gps]));
    }
}
