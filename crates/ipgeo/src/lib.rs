#![doc = include_str!("../README.md")]

use std::{
    collections::HashMap,
    fmt,
    io::{Read, Seek, SeekFrom},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ops::RangeInclusive,
};

use compact_str::CompactString;
use csv::{ByteRecord, ReaderBuilder};
use flate2::read::GzDecoder;
use heck::ToTitleCase;
use indexmap::IndexSet;
use rangemap::{RangeInclusiveMap, StepLite};

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

// CSV indexes for city-ipv[4/6][-num].csv format
//
//   https://github.com/sapics/ip-location-db?tab=readme-ov-file#city-csv-format
//
const IP_RANGE_START_IDX: usize = 0;
const IP_RANGE_END_IDX: usize = 1;
const COUNTRY_CODE_IDX: usize = 2;
const STATE_IDX: usize = 3;
const CITY_IDX: usize = 5;
const LATITUDE_IDX: usize = 7;
const LONGITUDE_IDX: usize = 8;

/// A database of IP address ranges and their corresponding coordinates and location metadata.
#[derive(PartialEq)]
pub struct GeoDatabase {
    inner: GdbType,
}

impl GeoDatabase {
    /// Automatically detects the format of the GeoDatabase and parses it.
    pub fn from_read(mut source: impl Read + Seek) -> Result<Self, Error> {
        // Check for GZIP magic numbers
        let mut head = [0; 2];
        source.read_exact(&mut head).map_err(Error::Io)?;
        source.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
        let is_gzip = head == GZIP_MAGIC;

        // Pull out the first IP to test the format
        let mut scratch_buff = [0u8; 50];
        if is_gzip {
            let mut decoder = GzDecoder::new(source);
            decoder.read_exact(&mut scratch_buff).map_err(Error::Io)?;
            source = decoder.into_inner();
        } else {
            source.read_exact(&mut scratch_buff).map_err(Error::Io)?;
        }
        source.seek(SeekFrom::Start(0)).map_err(Error::Io)?;

        // Read the first line and check for IP range formats.
        let test_ip = scratch_buff
            .split(|&b| b == b',')
            .next()
            .and_then(|line| CompactString::from_utf8(line).ok())
            .ok_or(Error::NoRecords)?;

        let parsed_ip = test_ip.parse::<IpAddr>();
        let is_num = parsed_ip.is_err();

        let is_ipv6 = if is_num {
            test_ip.parse::<u32>().is_err()
        } else {
            parsed_ip.is_ok_and(|ip| ip.is_ipv6())
        };

        // is this too monomorphized? Should this be better with dyn dispatch...?
        #[rustfmt::skip]
        let inner = match (is_gzip, is_num, is_ipv6) {
            (false, false, false) => Gdb::from_read::<StrParser>(source).map(GdbType::Ipv4)?,
            (false, true, false) => Gdb::from_read::<NumParser>(source).map(GdbType::Ipv4)?,
            (false, false, true) => Gdb::from_read::<StrParser>(source).map(GdbType::Ipv6)?,
            (false, true, true) => Gdb::from_read::<NumParser>(source).map(GdbType::Ipv6)?,
            (true, false, false) => Gdb::from_read::<StrParser>(GzDecoder::new(source)).map(GdbType::Ipv4)?,
            (true, true, false) => Gdb::from_read::<NumParser>(GzDecoder::new(source)).map(GdbType::Ipv4)?,
            (true, false, true) => Gdb::from_read::<StrParser>(GzDecoder::new(source)).map(GdbType::Ipv6)?,
            (true, true, true) => Gdb::from_read::<NumParser>(GzDecoder::new(source)).map(GdbType::Ipv6)?,
        };

        Ok(Self { inner })
    }

    /// Returns the coordinate in the database for a given IP address.
    pub fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        match (&self.inner, ip) {
            (GdbType::Ipv4(db), IpAddr::V4(ip)) => db.get_coordinate(ip),
            (GdbType::Ipv6(db), IpAddr::V6(ip)) => db.get_coordinate(ip),
            _ => None,
        }
    }

    /// Returns the coordinate and location metadata for a given IP address.
    pub fn get(&self, ip: IpAddr) -> Option<(Coordinate, Location)> {
        match (&self.inner, ip) {
            (GdbType::Ipv4(db), IpAddr::V4(ip)) => db.get(ip),
            (GdbType::Ipv6(db), IpAddr::V6(ip)) => db.get(ip),
            _ => None,
        }
    }

    pub fn is_ipv4(&self) -> bool {
        matches!(self.inner, GdbType::Ipv4(_))
    }

    pub fn is_ipv6(&self) -> bool {
        matches!(self.inner, GdbType::Ipv6(_))
    }
}

