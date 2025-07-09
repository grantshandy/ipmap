use std::fmt;

use compact_str::CompactString;
use csv::ByteRecord;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{Error, csv_format};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct LookupInfo {
    pub crd: Coordinate,
    pub loc: Location,
}

/// Location metadata.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub city: Option<String>,
    pub region: Option<String>,
    pub country_code: String,
}

/// A latitude/longitude coordinate.
#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
pub struct Coordinate {
    pub lat: f32,
    pub lng: f32,
}

impl TryFrom<&ByteRecord> for Coordinate {
    type Error = Error;

    fn try_from(record: &ByteRecord) -> Result<Self, Self::Error> {
        let lat = record
            .get(csv_format::LATITUDE_IDX)
            .map(CompactString::from_utf8_lossy)
            .and_then(|s| s.parse::<f32>().ok())
            .ok_or(Error::InvalidFormat)?;

        let lng = record
            .get(csv_format::LONGITUDE_IDX)
            .map(CompactString::from_utf8_lossy)
            .and_then(|s| s.parse::<f32>().ok())
            .ok_or(Error::InvalidFormat)?;

        Ok(Self { lat, lng })
    }
}

impl PartialEq for Coordinate {
    fn eq(&self, other: &Self) -> bool {
        self.lat.to_ne_bytes() == other.lat.to_ne_bytes()
            && self.lng.to_ne_bytes() == other.lng.to_ne_bytes()
    }
}
impl Eq for Coordinate {}

impl std::hash::Hash for Coordinate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lat.to_ne_bytes().hash(state);
        self.lng.to_ne_bytes().hash(state);
    }
}

/// An ISO 3166 2-digit ASCII country code.
// Takes advantage of their compact representation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct CountryCode(u16);

impl<A: AsRef<[u8]>> From<A> for CountryCode {
    fn from(value: A) -> Self {
        match value.as_ref() {
            [a, b, ..] => Self(u16::from_ne_bytes([*a, *b])),
            _ => Self(0),
        }
    }
}

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.to_ne_bytes() {
            [0, 0] => "??".fmt(f),
            [a, b] => unsafe {
                char::from_u32_unchecked(a as u32).fmt(f)?;
                char::from_u32_unchecked(b as u32).fmt(f)
            },
        }
    }
}
