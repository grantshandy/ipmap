use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use memmap2::Mmap;
use rkyv::{rancor, ser::writer::IoWriter};

use super::{DatabaseSource, DynamicDatabase};

#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct DiskArchive {
    pub source: DatabaseSource,
    pub db: DynamicDatabase,
}

/// Open an rkyv archive type as a file resource, memmapped.
pub struct FileArchive {
    file: File,
    view: Mmap,
    path: PathBuf,
}

impl FileArchive {
    pub fn create(path: &PathBuf, data: &DiskArchive) -> anyhow::Result<FileArchive> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(&path)?;

        tracing::debug!("created archive at {path:?}");

        let io_writer = IoWriter::new(BufWriter::new(file));
        let mut bw =
            rkyv::api::high::to_bytes_in::<_, rancor::Error>(data, io_writer)?.into_inner();
        bw.flush()?;

        let file = bw.into_inner()?;
        file.sync_all()?;

        drop(file);
        let file = File::open(path)?;

        tracing::debug!("wrote archive to {path:?}");

        let view = unsafe { Mmap::map(&file)? };

        tracing::debug!("mapped archive at {path:?}");

        Ok(Self {
            file,
            view,
            path: path.clone(),
        })
    }

    pub fn open(path: &PathBuf) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let view = unsafe { Mmap::map(&file)? };

        tracing::debug!("mapped archive at {path:?}");

        Ok(Self {
            file,
            view,
            path: path.clone(),
        })
    }

    pub fn get_data(&self) -> &ArchivedDiskArchive {
        unsafe { rkyv::access_unchecked::<ArchivedDiskArchive>(&self.view) }
    }

    pub fn delete(self) -> anyhow::Result<()> {
        fs::remove_file(self.path)?;

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
