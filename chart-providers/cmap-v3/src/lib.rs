use seatracker_charts::{ChartProvider, ProviderCapabilities};

#[derive(Debug, Default)]
pub struct CmapV3LicensedProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmapV3Status {
    RequiresLicensedBackend,
    BackendAvailable,
}

impl CmapV3LicensedProvider {
    pub fn status(&self) -> CmapV3Status {
        CmapV3Status::RequiresLicensedBackend
    }

    pub fn protection_bypass_supported(&self) -> bool {
        false
    }
}

impl ChartProvider for CmapV3LicensedProvider {
    fn provider_name(&self) -> &'static str { "CMapV3LicensedProvider" }
    fn provider_version(&self) -> &'static str { "0.1.0" }

    fn can_open(&self, _header: &[u8], extension: Option<&str>) -> bool {
        matches!(extension, Some(ext) if ext.eq_ignore_ascii_case("senc") || ext.eq_ignore_ascii_case("cmap3"))
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
    fn never_claims_protection_bypass() {
        let provider = CmapV3LicensedProvider;
        assert!(!provider.protection_bypass_supported());
        assert_eq!(provider.status(), CmapV3Status::RequiresLicensedBackend);
    }
}
