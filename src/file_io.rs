use std::fs;
use std::io;
use std::path::Path;

/// Replace the file at `path` with `content` via the tempfile + rename idiom.
///
/// Crashes (SIGKILL, power loss) between the write and rename leave the
/// original file intact; readers either see the old content or the new one,
/// never a truncated-mid-write file. The rename itself is atomic at the
/// filesystem-entry level on POSIX (same directory keeps it on one fs).
///
/// This is NOT atomic in the CPU/threading sense — the bytes still go to
/// disk via many syscalls. The atomicity is in the directory entry swap.
pub fn replace_file(path: &Path, content: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no filename"))?;

    // PID-suffixed temp name in the same dir, so concurrent fragment runs
    // don't collide and the rename stays on one filesystem.
    let mut tmp_name = file_name.to_os_string();
    tmp_name.push(format!(".fragments-{}.tmp", std::process::id()));
    let tmp_path = parent.join(tmp_name);

    if let Err(e) = fs::write(&tmp_path, content) {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }
    if let Err(e) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }
    Ok(())
}
