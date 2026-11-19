use rand::RngExt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::mem::DropGuard;
use std::path::{Path, PathBuf};

pub(super) fn write_target(path: &Path) -> io::Result<PathBuf> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_symlink() => fs::canonicalize(path),
        Ok(_) => Ok(path.to_path_buf()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(path.to_path_buf()),
        Err(err) => Err(err),
    }
}

pub(super) fn lock(path: &Path) -> io::Result<File> {
    let lock = options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_added_extension("lock"))?;
    lock.lock()?;
    Ok(lock)
}

pub(super) fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = unique_sibling(path, "tmp");
    let mut file = options().write(true).create_new(true).open(&temporary)?;
    let cleanup = DropGuard::new(temporary.as_path(), |path| {
        let _ = fs::remove_file(path);
    });
    let write_result = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    write_result?;
    fs::rename(&temporary, path)?;
    DropGuard::dismiss(cleanup);
    Ok(())
}

pub(super) fn unique_sibling(path: &Path, label: &str) -> PathBuf {
    let nonce = rand::rng().random::<u64>();
    path.with_added_extension(format!("{label}-{nonce:016x}"))
}

fn options() -> OpenOptions {
    let mut options = OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}
