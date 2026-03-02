// Licensed under the MIT license. See LICENSE.md

use std::marker::PhantomData;
use std::vec::Vec;
use std::io;
use std::fs::File;
use std::ptr;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::Mutex;
use std::mem::transmute;

use super::{Arena, Stats};



struct Mapping {
    start: *mut libc::c_void,
    size: usize,
}

enum Content {
    Mapped(Mapping),
    Boxed(Box<[u8]>),
}

/// Utility that reads files and keeps them loaded in immovable place in memory
/// for its lifetime. So the returned byte slices can be used as long as the
/// object of this struct is alive.
///
/// This implementation uses mmap, which means that if an external process
/// changes the file, the content of the memory may change or cause crash if
/// the file truncated.
pub struct MmapArena<'a> {
    contents: Mutex<Vec<Content>>,
    _phantom: PhantomData<&'a [u8]>,
}

// We have `*mut libc::c_void` in there, but we don't use it to mutate anything
// concurently. So no worries...
unsafe impl Sync for MmapArena<'_> {}

impl MmapArena<'_> {
    pub fn new() -> Self {
        Self {
            contents: Mutex::new(Vec::new()),
            _phantom: PhantomData,
        }
    }
}

impl<'a> Arena for MmapArena<'a> {
    /// Load the file and return byte slice of its complete content. The slice
    /// is valid as long as this object is alive. (Same lifetimes.)
    fn load_file(&self, path: &Path) -> Result<&[u8], io::Error> {
        let file = File::open(path)?;
        let size = file.metadata()?.len() as usize;
        let fd = file.as_raw_fd();

        let start = unsafe {
            let start = libc::mmap(ptr::null_mut(),
                size,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                fd,
                0
            );

            if start == libc::MAP_FAILED {
                return Err(io::Error::last_os_error());
            }

            start
        };

        let mapping = Mapping {
            start,
            size,
        };

        self.contents.lock().unwrap().push(Content::Mapped(mapping)); // NOTE(unwrap): If the lock is poisoned, some other thread panicked. We may as well.

        let slice = unsafe {
            std::slice::from_raw_parts::<'a>(start as *const u8, size)
        };

        Ok(slice)
    }

    /// Read a symbolic link and return byte slice of its value. The slice is
    /// valid as long as this object is alive. (Same lifetimes.)
    fn load_symlink(&self, path: &Path) -> Result<&[u8], io::Error> {
        let data = path.read_link()?.into_os_string().into_encoded_bytes().into_boxed_slice();

        let slice = unsafe {
            // We guarantee to the compiler that we will hold the content of the
            // Box for as long as we are alive. We will place the Box into the
            // `contents` Vec and we never delete items from there. Reallocating
            // the `contents` backing storage doesn't affect the content of the
            // Boxes.
            transmute::<&[u8], &'a [u8]>(&data)
        };

        self.contents.lock().unwrap().push(Content::Boxed(data)); // NOTE(unwrap): If the lock is poisoned, some other thread panicked. We may as well.

        Ok(slice)
    }

    /// Get statistics
    fn stats(&self) -> Stats {
        let contents = self.contents.lock().unwrap(); // NOTE(unwrap): If the lock is poisoned, some other thread panicked. We may as well.

        Stats {
            loaded_files: contents.len(),
            total_size: contents.iter().map(|item| match item {
                Content::Mapped(m) => m.size,
                Content::Boxed(d) => d.len(),
            }).sum(),
        }
    }
}

impl Drop for MmapArena<'_> {
    fn drop(&mut self) {
        if let Ok(contents) = self.contents.lock() {
            for mapping in contents.iter().filter_map(|item| match item {
                Content::Mapped(m) => Some(m),
                _ => None,
            }) {
                unsafe {
                    libc::munmap(mapping.start, mapping.size);
                }
            }
        }
    }
}

#[cfg(test)]
#[test]
fn test_empty() {
    super::test_empty(&MmapArena::new())
}

#[cfg(test)]
#[test]
fn test_regular() -> Result<(), io::Error> {
    super::test_regular(&MmapArena::new())
}

#[cfg(test)]
#[test]
fn test_directory() -> Result<(), io::Error> {
    let work_dir = tempfile::tempdir()?;
    let arena = MmapArena::new();
    let content = arena.load_file(&work_dir.path());
    assert!(matches!(content, Err(error) if error.raw_os_error() == Some(libc::ENODEV)));

    Ok(())
}

#[cfg(test)]
#[test]
fn test_symlink() -> Result<(), io::Error> {
    super::test_symlink(&MmapArena::new())
}