/// A latitude/longitude coordinate.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Coordinate {
    pub lat: f32,
    pub lng: f32,
}

/// Location metadata.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location {
    pub city: Option<String>,
    pub region: Option<String>,
    pub country_code: String,
}

/// All errors that can occur when parsing the database.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("csv parsing error: {0}")]
    ReadCsv(#[from] csv::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// TODO: add line/column number to this error.
    #[error("error parsing CSV contents")]
    InvalidFormat,
    /// Indexes into the string database are stored in u32s to save space.
    /// I've never found a database where there aren't more than u32::MAX strings, If they're being generated, please contact me and I'll change this to u64.
    #[error("database has more than u32::MAX unique strings, which is not supported")]
    DatabaseMetadataOverflow,
    #[error("no records found")]
    NoRecords,
}

#[derive(PartialEq)]
enum GdbType {
    Ipv4(Gdb<Ipv4Addr>),
    Ipv6(Gdb<Ipv6Addr>),
}

/// IpAddr -(map)-> PackedCoordinate -(locations)-> LocationIndices -(strings)-> Location
#[derive(PartialEq)]
struct Gdb<B> {
    map: RangeInclusiveMap<SteppedIp<B>, PackedCoordinate>,
    locations: HashMap<PackedCoordinate, LocationIndices>,
    strings: IndexSet<CompactString>,
}

impl<B> Gdb<B>
where
    B: Ord + Clone,
    SteppedIp<B>: StepLite,
{
    /// Internal generic implementation.
    fn from_read<P: IpParser<B>>(file: impl Read) -> Result<Self, Error> {
        let mut db = Self {
            map: RangeInclusiveMap::new(),
            locations: HashMap::new(),
            strings: IndexSet::new(),
        };

        // The first element is the "null" value, so indexes in LocationIndices that are 0 are null.
        // This is better than Option<StringDictKey> because it saves a bit of memory.
        let (zero, _) = db.strings.insert_full(CompactString::default());
        assert_eq!(zero, 0);

        for record in ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file)
            .byte_records()
        {
            let record = record.map_err(Error::ReadCsv)?;
            let coord = PackedCoordinate::try_from(&record)?;

            db.map.insert(
                RangeInclusive::new(
                    P::parse(&record[IP_RANGE_START_IDX]).ok_or(Error::InvalidFormat)?,
                    P::parse(&record[IP_RANGE_END_IDX]).ok_or(Error::InvalidFormat)?,
                ),
                coord,
            );
            db.insert_location(coord, &record)?;
        }

        Ok(db)
    }

    /// Returns the coordinate in the database for a given IP address.
    fn get_coordinate(&self, ip: B) -> Option<Coordinate> {
        self.map.get(&SteppedIp::<B>(ip)).copied().map(Into::into)
    }

    /// Returns the coordinate and location metadata for a given IP address.
    fn get(&self, ip: B) -> Option<(Coordinate, Location)> {
        let coord = self.map.get(&SteppedIp::<B>(ip)).copied()?;

        let location = self.locations.get(&coord).map(|loc| Location {
            city: self.get_string(loc.city),
            region: self.get_string(loc.region),
            country_code: loc.country_code.to_string(),
        })?;

        Some((coord.into(), location))
    }

    /// Inserts a location into the database if it doesn't already exist.
    fn insert_location(
        &mut self,
        coord: PackedCoordinate,
        record: &ByteRecord,
    ) -> Result<(), Error> {
        if self.locations.contains_key(&coord) {
            return Ok(());
        }

        let city = self.add_string(&record[CITY_IDX])?;
        let region = self.add_string(&record[STATE_IDX])?;
        let country_code = CountryCode::from(&record[COUNTRY_CODE_IDX]);

        self.locations.insert(
            coord,
            LocationIndices {
                city,
                region,
                country_code,
            },
        );

        Ok(())
    }

    /// Adds a string to the string database and returns its index.
    fn add_string(&mut self, item: &[u8]) -> Result<StringDictKey, Error> {
        if item.is_empty() {
            return Ok(0);
        }

        let (idx, _) = self
            .strings
            .insert_full(CompactString::from_utf8_lossy(item).to_lowercase());

        if idx > StringDictKey::MAX as usize {
            return Err(Error::DatabaseMetadataOverflow);
        }

        Ok(idx as StringDictKey)
    }

    /// Retrieves a string from the string database by its index.
    fn get_string(&self, idx: StringDictKey) -> Option<String> {
        if idx == 0 {
            return None;
        }

        self.strings
            .get_index(idx as usize)
            .map(|s| s.to_title_case())
    }
}

