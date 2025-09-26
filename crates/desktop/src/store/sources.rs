use compact_str::CompactString;

use crate::store::NamedDatabaseMetadata;

pub struct DatabaseSource {
    pub(crate) ipv4_urls: &'static [&'static str],
    pub(crate) ipv6_urls: &'static [&'static str],
    pub(crate) metadata: super::NamedDatabaseMetadata,
}

pub const DBIP_CITY: DatabaseSource = DatabaseSource {
    ipv4_urls: &[
        "https://github.com/sapics/ip-location-db/raw/refs/heads/main/dbip-city/dbip-city-ipv4-num.csv.gz",
        "https://unpkg.com/@ip-location-db/dbip-city/dbip-city-ipv4-num.csv.gz",
    ],
    ipv6_urls: &[
        "https://github.com/sapics/ip-location-db/raw/refs/heads/main/dbip-city/dbip-city-ipv6-num.csv.gz",
        "https://unpkg.com/@ip-location-db/dbip-city/dbip-city-ipv6-num.csv.gz",
    ],
    metadata: NamedDatabaseMetadata {
        display_name: CompactString::const_new("DB-IP City"),
        file_name: CompactString::const_new("dbip-city-all"),
        copyright: CompactString::const_new("Copyright DB-IP"),
    },
};

pub const GEOLITE2_CITY: DatabaseSource = DatabaseSource {
    ipv4_urls: &[
        "https://github.com/sapics/ip-location-db/raw/refs/heads/main/geolite2-city/geolite2-city-ipv4-num.csv.gz",
        "https://cdn.jsdelivr.net/npm/@ip-location-db/geolite2-city/geolite2-city-ipv4-num.csv.gz",
    ],
    ipv6_urls: &[
        "https://github.com/sapics/ip-location-db/raw/refs/heads/main/geolite2-city/geolite2-city-ipv6-num.csv.gz",
        "https://cdn.jsdelivr.net/npm/@ip-location-db/geolite2-city/geolite2-city-ipv6-num.csv.gz",
    ],
    metadata: NamedDatabaseMetadata {
        display_name: CompactString::const_new("Geolite2 City"),
        file_name: CompactString::const_new("geolite2-city-all"),
        copyright: CompactString::const_new("Copyright Geolite2"),
    },
};
