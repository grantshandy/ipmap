use serde::{Deserialize, Serialize};
use specta::Type;
use std::fmt;

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

impl Coordinate {
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
        self.lat.to_ne_bytes().hash(state);
        self.lng.to_ne_bytes().hash(state);
    }
}

/// An ISO 3166 2-digit ASCII country code.
// Takes advantage of their compact representation.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::CountryCode;

    #[test]
    fn country_code() {
        assert_eq!("us", CountryCode::from("us").to_string());
        assert_eq!("us", CountryCode::from("usa").to_string());
        assert_eq!("??", CountryCode::from("!").to_string());
    }
}
