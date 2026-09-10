use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    Serial,
    Network,
    Filesystem,
    Gps,
    Bluetooth,
    Usb,
    ChartData,
    NavigationState,
    AisTargets,
    Alarms,
    RouteControl,
    AutopilotOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Capability {
    ChartProvider,
    OverlayProvider,
    DataConnection,
    Nmea0183,
    Nmea2000,
    SignalK,
    Ais,
    Dashboard,
    InstrumentProvider,
    RouteService,
    TrackService,
    AlarmProvider,
    Weather,
    TidesCurrents,
    Grib,
    WeatherRouting,
    AutopilotAdapter,
    ImportExport,
    Scripting,
    Diagnostics,
    Simulator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub api_version: u32,
    pub permissions: BTreeSet<Permission>,
    pub capabilities: BTreeSet<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginEvent {
    Started {
        plugin_id: String,
    },
    Stopped {
        plugin_id: String,
    },
    Fault {
        plugin_id: String,
        message: String,
    },
    Nmea0183 {
        sentence: String,
        received_at_ms: u64,
    },
    SignalKJson {
        json: String,
        received_at_ms: u64,
    },
    AisSentence {
        sentence: String,
        received_at_ms: u64,
    },
    NavigationChanged,
    RouteChanged,
    TrackChanged,
    AlarmRaised {
        alarm_id: String,
        message: String,
    },
    AlarmCleared {
        alarm_id: String,
    },
    ChartChanged {
        chart_id: String,
    },
    Diagnostic {
        category: String,
        message: String,
    },
}

pub trait SeaTrackerPlugin {
    fn manifest(&self) -> &PluginManifest;
    fn start(&mut self, host: &mut dyn PluginHost) -> Result<(), String>;
    fn stop(&mut self, host: &mut dyn PluginHost) -> Result<(), String>;
    fn on_event(&mut self, _event: &PluginEvent, _host: &mut dyn PluginHost) -> Result<(), String> {
        Ok(())
    }
}

pub trait PluginHost {
    fn emit(&mut self, event: PluginEvent);
    fn has_permission(&self, plugin_id: &str, permission: &Permission) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginState {
    Registered,
    Running,
    Stopped,
    Faulted(String),
}

#[derive(Default)]
pub struct PluginRegistry {
    plugins: BTreeMap<String, Box<dyn SeaTrackerPlugin>>,
    states: BTreeMap<String, PluginState>,
    grants: BTreeMap<String, BTreeSet<Permission>>,
    queue: Vec<PluginEvent>,
}

impl PluginRegistry {
    pub fn register(&mut self, plugin: Box<dyn SeaTrackerPlugin>) -> Result<(), String> {
        let id = plugin.manifest().id.clone();
        if id.trim().is_empty() {
            return Err("plugin id cannot be empty".into());
        }
        if self.plugins.contains_key(&id) {
            return Err(format!("plugin already registered: {id}"));
        }
        self.states.insert(id.clone(), PluginState::Registered);
        self.plugins.insert(id, plugin);
        Ok(())
    }

    pub fn grant(&mut self, plugin_id: &str, permissions: impl IntoIterator<Item = Permission>) {
        self.grants
            .entry(plugin_id.to_string())
            .or_default()
            .extend(permissions);
    }

    pub fn state(&self, plugin_id: &str) -> Option<&PluginState> {
        self.states.get(plugin_id)
    }

    pub fn manifests(&self) -> Vec<&PluginManifest> {
        self.plugins
            .values()
            .map(|plugin| plugin.manifest())
            .collect()
    }

    pub fn start(&mut self, plugin_id: &str) -> Result<(), String> {
        let requested = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("plugin not found: {plugin_id}"))?
            .manifest()
            .permissions
            .clone();
        let granted = self.grants.get(plugin_id).cloned().unwrap_or_default();
        if !requested.is_subset(&granted) {
            return Err(format!("permissions not granted for plugin: {plugin_id}"));
        }

        let mut plugin = self
            .plugins
            .remove(plugin_id)
            .ok_or_else(|| format!("plugin not found: {plugin_id}"))?;
        let result = plugin.start(self);
        self.plugins.insert(plugin_id.to_string(), plugin);

        match result {
            Ok(()) => {
                self.states
                    .insert(plugin_id.to_string(), PluginState::Running);
                self.emit(PluginEvent::Started {
                    plugin_id: plugin_id.to_string(),
                });
                Ok(())
            }
            Err(error) => {
                self.states
                    .insert(plugin_id.to_string(), PluginState::Faulted(error.clone()));
                self.emit(PluginEvent::Fault {
                    plugin_id: plugin_id.to_string(),
                    message: error.clone(),
                });
                Err(error)
            }
        }
    }

    pub fn stop(&mut self, plugin_id: &str) -> Result<(), String> {
        let mut plugin = self
            .plugins
            .remove(plugin_id)
            .ok_or_else(|| format!("plugin not found: {plugin_id}"))?;
        let result = plugin.stop(self);
        self.plugins.insert(plugin_id.to_string(), plugin);
        if result.is_ok() {
            self.states
                .insert(plugin_id.to_string(), PluginState::Stopped);
            self.emit(PluginEvent::Stopped {
                plugin_id: plugin_id.to_string(),
            });
        }
        result
    }

    pub fn dispatch(&mut self, event: PluginEvent) {
        let ids: Vec<String> = self.plugins.keys().cloned().collect();
        for id in ids {
            if self.states.get(&id) != Some(&PluginState::Running) {
                continue;
            }
            let Some(mut plugin) = self.plugins.remove(&id) else {
                continue;
            };
            if let Err(error) = plugin.on_event(&event, self) {
                self.states
                    .insert(id.clone(), PluginState::Faulted(error.clone()));
                self.emit(PluginEvent::Fault {
                    plugin_id: id.clone(),
                    message: error,
                });
            }
            self.plugins.insert(id, plugin);
        }
    }

    pub fn drain_events(&mut self) -> Vec<PluginEvent> {
        std::mem::take(&mut self.queue)
    }
}

impl PluginHost for PluginRegistry {
    fn emit(&mut self, event: PluginEvent) {
        self.queue.push(event);
    }

    fn has_permission(&self, plugin_id: &str, permission: &Permission) -> bool {
        self.grants
            .get(plugin_id)
            .is_some_and(|grants| grants.contains(permission))
    }
}

pub fn permissions_granted(requested: &[Permission], granted: &[Permission]) -> bool {
    requested
        .iter()
        .all(|permission| granted.contains(permission))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DemoPlugin {
        manifest: PluginManifest,
        starts: usize,
    }

    impl DemoPlugin {
        fn new() -> Self {
            Self {
                manifest: PluginManifest {
                    id: "demo".into(),
                    name: "Demo".into(),
                    version: "1.0.0".into(),
                    author: "SeaTracker".into(),
                    api_version: 1,
                    permissions: BTreeSet::from([Permission::NavigationState]),
                    capabilities: BTreeSet::from([Capability::Dashboard]),
                },
                starts: 0,
            }
        }
    }

    impl SeaTrackerPlugin for DemoPlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }

        fn start(&mut self, _host: &mut dyn PluginHost) -> Result<(), String> {
            self.starts += 1;
            Ok(())
        }

        fn stop(&mut self, _host: &mut dyn PluginHost) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn permissions_are_explicit() {
        assert!(!permissions_granted(
            &[Permission::Gps, Permission::Network],
            &[Permission::Gps]
        ));
    }

    #[test]
    fn plugin_requires_grants_before_start() {
        let mut registry = PluginRegistry::default();
        registry.register(Box::new(DemoPlugin::new())).unwrap();
        assert!(registry.start("demo").is_err());
        registry.grant("demo", [Permission::NavigationState]);
        assert!(registry.start("demo").is_ok());
        assert_eq!(registry.state("demo"), Some(&PluginState::Running));
    }
}
