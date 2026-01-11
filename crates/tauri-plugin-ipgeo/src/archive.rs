//! Tools for using files as immutable memory-mapped databases with [`rkyv`] types.

use std::{
    fs::{self, File, OpenOptions},
    hash::Hasher,
    io::{BufWriter, Write},
    marker::PhantomData,
    ops::Deref,
    path::{Path, PathBuf},
};

use memmap2::{Mmap, MmapOptions};
use rkyv::{
    Archive, Serialize,
    rancor::{self, Strategy},
    ser::{allocator::ArenaHandle, writer::IoWriter},
};
use rustc_hash::FxHasher;
use std::io::{Read, Seek, SeekFrom};
use time::UtcDateTime;

const HASH_BUF_SIZE: usize = 64 * 1024; // 64 KB buffer
const CHECKSUM_SIZE: u64 = 8;

const EXTENSION: &str = "res";

/// Returns an iterator over the resources possibly created in a directory.
pub fn resource_dir_list(dir: &Path) -> anyhow::Result<impl Iterator<Item = (PathBuf, u64)>> {
    let r = fs::read_dir(&dir)?
        .filter_map(|d| d.ok())
        .filter(|d| {
            d.path().extension().is_some_and(|ext| ext == EXTENSION)
                && d.file_type().is_ok_and(|ft| ft.is_file())
        })
        .filter_map(|p| {
            p.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.parse::<u64>().ok())
                .map(|s| (p.path(), s))
        });

    Ok(r)
}

/// Represents a memory-mapped, checksummed [`rkyv`] archive file resource
/// for any [`Serialize`] and [`Archive`] type.
///
/// This struct provides safe access to a serialized archive stored on disk,
/// ensuring data integrity via a checksum appended to the file. The archive
/// is memory-mapped for efficient zero-copy access.
pub struct FileResource<T> {
    view: Mmap,
    checksum: u64,
    path: PathBuf,
    _marker: PhantomData<T>,
}

impl<T> FileResource<T>
where
    T: for<'a> Serialize<
        Strategy<
            rkyv::ser::Serializer<
                IoWriter<BufWriter<File>>,
                ArenaHandle<'a>,
                rkyv::ser::sharing::Share,
            >,
            rancor::Error,
        >,
    >,
{
    /// Creates a new archive file resource from the given data, writing it to the specified cache directory.
    ///
    /// The resulting file's name is generated based on the checksum of the data, it can be found at [`FileResource::path`].
    ///
    pub fn create(dir: impl AsRef<Path>, data: &T) -> anyhow::Result<Self> {
        if let Some(parent) = dir.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = dir
            .as_ref()
            .join(UtcDateTime::now().unix_timestamp().to_string());

        // 1. Write archive data
        {
            let io_writer = IoWriter::new(BufWriter::new(File::create(&temp_path)?));
            let mut bw =
                rkyv::api::high::to_bytes_in::<_, rancor::Error>(data, io_writer)?.into_inner();
            bw.flush()?;
            bw.into_inner()?.sync_all()?;
        }

        // 2. Hash the file
        let checksum = {
            let mut file = File::open(&temp_path)?;
            file.seek(SeekFrom::Start(0))?;
            let mut hasher = FxHasher::default();
            let mut buf = [0u8; HASH_BUF_SIZE];
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.write(&buf[..n]);
            }
            hasher.finish()
        };

        // 3. Rename to <cache dir>/<checksum>.ipgeodb
        let final_path = dir
            .as_ref()
            .join(checksum.to_string())
            .with_extension(EXTENSION);

        if final_path.exists() {
            anyhow::bail!("Archive already exists");
        }

        fs::rename(&temp_path, &final_path)?;

        // 4. Append checksum
        {
            let mut file = OpenOptions::new().append(true).open(&final_path)?;
            file.write_all(&checksum.to_ne_bytes())?;
            file.sync_all()?;
        }

        // 5. Re-open for mmap
        let file = File::open(&final_path)?;
        let view = unsafe {
            MmapOptions::new()
                .len((file.metadata()?.len() - CHECKSUM_SIZE) as usize)
                .map(&file)?
        };

        Ok(Self {
            view,
            checksum,
            path: final_path,
            _marker: PhantomData,
        })
    }

    /// Opens an existing archive file resource from the specified path, verifying its checksum.
    ///
    /// This must be a file previously created with the associated [`FileResource::create`].
    ///
    pub fn open(path: &PathBuf) -> anyhow::Result<Self> {
        let Some(expected) = path
            .file_stem()
            .and_then(|x| x.to_str())
            .and_then(|x| x.parse::<u64>().ok())
        else {
            anyhow::bail!("Incorrect archive checksum name");
        };

        let mut file = File::open(path)?;
        let data_len = file.metadata()?.len() - CHECKSUM_SIZE;

        let Some(checksum) = verify_checksum(&mut file)?.filter(|x| *x == expected) else {
            anyhow::bail!("Archive file checksum mismatch: possible corruption");
        };

        // Memory-map only the data portion (not the checksum)
        let file = File::open(path)?;
        let view = unsafe { MmapOptions::new().len(data_len as usize).map(&file)? };

        tracing::debug!("mapped archive at {path:?}");

        Ok(Self {
            view,
            checksum,
            path: path.clone(),
            _marker: PhantomData,
        })
    }
}

impl<T> FileResource<T>
where
    T: Archive,
{
    /// Returns the inner data of the archive file resource.
    pub fn inner(&self) -> &<T as Archive>::Archived {
        unsafe { rkyv::access_unchecked(&self.view) }
    }
}

impl<T> FileResource<T> {
    /// Returns the checksum associated with this archive file resource.
    pub fn checksum(&self) -> u64 {
        self.checksum
    }

    /// The path to the underlying archive file resource.
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Deletes the underlying archive file from disk, destructing the object.
    pub fn delete(self) -> anyhow::Result<()> {
        fs::remove_file(self.path)?;

        Ok(())
    }
}

impl<T> Deref for FileResource<T>
where
    T: Archive,
{
    type Target = <T as Archive>::Archived;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

/// Hash the contents and compare the result to the hash stored at the
/// end to verify the integrity of the archive.
fn verify_checksum<R: Read + Seek>(reader: &mut R) -> std::io::Result<Option<u64>> {
    let len = reader.seek(SeekFrom::End(0))?;

    if len < CHECKSUM_SIZE as u64 {
        return Ok(None);
    }

    let data_len = len - CHECKSUM_SIZE as u64;

    reader.seek(SeekFrom::End(-(CHECKSUM_SIZE as i64)))?;
    let mut checksum = [0u8; CHECKSUM_SIZE as usize];
    reader.read_exact(&mut checksum)?;
    let expected = u64::from_ne_bytes(checksum);

    reader.seek(SeekFrom::Start(0))?;

    let mut hasher = FxHasher::default();
    let mut buf = [0u8; HASH_BUF_SIZE];
    let mut remaining = data_len;

    while remaining > 0 {
        let read_len = buf.len().min(remaining as usize);

        let n = reader.read(&mut buf[..read_len])?;
        if n == 0 {
            break;
        }

        hasher.write(&buf[..n]);
        remaining -= n as u64;
    }

    Ok((hasher.finish() == expected).then_some(expected))
}
