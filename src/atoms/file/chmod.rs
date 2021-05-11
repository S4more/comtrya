use super::super::Atom;
use super::FileAtom;
use std::path::PathBuf;
use tracing::error;

pub struct FilePermissions {
    path: PathBuf,
    mode: u32,
}

impl FileAtom for FilePermissions {
    fn get_path(&self) -> &PathBuf {
        &self.path
    }
}

impl std::fmt::Display for FilePermissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The permissions on {} need to be set to {}",
            self.path.to_str().unwrap(),
            self.mode
        )
    }
}

#[cfg(unix)]
use std::os::unix::prelude::PermissionsExt;

#[cfg(unix)]
impl Atom for FilePermissions {
    fn plan(&self) -> bool {
        let metadata = match std::fs::metadata(&self.path) {
            Ok(m) => m,
            Err(err) => {
                error!(
                    "Couldn't get metadata for {}, rejecting atom: {}",
                    &self.path.as_os_str().to_str().unwrap(),
                    err.to_string()
                );

                return false;
            }
        };

        // We expect permissions to come through as if the user was using chmod themselves.
        // This means we support 644/755 decimal syntax. We need to add 0o100000 to support
        // the part of chmod they don't often type.
        std::fs::Permissions::from_mode(0o100000 + self.mode).mode()
            != metadata.permissions().mode()
    }

    fn execute(&self) -> anyhow::Result<()> {
        std::fs::set_permissions(
            self.path.as_path(),
            std::fs::Permissions::from_mode(self.mode),
        )?;

        return Ok(());
    }
}

#[cfg(not(unix))]
impl Atom for FilePermissions {
    fn plan(&self) -> bool {
        false
    }

    fn execute(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn revert(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_plan() {
        let temp_dir = match tempfile::tempdir() {
            std::result::Result::Ok(dir) => dir,
            std::result::Result::Err(_) => {
                assert_eq!(false, true);
                return;
            }
        };

        match std::fs::File::create(temp_dir.path().join("644")) {
            std::result::Result::Ok(file) => file,
            std::result::Result::Err(_) => {
                assert_eq!(false, true);
                return;
            }
        };

        assert_eq!(
            true,
            std::fs::set_permissions(
                temp_dir.path().join("644"),
                std::fs::Permissions::from_mode(0o644)
            )
            .is_ok(),
        );

        let file_chmod = FilePermissions {
            path: temp_dir.path().join("644"),
            mode: 0o644,
        };

        assert_eq!(false, file_chmod.plan());

        let file_chmod = FilePermissions {
            path: temp_dir.path().join("644"),
            mode: 0o640,
        };

        assert_eq!(true, file_chmod.plan());
    }

    #[test]
    fn it_can_execute() {
        let temp_dir = match tempfile::tempdir() {
            std::result::Result::Ok(dir) => dir,
            std::result::Result::Err(_) => {
                assert_eq!(false, true);
                return;
            }
        };

        match std::fs::File::create(temp_dir.path().join("644")) {
            std::result::Result::Ok(file) => file,
            std::result::Result::Err(_) => {
                assert_eq!(false, true);
                return;
            }
        };

        assert_eq!(
            true,
            std::fs::set_permissions(
                temp_dir.path().join("644"),
                std::fs::Permissions::from_mode(0o644)
            )
            .is_ok(),
        );

        let file_chmod = FilePermissions {
            path: temp_dir.path().join("644"),
            mode: 0o644,
        };

        assert_eq!(false, file_chmod.plan());

        let file_chmod = FilePermissions {
            path: temp_dir.path().join("644"),
            mode: 0o640,
        };

        assert_eq!(true, file_chmod.plan());
        assert_eq!(true, file_chmod.execute().is_ok());
        assert_eq!(false, file_chmod.plan());
    }

    #[test]
    fn it_can_revert() {}
}
