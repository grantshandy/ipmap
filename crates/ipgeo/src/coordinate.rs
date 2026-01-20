//! Types for working with and storing coordinates.

use std::hash;

const LAT_RANGE: f32 = 90.0;
const LNG_RANGE: f32 = 180.0;

/// A basic latitude/longitude pair.
#[derive(Copy, Clone, Debug, Default, specta::Type, serde::Serialize, serde::Deserialize)]
pub struct Coordinate {
    /// Latitude
    pub lat: f32,
    /// Longitude
    pub lng: f32,
}

impl Coordinate {
    /// Returns true if the coordinates are equal within the precision of the database via `PackedCoordinate`.
    pub fn approx_eq(&self, other: &Self) -> bool {
        pack_degree(self.lat, LAT_RANGE) == pack_degree(other.lat, LAT_RANGE)
            && pack_degree(self.lng, LNG_RANGE) == pack_degree(other.lng, LNG_RANGE)
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

impl hash::Hash for Coordinate {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

/// Converts from a packed u16 type to an f32 degree, given an absolute range (-abs_rng..=abs_rng).
#[inline]
const fn pack_degree(deg: f32, range: f32) -> u16 {
    (((deg.clamp(-range, range) + range) / (range * 2.0)) * u16::MAX as f32).round() as u16
}

/// Converts from a packed u16 type to an f32 degree, given an absolute range (-abs_rng..=abs_rng).
#[inline]
const fn unpack_degree(deg_u: u16, range: f32) -> f32 {
    (deg_u as f32 / u16::MAX as f32) * (range * 2.0) - range
}

/// A packed 32-bit representation of a global coordinate.
///
/// Stores latitude and longitude as two 16-bit unsigned integers, allowing for efficient
/// serialization and comparison. Conversion to/from `Coordinate` is lossy, but tests
/// guarantee that it only loses at most a few hundred meters of precision.
///
/// See `coordinate::tests::packed_coordinate_max_error`.
///
/// # Precision
/// - Maximum latitude error: ~0.00139° (~155 meters)
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
        PackedCoordinate {
            lat_u: pack_degree(coord.lat, LAT_RANGE),
            lng_u: pack_degree(coord.lng, LNG_RANGE),
        }
    }
}

impl From<&PackedCoordinate> for Coordinate {
    fn from(packed: &PackedCoordinate) -> Self {
        Coordinate {
            lat: unpack_degree(packed.lat_u, LAT_RANGE),
            lng: unpack_degree(packed.lng_u, LNG_RANGE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAT_MAX_ERROR: f32 = 0.00139;
    const LNG_MAX_ERROR: f32 = 0.00277;

    const TEST_STEP: f32 = 0.025;

    #[test]
    fn packed_coordinate_max_error() {
        let mut lat = -LAT_RANGE;
        let mut lng = -LNG_RANGE;

        while lat < LAT_RANGE {
            while lng < LNG_RANGE {
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

            lng = -LNG_RANGE;
            lat += TEST_STEP;
        }
    }
}
