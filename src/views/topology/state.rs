use std::collections::HashMap;

use crate::metrics::alerts::Alert;
use crate::topology::posture::Posture;

/// The fleet map (ERR-120): view (pan, zoom, selection, filters) and what
/// it shows beyond the fleet itself (posture checks, open alerts).
pub struct TopologyState {
    pub pan: (f32, f32),
    pub zoom: f32,
    /// While dragging the map: (mouse at drag start, pan at drag start).
    pub drag: Option<((f32, f32), (f32, f32))>,
    pub selected: Option<String>,
    /// Only this lane (env) at full strength; the rest dimmed.
    pub lane: Option<String>,
    pub risky_only: bool,
    pub posture: HashMap<String, Posture>,
    pub posture_checked_at: i64,
    pub scanning: bool,
    pub alerts: Vec<Alert>,
}

impl Default for TopologyState {
    fn default() -> Self {
        Self { pan: (40.0, 40.0), zoom: 1.0, drag: None, selected: None, lane: None, risky_only: false, posture: HashMap::new(), posture_checked_at: 0, scanning: false, alerts: Vec::new() }
    }
}

pub const MIN_ZOOM: f32 = 0.4;
pub const MAX_ZOOM: f32 = 2.0;

/// Zooms by `factor` keeping the world point under `at` (screen, relative
/// to the map) where it is.
pub fn zoom_around(pan: (f32, f32), zoom: f32, factor: f32, at: (f32, f32)) -> ((f32, f32), f32) {
    let z = (zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
    let world = ((at.0 - pan.0) / zoom, (at.1 - pan.1) / zoom);
    ((at.0 - world.0 * z, at.1 - world.1 * z), z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_keeps_the_point_under_the_cursor() {
        let (pan, zoom) = zoom_around((40.0, 40.0), 1.0, 1.5, (300.0, 200.0));
        let world_before = ((300.0 - 40.0) / 1.0, (200.0 - 40.0) / 1.0);
        let screen_after = (world_before.0 * zoom + pan.0, world_before.1 * zoom + pan.1);
        assert!((screen_after.0 - 300.0).abs() < 0.01 && (screen_after.1 - 200.0).abs() < 0.01);
        assert_eq!(zoom_around((0.0, 0.0), 1.9, 2.0, (0.0, 0.0)).1, MAX_ZOOM);
    }
}
