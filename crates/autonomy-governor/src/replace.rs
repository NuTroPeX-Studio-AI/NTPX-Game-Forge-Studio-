use std::{fs, io, path::Path};

/// Replace `target` with the already-durable same-directory `stage` file.
///
/// Unix uses the platform rename replacement primitive directly. Windows first attempts the same
/// operation and, when an existing target blocks replacement, falls back to a reversible
/// target-to-backup then stage-to-target sequence. The Windows fallback is failure-safe but is not
/// claimed to be kernel-atomic across the two renames.
pub(crate) fn replace_file(stage: &Path, target: &Path) -> io::Result<()> {
    replace_file_platform(stage, target)
}

#[cfg(not(windows))]
fn replace_file_platform(stage: &Path, target: &Path) -> io::Result<()> {
    fs::rename(stage, target)
}

#[cfg(windows)]
fn replace_file_platform(stage: &Path, target: &Path) -> io::Result<()> {
    let first_error = match fs::rename(stage, target) {
        Ok(()) => return Ok(()),
        Err(error) => error,
    };
    if !target.exists() {
        return Err(first_error);
    }

    let backup = stage.with_extension("gameforge-replace-backup");
    if backup.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "GameForge replacement backup already exists",
        ));
    }

    fs::rename(target, &backup)?;
    match fs::rename(stage, target) {
        Ok(()) => {
            fs::remove_file(backup)?;
            Ok(())
        }
        Err(stage_error) => match fs::rename(&backup, target) {
            Ok(()) => Err(stage_error),
            Err(restore_error) => Err(io::Error::new(
                restore_error.kind(),
                format!(
                    "replacement failed ({stage_error}) and target backup restore failed ({restore_error})"
                ),
            )),
        },
    }
}
