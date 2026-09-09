use seatracker_charts::ChartProvider;

#[derive(Debug, Default)]
pub struct MbTilesProvider;

impl MbTilesProvider {
    pub const SQLITE_MAGIC: &'static [u8] = b"SQLite format 3\0";
    pub fn is_sqlite(header: &[u8]) -> bool {
        header.starts_with(Self::SQLITE_MAGIC)
    }
}

impl ChartProvider for MbTilesProvider {
    fn provider_name(&self) -> &'static str { "MBTilesProvider" }
    fn can_open(&self, header: &[u8], extension: Option<&str>) -> bool {
        extension.map(|e| e.eq_ignore_ascii_case("mbtiles")).unwrap_or(false)
            || Self::is_sqlite(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_sqlite_magic() {
        assert!(MbTilesProvider::is_sqlite(b"SQLite format 3\0rest"));
    }
}
