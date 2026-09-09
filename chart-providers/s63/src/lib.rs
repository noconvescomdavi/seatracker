use seatracker_charts::ChartProvider;

#[derive(Debug, Default)]
pub struct S63Provider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum S63Access {
    AuthorizedExchangeSet,
    MissingPermit,
    Unsupported,
}

impl S63Provider {
    pub fn access_state(&self, has_authorized_permit: bool) -> S63Access {
        if has_authorized_permit {
            S63Access::AuthorizedExchangeSet
        } else {
            S63Access::MissingPermit
        }
    }
}

impl ChartProvider for S63Provider {
    fn provider_name(&self) -> &'static str {
        "S63Provider"
    }

    fn can_open(&self, _header: &[u8], extension: Option<&str>) -> bool {
        matches!(extension, Some(ext) if ext.eq_ignore_ascii_case("000") || ext.eq_ignore_ascii_case("os63"))
    }
}