/// A wrapper around Ipv4Addr and Ipv6Addr to allow for a StepLite implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[repr(transparent)]
struct SteppedIp<B>(pub B);

impl rangemap::StepLite for SteppedIp<Ipv4Addr> {
    fn add_one(&self) -> Self {
        Self(self.0.to_bits().add_one().into())
    }

    fn sub_one(&self) -> Self {
        Self(self.0.to_bits().sub_one().into())
    }
}

impl rangemap::StepLite for SteppedIp<Ipv6Addr> {
    fn add_one(&self) -> Self {
        Self(self.0.to_bits().add_one().into())
    }

    fn sub_one(&self) -> Self {
        Self(self.0.to_bits().sub_one().into())
    }
}

trait IpParser<B> {
    fn parse(record: &[u8]) -> Option<SteppedIp<B>>;
}

/// Parses IP addresses from strings in their typical form.
struct StrParser;

impl IpParser<Ipv4Addr> for StrParser {
    fn parse(record: &[u8]) -> Option<SteppedIp<Ipv4Addr>> {
        CompactString::from_utf8(record)
            .ok()
            .and_then(|s| s.parse::<Ipv4Addr>().ok())
            .map(SteppedIp)
    }
}

impl IpParser<Ipv6Addr> for StrParser {
    fn parse(record: &[u8]) -> Option<SteppedIp<Ipv6Addr>> {
        CompactString::from_utf8(record)
            .ok()
            .and_then(|s| s.parse::<Ipv6Addr>().ok())
            .map(SteppedIp)
    }
}

/// Parses IP addresses from strings where IPs are converted to u32 to u128 integers.
struct NumParser;

impl IpParser<Ipv4Addr> for NumParser {
    fn parse(record: &[u8]) -> Option<SteppedIp<Ipv4Addr>> {
        CompactString::from_utf8(record)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .map(Ipv4Addr::from_bits)
            .map(SteppedIp)
    }
}

impl IpParser<Ipv6Addr> for NumParser {
    fn parse(record: &[u8]) -> Option<SteppedIp<Ipv6Addr>> {
        CompactString::from_utf8(record)
            .ok()
            .and_then(|s| s.parse::<u128>().ok())
            .map(Ipv6Addr::from_bits)
            .map(SteppedIp)
    }
}

/// A memory-packed representation of a lat/lng coordinate.
///
/// TODO: take advantage of info density of lat/lng to make this smaller in memory?
#[derive(Debug, Clone, Copy)]
struct PackedCoordinate {
    lat: f32,
    lng: f32,
}

impl TryFrom<&ByteRecord> for PackedCoordinate {
    type Error = Error;

    fn try_from(record: &ByteRecord) -> Result<Self, Error> {
        Ok(Self {
            lat: record
                .get(LATITUDE_IDX)
                .map(CompactString::from_utf8_lossy)
                .and_then(|s| s.parse::<f32>().ok())
                .ok_or(Error::InvalidFormat)?,
            lng: record
                .get(LONGITUDE_IDX)
                .map(CompactString::from_utf8_lossy)
                .and_then(|s| s.parse::<f32>().ok())
                .ok_or(Error::InvalidFormat)?,
        })
    }
}

impl PartialEq for PackedCoordinate {
    fn eq(&self, other: &Self) -> bool {
        self.lat.to_ne_bytes() == other.lat.to_ne_bytes()
            && self.lng.to_ne_bytes() == other.lng.to_ne_bytes()
    }
}
impl Eq for PackedCoordinate {}

impl std::hash::Hash for PackedCoordinate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lat.to_ne_bytes().hash(state);
        self.lng.to_ne_bytes().hash(state);
    }
}

impl From<PackedCoordinate> for Coordinate {
    fn from(coord: PackedCoordinate) -> Self {
        Self {
            lat: coord.lat,
            lng: coord.lng,
        }
    }
}

/// Number of strings in the database must be less than u32::MAX
type StringDictKey = u32;

/// The city and region are stored as indexes into a string database.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct LocationIndices {
    city: StringDictKey,
    region: StringDictKey,
    country_code: CountryCode,
}

/// An ISO 3166 2-digit ASCII country code.
// Takes advantage of their compact representation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
struct CountryCode([u8; 2]);

impl<A: AsRef<[u8]>> From<A> for CountryCode {
    fn from(value: A) -> Self {
        match value.as_ref() {
            [a, b, ..] => Self([*a, *b]),
            _ => Self([0, 0]),
        }
    }
}

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            [0, 0] => "??".fmt(f),
            [a, b] => unsafe {
                char::from_u32_unchecked(a as u32).fmt(f)?;
                char::from_u32_unchecked(b as u32).fmt(f)
            },
        }
    }
}
