use seatracker_charts::ChartProvider;

#[derive(Debug, Default)]
pub struct Cm93Provider;

impl ChartProvider for Cm93Provider {
    fn provider_name(&self) -> &'static str { "CM93Provider" }

    fn can_open(&self, _header: &[u8], extension: Option<&str>) -> bool {
        matches!(extension, Some(ext) if ext.eq_ignore_ascii_case("cm93"))
    }
}

pub fn supported_without_protection_bypass() -> bool {
    false
}
