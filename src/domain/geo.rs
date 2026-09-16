const EARTH_RADIUS_KM: f64 = 6371.0;

/// Great-circle distance between two (lat, lng) points in kilometers, using the haversine formula.
pub fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();

    let a =
        (dlat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (dlng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_KM * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sf_to_oakland_is_roughly_13km() {
        let sf = (37.7749, -122.4194);
        let oakland = (37.8044, -122.2712);
        let d = haversine_km(sf.0, sf.1, oakland.0, oakland.1);
        assert!((d - 13.0).abs() <= 3.0, "expected ~13km +/- 3km, got {}", d);
    }

    #[test]
    fn same_point_is_zero() {
        let d = haversine_km(10.0, 20.0, 10.0, 20.0);
        assert!(d.abs() < 1e-9);
    }
}
