use crate::Error;
use compact_str::CompactString;
use heck::ToTitleCase;
use indexmap::IndexSet;
use rustc_hash::FxBuildHasher;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
    num::NonZero,
};

/// A memory-efficient store of named locations by their coordinates.
#[derive(Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocationStore {
    /// Coordinate to a single identifiable "location" (city) key
    pub(crate) coordinates: HashMap<Coordinate, LocationKey, FxBuildHasher>,
    /// Location key to associated string keys for city and region
    pub(crate) locations: IndexSet<LocationIndices, FxBuildHasher>,
    /// Deduplicated location name strings
    pub(crate) strings: StringDict,
}

impl LocationStore {
    /// Insert a new location into the store, only allocating/parsing/inserting strings when necessary
    pub fn insert(
        &mut self,
        coord: Coordinate,
        create_location: &dyn Fn(&mut StringDict) -> Result<LocationIndices, Error>,
    ) -> Result<(), Error> {
        // only allocating if the location is new saves millions of parses/allocations per database.
        if let Entry::Vacant(entry) = self.coordinates.entry(coord) {
            entry.insert(
                self.locations
                    .insert_full(create_location(&mut self.strings)?)
                    .0,
            );
        }

        Ok(())
    }

    /// Get the location for an associated coordinate.
    pub fn get(&self, coord: Coordinate) -> Option<Location> {
        self.coordinates.get(&coord).map(|i| {
            // UNWRAP: self.locations and self.coordinates are updated at the same time in Self::insert_location
            self.locations
                .get_index(*i)
                .unwrap()
                .populate(&self.strings)
        })
    }
}

type LocationKey = usize;
type StringDictKey = NonZero<u32>;

/// A compact database of strings that can store less than u32::MAX items.
#[derive(PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct StringDict(IndexSet<CompactString, FxBuildHasher>);

impl StringDict {
    pub fn insert_str(&mut self, item: CompactString) -> Option<StringDictKey> {
        if item.is_empty() {
            return None;
        }

        self.insert_bytes(item.as_bytes())
    }

    pub fn insert_bytes(&mut self, item: &[u8]) -> Option<StringDictKey> {
        if item.is_empty() {
            return None;
        }

        let s = CompactString::from_utf8(item).ok()?.to_lowercase();
        let (idx, _) = self.0.insert_full(s);

        NonZero::new((idx + 1) as u32)
    }

    pub fn get(&self, idx: StringDictKey) -> Option<String> {
        self.0
            .get_index((idx.get() - 1) as usize)
            .map(|c| c.to_title_case())
    }
}

/// A [`Coordinate`]/[`Location`] pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct LookupInfo {
    pub crd: Coordinate,
    pub loc: Location,
}

/// A [`Coordinate`]'s associated city, region, and country.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Type, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub city: Option<String>,
    pub region: Option<String>,
    pub country_code: String,
}

/// The city and region stored as indexes into a `StringDict` database.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct LocationIndices {
    pub(crate) city: Option<StringDictKey>,
    pub(crate) region: Option<StringDictKey>,
    pub(crate) country_code: CountryCode,
}

impl LocationIndices {
    fn populate(&self, strings: &StringDict) -> Location {
        Location {
            city: self.city.and_then(|i| strings.get(i)),
            region: self.region.and_then(|i| strings.get(i)),
            country_code: self.country_code.to_string(),
        }
    }
}

/// A basic latitude/longitude pair.
#[derive(Copy, Clone, Debug, Type, Default, Serialize, Deserialize)]
pub struct Coordinate {
    /// Latitude
    pub lat: f32,
    /// Longitude
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
        self.as_bytes().hash(state);
    }
}

/// An ISO 3166 2-digit ASCII country code.
// Takes advantage of its compact representation.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
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
    use super::*;

    #[test]
    fn country_code() {
        assert_eq!("us", CountryCode::from("us").to_string());
        assert_eq!("us", CountryCode::from("usa").to_string());
        assert_eq!("??", CountryCode::from("!").to_string());
    }

    #[test]
    fn string_dict() {
        let mut s = StringDict::default();

        let city = "A Region".to_string();
        let region = "City Name Here".to_string();

        let city_idx = s.insert_str(city.clone().into()).unwrap();
        let region_idx = s.insert_bytes(region.as_bytes()).unwrap();

        assert_eq!(Some(city), s.get(city_idx));
        assert_eq!(Some(region), s.get(region_idx));
    }
}
