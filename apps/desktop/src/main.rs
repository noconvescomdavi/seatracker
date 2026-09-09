use seatracker_navigation::{DataValidity, NavigationState};

fn main() {
    let state = NavigationState {
        latitude: 0.0,
        longitude: 0.0,
        sog_knots: 0.0,
        cog_deg: 0.0,
        validity: DataValidity::Unavailable,
        ..NavigationState::default()
    };

    println!("SeaTracker Desktop bootstrap");
    println!("Shared navigation core linked: {}", !state.is_usable());
    println!("UI backend pending desktop framework integration.");
}
