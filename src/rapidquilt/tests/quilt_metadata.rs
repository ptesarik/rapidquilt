use crate::cmd;

use std::fs;

use std::ffi::OsStr;
use std::path::Path;
use std::io::Read;
use anyhow::{anyhow, Context, Result};
use paste::paste;

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    for entry in fs::read_dir(from).context(format!("Copying {:?}", from))? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = to.join(entry.file_name());

        if src_path.is_symlink() {
            let target = fs::read_link(&src_path)?;
	    #[cfg(unix)]
	    {
		use std::os::unix::fs::symlink;
		symlink(target, &dest_path)
                    .context(format!("Creating symlink {:?} under {:?}", dest_path, to))?;
	    }
	    #[cfg(windows)]
	    {
		use std::os::windows::fs::symlink_file;
		symlink_file(target, &dest_path)
		    .context(format!("Creating symlink {:?} under {:?}", dest_path, to))?;
	    }
        } else if src_path.is_file() {
            fs::copy(&src_path, &dest_path)
                .context(format!("Copying {:?} under {:?}", src_path, to))?;
        } else if src_path.is_dir() {
            fs::create_dir(&dest_path)
                .context(format!("Creating directory {:?}", dest_path))?;
            copy_tree(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

fn compare_tree(src: &Path, dst: &Path) -> Result<()> {
    for entry in fs::read_dir(src).context(format!("Reading {:?}", src))? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dst.join(entry.file_name());
        let src_meta = fs::symlink_metadata(&src_path)
            .context(format!("Querying {:?} metadata", src_path))?;
        let dest_meta = fs::symlink_metadata(&dest_path)
            .context(format!("Querying {:?} metadata", dest_path))?;
        if src_meta.permissions() != dest_meta.permissions() {
            eprintln!("Mismatch in {:?}", entry.file_name());
            eprintln!("  expected: {:?}", src_meta.permissions());
            eprintln!("  actual: {:?}", dest_meta.permissions());

            panic!("Permission mismatch at {}", src.display());
        }

        if src_meta.is_symlink() {
            let src_target = fs::read_link(&src_path)
		.context(format!("Reading symlink {:?}", src_path))?;
            let dest_target = fs::read_link(&dest_path)
		.context(format!("Reading symlink {:?}", dest_path))?;
            if src_target != dest_target {
                panic!("Symlink target mismatch at {}: expected {:?}, actual {:?}", dest_path.display(), src_target, dest_target);
            }
        } else if src_meta.is_file() {
            let mut src_file = std::fs::File::open(&src_path)
                .context(format!("Opening {:?}", src_path))?;
            let mut dest_file = std::fs::File::open(&dest_path)
                .context(format!("Opening {:?}", dest_path))?;

            let mut expected = Vec::new();
            src_file.read_to_end(&mut expected)
                .context(format!("Reading {:?}", src_path))?;
            let mut actual = Vec::new();
            dest_file.read_to_end(&mut actual)
                .context(format!("Reading {:?}", dest_path))?;
            if actual != expected {
                eprintln!("Mismatch in {:?}", entry.file_name());
                eprintln!("<<< EXPECTED\n{}",
                          String::from_utf8_lossy(&expected));
                eprintln!("=== ACTUAL\n{}",
                          String::from_utf8_lossy(&actual));
                eprintln!(">>>");

                panic!("Content mismatch at {}", src.display());
            }
        } else if src_meta.is_dir() {
            compare_tree(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

fn check_extra_files(src: &Path, dst: &Path) -> Result<()> {
    let mut errors = Vec::<String>::new();
    for entry in fs::read_dir(dst).context(format!("Reading {:?}", dst))? {
        let entry = entry?;
        let dst_path = entry.path();
        let src_path = src.join(entry.file_name());
        if let Err(_) = src_path.symlink_metadata() {
            errors.push(format!("Unexpected file {:?}", dst_path));
        } else if dst_path.is_dir() {
            check_extra_files(&src_path, &dst_path)?;
        }
    }
    match errors.len() {
        0 => Ok(()),
        _ => Err(anyhow!(errors.join("\n"))),
    }
}

fn push_all(path: &Path, num_threads: usize, expect: bool) -> Result<()> {
    eprintln!("Push all patches in {}", path.display());

    let work_dir = tempfile::tempdir()?;
    let work_path = work_dir.path();
    copy_tree(&path.join("input"), &work_path)?;

    let num_threads = num_threads.to_string();
    let args = [
        OsStr::new("push"),
        OsStr::new("--quiet"),
        OsStr::new("--threads"), OsStr::new(&num_threads),
        OsStr::new("--all"),
        OsStr::new("--directory"), work_path.as_os_str(),
        OsStr::new("--backup"), OsStr::new("always"),
    ];
    let result = cmd::run(&args);

    match result {
        Ok(status) if status == expect => {
            compare_tree(&path.join("expect"), &work_path)?;
            check_extra_files(&path.join("expect"), &work_path)
        },
        Ok(_) => Err(anyhow!(match expect {
            true => "Push failed unexpectedly",
            false => "Push was expected to fail but it did not",
        })),
        Err(err) => Err(err)
    }
}

const BASE_PATH: &str = "testdata/quilt";
const NUM_THREADS: usize = 2;

macro_rules! series_path {
    ( $expect:ident, $name:ident ) => {
	Path::new(BASE_PATH)
	    .join(stringify!($expect))
	    .join(stringify!($name).replace("_", "-"))
    };
}

macro_rules! parallel_threads {
    ( sequential ) => { 1 };
    ( parallel ) => { NUM_THREADS };
}

macro_rules! expect_bool {
    ( ok ) => { true };
    ( fail) => { false };
}

macro_rules! make_test {
    ( $expect:ident, $name:ident, $parallel:ident ) => {
	paste!{
	    #[test]
	    fn [< $expect _ $name _ $parallel >]() -> Result<()> {
		push_all(&series_path!($expect, $name),
			 parallel_threads!($parallel),
			 expect_bool!($expect))
	    }
	}
    }
}

macro_rules! check_series {
    ( $expect:ident, $name:ident ) => {
	make_test!{$expect, $name, sequential}
	make_test!{$expect, $name, parallel}
    }
}

check_series!(ok, basic);
check_series!(ok, cleandir);
check_series!(ok, create);
check_series!(ok, double);
check_series!(ok, double_readonly);
check_series!(ok, perms);
check_series!(ok, symlink);
check_series!(ok, zerolen);

check_series!(fail, file_to_symlink);
check_series!(fail, mismatch);
check_series!(fail, overlap_rollback);
check_series!(fail, restore_truncated);
check_series!(fail, symlink_nomode);
