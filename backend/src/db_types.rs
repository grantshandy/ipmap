#[derive(Clone, serde::Deserialize)]
pub struct CityRecordIpv4Num {
    pub ip_range_start: u32,
    pub ip_range_end: u32,
    country_code: Option<String>,
    state1: Option<String>,
    _state2: Option<String>,
    city: Option<String>,
    _postcode: Option<String>,
    latitude: Option<f32>,
    longitude: Option<f32>,
    timezone: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Location {
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub timezone: Option<String>,
    pub state: Option<String>,
}
impl Eq for Location {}

impl From<CityRecordIpv4Num> for Location {
    fn from(other: CityRecordIpv4Num) -> Self {
        Self {
            latitude: other.latitude,
            longitude: other.longitude,
            city: other.city,
            country_code: other.country_code,
            timezone: other.timezone,
            state: other.state1,
        }
    }
}

pub type GeoDb = rangemap::RangeInclusiveMap<u32, Location>;
