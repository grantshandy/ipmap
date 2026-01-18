//! Types that are stored directly on disk and sent to the user interface,
//! as referred to by [`DbState`](super::DbState).

use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use ipgeo::{
    ArchivedGenericDatabase, CombinedDatabase, Coordinate, Database, GenericDatabase, Location,
};

/// The base structure stored in the file, identifying a generic IP-geolocation database.
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct DiskArchive {
    pub source: DatabaseSource,
    pub db: DynamicDatabase,
}

/// Sources for where this database came from, as given to the user.
/// This allows us to de-duplicate common databases and download them
/// in-application.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    specta::Type,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DatabaseSource {
    DbIpCombined,
    Geolite2Combined,
    File(String),
}

/// A generic database type that can represent any kind of IP address database.
///
/// Its archive implements [`Database`] for [`Ipv4Addr`], [`Ipv6Addr`], and [`IpAddr`],
/// it's the job of [`DbSet`](super::DbSet)/[`DbState`](super::DbState) to keep
/// track of the database's address type for routing.
#[allow(clippy::large_enum_variant)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum DynamicDatabase {
    Combined(CombinedDatabase),
    Generic(GenericDatabase),
}

fn url_filename_guess(path: &str) -> &str {
    path.rsplit_once(['/', '\\'])
        .map(|(_, last)| last)
        .unwrap_or("unknown")
}

impl fmt::Display for DatabaseSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseSource::DbIpCombined => f.write_str("DB-IP City"),
            DatabaseSource::Geolite2Combined => f.write_str("Geolite2 City"),
            DatabaseSource::File(path) => f.write_str(url_filename_guess(path)),
        }
    }
}

impl fmt::Display for ArchivedDatabaseSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchivedDatabaseSource::DbIpCombined => f.write_str("DB-IP City"),
            ArchivedDatabaseSource::Geolite2Combined => f.write_str("Geolite2 City"),
            ArchivedDatabaseSource::File(path) => f.write_str(url_filename_guess(path)),
        }
    }
}

impl PartialEq<DatabaseSource> for ArchivedDatabaseSource {
    fn eq(&self, other: &DatabaseSource) -> bool {
        match (self, other) {
            (ArchivedDatabaseSource::File(path), DatabaseSource::File(other_path)) => {
                path == other_path
            }
            (ArchivedDatabaseSource::DbIpCombined, DatabaseSource::DbIpCombined) => true,
            (ArchivedDatabaseSource::Geolite2Combined, DatabaseSource::Geolite2Combined) => true,
            _ => false,
        }
    }
}

impl From<&ArchivedDatabaseSource> for DatabaseSource {
    fn from(value: &ArchivedDatabaseSource) -> Self {
        match value {
            ArchivedDatabaseSource::DbIpCombined => DatabaseSource::DbIpCombined,
            ArchivedDatabaseSource::Geolite2Combined => DatabaseSource::Geolite2Combined,
            ArchivedDatabaseSource::File(path) => DatabaseSource::File(path.to_string()),
        }
    }
}

impl Database<Ipv4Addr> for ArchivedDynamicDatabase {
    fn get_coordinate(&self, ip: Ipv4Addr) -> Option<Coordinate> {
        match self {
            ArchivedDynamicDatabase::Combined(db) => db.get_coordinate(ip.into()),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(db)) => {
                db.get_coordinate(ip)
            }
            _ => None,
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        match self {
            ArchivedDynamicDatabase::Combined(db) => db.get_location(crd),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(db)) => {
                db.get_location(crd)
            }
            _ => None,
        }
    }
}

impl Database<Ipv6Addr> for ArchivedDynamicDatabase {
    fn get_coordinate(&self, ip: Ipv6Addr) -> Option<Coordinate> {
        match self {
            ArchivedDynamicDatabase::Combined(db) => db.get_coordinate(ip.into()),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(db)) => {
                db.get_coordinate(ip)
            }
            _ => None,
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        match self {
            ArchivedDynamicDatabase::Combined(db) => db.get_location(crd),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(db)) => {
                db.get_location(crd)
            }
            _ => None,
        }
    }
}

impl Database<IpAddr> for ArchivedDynamicDatabase {
    fn get_coordinate(&self, ip: IpAddr) -> Option<Coordinate> {
        match (self, ip) {
            (ArchivedDynamicDatabase::Combined(db), ip) => db.get_coordinate(ip),
            (
                ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(db)),
                IpAddr::V4(ip),
            ) => db.get_coordinate(ip),
            (
                ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(db)),
                IpAddr::V6(ip),
            ) => db.get_coordinate(ip),
            _ => None,
        }
    }

    fn get_location(&self, crd: Coordinate) -> Option<Location> {
        match self {
            ArchivedDynamicDatabase::Combined(db) => db.get_location(crd),
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv4(db)) => {
                db.get_location(crd)
            }
            ArchivedDynamicDatabase::Generic(ArchivedGenericDatabase::Ipv6(db)) => {
                db.get_location(crd)
            }
        }
    }
}
