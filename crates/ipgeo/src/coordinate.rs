const LAT_MAX_ERROR: f32 = 0.00139;
const LNG_MAX_ERROR: f32 = 0.00277;

const LAT_MAX: f32 = 90.0;
const LNG_MAX: f32 = 180.0;

/// A basic latitude/longitude pair.
#[derive(Copy, Clone, Debug, Default, specta::Type, serde::Serialize, serde::Deserialize)]
pub struct Coordinate {
    /// Latitude
    pub lat: f32,
    /// Longitude
    pub lng: f32,
}

impl Coordinate {
    /// Returns true if the coordinates are equal within the packing error margin.
    pub fn approx_eq(&self, other: &Self) -> bool {
        (self.lat - other.lat).abs() < LAT_MAX_ERROR && (self.lng - other.lng).abs() < LNG_MAX_ERROR
    }

    fn as_bytes(&self) -> u64 {
        let mut out = [0; 8];
        let (one, two) = out.split_at_mut(4);
        one.copy_from_slice(self.lat.to_ne_bytes().as_slice());
        two.copy_from_slice(self.lng.to_ne_bytes().as_slice());
        u64::from_ne_bytes(out)
    }
}

impl PartialEq for Coordinate {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}
impl Eq for Coordinate {}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Coordinate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_bytes().cmp(&other.as_bytes())
    }
}

impl std::hash::Hash for Coordinate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

/// A packed 32-bit representation of a global coordinate.
///
/// Stores latitude and longitude as two 16-bit unsigned integers, allowing for efficient
/// serialization and comparison. Conversion from/to `Coordinate` is lossy, but tests
/// guarantee that it only loses at most a few hundred meters of precision.
/// See `coordinate::tests::packed_coordinate_max_error`.
///
/// # Precision
/// - Maximum latitude error: ~0.00139° (~ 155 meters)
/// - Maximum longitude error: ~0.00277° (~308 meters at the equator; less at higher latitudes)
///
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(PartialEq, Eq, PartialOrd, Ord, Hash))]
pub(crate) struct PackedCoordinate {
    pub lat_u: u16,
    pub lng_u: u16,
}

impl From<Coordinate> for PackedCoordinate {
    fn from(coord: Coordinate) -> Self {
        let lat_norm = (coord.lat.clamp(-LAT_MAX, LAT_MAX) + LAT_MAX) / (LAT_MAX * 2.0);
        let lng_norm = (coord.lng.clamp(-LNG_MAX, LNG_MAX) + LNG_MAX) / (LNG_MAX * 2.0);

        let lat_u = (lat_norm * u16::MAX as f32).round() as u16;
        let lng_u = (lng_norm * u16::MAX as f32).round() as u16;

        PackedCoordinate { lat_u, lng_u }
    }
}

impl From<&PackedCoordinate> for Coordinate {
    fn from(packed: &PackedCoordinate) -> Self {
        let lat_norm = packed.lat_u as f32 / u16::MAX as f32;
        let lng_norm = packed.lng_u as f32 / u16::MAX as f32;

        let lat = lat_norm * (LAT_MAX * 2.0) - LAT_MAX;
        let lng = lng_norm * (LNG_MAX * 2.0) - LNG_MAX;

        Coordinate { lat, lng }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STEP: f32 = 0.025;

    #[test]
    fn packed_coordinate_max_error() {
        let mut lat = -LAT_MAX;
        let mut lng = -LNG_MAX;

        while lat < LAT_MAX {
            while lng < LNG_MAX {
                let packed: PackedCoordinate = Coordinate { lat, lng }.into();
                let Coordinate {
                    lat: lat_conv,
                    lng: lng_conv,
                } = (&packed).into();

                let lat_diff = (lat - lat_conv).abs();
                let lng_diff = (lng - lng_conv).abs();

                let within_range = lat_diff < LAT_MAX_ERROR && lng_diff < LNG_MAX_ERROR;

                if !within_range {
                    dbg!((lat_conv, lng_conv), (lat, lng));
                }

                assert!(within_range);

                lng += TEST_STEP;
            }
            lng = -LNG_MAX;
            lat += TEST_STEP;
        }
    }
}
