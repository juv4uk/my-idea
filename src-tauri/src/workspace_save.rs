// Спільний файловий механізм GUI і Ratatui TUI. Без семантики SENS.
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path};

/// Змінює наявний файл лише після повного запису тимчасового у тому ж каталозі.
/// Не допускає вихід за workspace через '..' чи symlink. При помилці запису
/// старе джерело не обрізається, тимчасовий файл вилучається.
pub fn save_existing_workspace_file(root: &Path, relative: &Path, contents: &str) -> Result<(), String> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative.components().any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
    {
        return Err("workspace save path must remain relative".into());
    }
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let destination = root.join(relative).canonicalize().map_err(|error| error.to_string())?;
    if !destination.starts_with(&root) {
        return Err("path escapes the workspace".into());
    }
    if !destination.is_file() {
        return Err("only existing workspace files can be saved".into());
    }
    let permissions = fs::metadata(&destination)
        .map_err(|error| format!("could not read file permissions: {error}"))?
        .permissions();
    if permissions.readonly() {
        return Err("workspace file is read-only".into());
    }
    let parent = destination.parent().ok_or("workspace file has no parent directory")?;
    let name = destination.file_name().ok_or("workspace file has no file name")?
        .to_string_lossy();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?.as_nanos();

    for attempt in 0..16_u128 {
        let temporary = parent.join(format!(
            ".{name}.my-idea-save-{}-{}.tmp",
            std::process::id(),
            nonce + attempt
        ));
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&temporary) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("could not create save temporary file: {error}")),
        };
        let result = (|| -> std::io::Result<()> {
            file.write_all(contents.as_bytes())?;
            file.sync_all()?;
            file.set_permissions(permissions.clone())?;
            drop(file);
            fs::rename(&temporary, &destination)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        return result.map_err(|error| format!("workspace file was not saved: {error}"));
    }
    Err("could not allocate a unique save temporary file".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_last_saved_source_on_read_only_failure() {
        let root = std::env::temp_dir().join(format!(
            "my-idea-shared-save-{}-{:?}", std::process::id(), std::thread::current().id()
        ));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("program.lisp");
        fs::write(&source, "початковий код").unwrap();
        save_existing_workspace_file(&root, Path::new("program.lisp"), "оновлений код").unwrap();
        assert_eq!(fs::read_to_string(&source).unwrap(), "оновлений код");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        let mut perms = fs::metadata(&source).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&source, perms).unwrap();
        assert!(save_existing_workspace_file(&root, Path::new("program.lisp"), "втрата").is_err());
        assert_eq!(fs::read_to_string(&source).unwrap(), "оновлений код");
        let mut perms = fs::metadata(&source).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(&source, perms).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlink_escape_and_traversal() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!("my-idea-save-root-{}", std::process::id()));
        let outside = std::env::temp_dir().join(format!("my-idea-save-outside-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("protected.lisp"), "залишити").unwrap();
        symlink(outside.join("protected.lisp"), root.join("escape.lisp")).unwrap();
        assert!(save_existing_workspace_file(&root, Path::new("escape.lisp"), "заміна").is_err());
        assert!(save_existing_workspace_file(&root, Path::new("../escape.lisp"), "заміна").is_err());
        assert_eq!(fs::read_to_string(outside.join("protected.lisp")).unwrap(), "залишити");
        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(&outside).unwrap();
    }
}
