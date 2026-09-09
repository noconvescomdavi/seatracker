#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataValidity { Unknown, Invalid, Stale, Unavailable, Valid }

#[derive(Debug, Clone, Copy)]
pub struct NavigationState {
    pub latitude: f64,
    pub longitude: f64,
    pub sog_knots: f32,
    pub cog_deg: f32,
    pub validity: DataValidity,
}
impl NavigationState {
    pub fn is_usable(&self) -> bool { self.validity == DataValidity::Valid }
}
