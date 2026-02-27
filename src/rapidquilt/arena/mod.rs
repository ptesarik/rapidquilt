use std::io;
use std::fmt;
use std::path::Path;

mod file_arena;

#[cfg(unix)]
mod mmap_arena;

pub use self::file_arena::FileArena;

#[cfg(unix)]
pub use self::mmap_arena::MmapArena;


pub trait Arena: Sync {
    /// Load the file and return byte slice of its complete content. The slice
    /// is valid as long as this object is alive. (Same lifetimes.)
    fn load_file(&self, path: &Path) -> Result<&[u8], io::Error>;

    /// Get statistics
    fn stats(&self) -> Stats;
}

pub struct Stats {
    loaded_files: usize,
    total_size: usize,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Arena Statistics (loaded files: {}, total size: {} B)", self.loaded_files, self.total_size)
    }
}

#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::Write;

#[cfg(test)]
fn test_empty(arena: &dyn Arena) {
    let stats = arena.stats();
    assert_eq!(stats.loaded_files, 0);
    assert_eq!(stats.total_size, 0);
}

#[cfg(test)]
fn test_regular(arena: &dyn Arena) -> Result<(), io::Error> {
    let work_dir = tempfile::tempdir()?;

    // A normal text file
    let path = work_dir.path().join("regular");
    let write_regular = b"Some content
Second line
Third line
";
    let mut file = File::create(&path)?;
    file.write_all(write_regular)?;

    let read_regular = arena.load_file(&path)?;
    assert_eq!(write_regular, read_regular);

    let stats = arena.stats();
    assert_eq!(stats.loaded_files, 1);
    assert_eq!(stats.total_size, write_regular.len());

    // A binary file (cannot be decoded as UTF-8)
    let path = work_dir.path().join("non-utf8");
    let write_non_utf8 = b"Some non-UTF8 binary content
Invalid byte 0xc0: \xc0
Invalid byte 0xff: \xff
Invalid sequence 0xc0 0x10: \xc0\x10
I could add more, but I only care if non-UTF-8 content can be loaded.
";
    let mut file = File::create(&path)?;
    file.write_all(write_non_utf8)?;

    let read_non_utf8 = arena.load_file(&path)?;
    assert_eq!(write_non_utf8, read_non_utf8);

    let stats = arena.stats();
    assert_eq!(stats.loaded_files, 2);
    assert_eq!(stats.total_size, write_regular.len() + write_non_utf8.len());

    Ok(())
}
