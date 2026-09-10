use seatracker_charts::{ChartProvider, ProviderCapabilities};

#[derive(Debug, Default)]
pub struct NavionicsSdkProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavionicsSdkState {
    MissingSdk,
    MissingCredentials,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavionicsSdkConfig {
    pub sdk_present: bool,
    pub project_token_present: bool,
    pub private_key_present: bool,
    pub configuration_token_present: bool,
}

impl NavionicsSdkConfig {
    pub fn state(&self) -> NavionicsSdkState {
        if !self.sdk_present {
            NavionicsSdkState::MissingSdk
        } else if !(self.project_token_present && self.private_key_present && self.configuration_token_present) {
            NavionicsSdkState::MissingCredentials
        } else {
            NavionicsSdkState::Ready
        }
    }
}

impl ChartProvider for NavionicsSdkProvider {
    fn provider_name(&self) -> &'static str { "NavionicsOfficialSdkProvider" }
    fn provider_version(&self) -> &'static str { "0.1.0" }

    fn can_open(&self, _header: &[u8], extension: Option<&str>) -> bool {
        matches!(extension, Some(ext) if ext.eq_ignore_ascii_case("nv2") || ext.eq_ignore_ascii_case("navionics"))
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            raster: false,
            vector: true,
            updates: true,
            protected_content: true,
            object_info: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn credentials_gate_readiness() {
        let missing = NavionicsSdkConfig { sdk_present: true, project_token_present: false, private_key_present: false, configuration_token_present: false };
        assert_eq!(missing.state(), NavionicsSdkState::MissingCredentials);
        let ready = NavionicsSdkConfig { sdk_present: true, project_token_present: true, private_key_present: true, configuration_token_present: true };
        assert_eq!(ready.state(), NavionicsSdkState::Ready);
    }
}
