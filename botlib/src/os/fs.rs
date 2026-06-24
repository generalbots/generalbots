use std::io;
use std::path::Path;

pub trait FilePermissions {
    fn set_executable(&self, path: &Path) -> io::Result<()>;
    fn set_readonly_owner(&self, path: &Path) -> io::Result<()>;
}

#[cfg(unix)]
pub struct UnixPermissions;

#[cfg(unix)]
impl FilePermissions for UnixPermissions {
    fn set_executable(&self, path: &Path) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)
    }

    fn set_readonly_owner(&self, path: &Path) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(path, perms)
    }
}

#[cfg(not(unix))]
pub struct NoOpPermissions;

#[cfg(not(unix))]
impl FilePermissions for NoOpPermissions {
    fn set_executable(&self, _path: &Path) -> io::Result<()> {
        // No-op on non-Unix systems
        Ok(())
    }

    fn set_readonly_owner(&self, _path: &Path) -> io::Result<()> {
        // No-op on non-Unix systems
        Ok(())
    }
}

pub fn get_permissions_manager() -> Box<dyn FilePermissions> {
    #[cfg(unix)]
    {
        Box::new(UnixPermissions)
    }
    #[cfg(not(unix))]
    {
        Box::new(NoOpPermissions)
    }
}
