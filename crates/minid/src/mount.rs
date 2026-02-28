//! Rootfs and mount namespace setup.
//!
//! Handles bind-mounting the container rootfs, mounting essential
//! virtual filesystems (`/proc`, `/dev`), and performing `pivot_root`.

use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use nix::mount::{self, MntFlags, MsFlags};
use nix::unistd;
use tracing::debug;

/// Set up the container's root filesystem.
///
/// 1. Bind-mount rootfs onto itself (required for pivot_root).
/// 2. Mount `/proc` inside the new root.
/// 3. Create minimal `/dev` entries.
/// 4. `pivot_root` into the new root.
/// 5. Unmount the old root and remove it.
pub fn setup_rootfs(rootfs: &Path) -> crate::Result<()> {
    debug!(?rootfs, "setting up container rootfs");

    // 1. Bind mount rootfs to itself so pivot_root can work.
    mount::mount(
        Some(rootfs),
        rootfs,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )?;

    // 2. Mount /proc.
    let proc_path = rootfs.join("proc");
    fs::create_dir_all(&proc_path)?;
    mount::mount(
        Some("proc"),
        &proc_path,
        Some("proc"),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC,
        None::<&str>,
    )?;

    // 3. Create minimal /dev directory with essential device nodes.
    let dev_path = rootfs.join("dev");
    fs::create_dir_all(&dev_path)?;
    create_dev_symlinks(&dev_path)?;

    // 4. pivot_root: make rootfs the new /.
    let old_root = rootfs.join("oldroot");
    fs::create_dir_all(&old_root)?;

    debug!("performing pivot_root");
    unistd::pivot_root(rootfs, &old_root)?;

    // 5. Change directory to new root and unmount old root.
    unistd::chdir("/")?;
    mount::umount2("/oldroot", MntFlags::MNT_DETACH)?;
    fs::remove_dir("/oldroot")?;

    Ok(())
}

/// Create essential symlinks in /dev (stdin, stdout, stderr, null).
fn create_dev_symlinks(dev_path: &Path) -> crate::Result<()> {
    let links = [
        ("fd", "/proc/self/fd"),
        ("stdin", "/proc/self/fd/0"),
        ("stdout", "/proc/self/fd/1"),
        ("stderr", "/proc/self/fd/2"),
    ];

    for (name, target) in &links {
        let link = dev_path.join(name);
        if !link.exists() {
            debug!(link = ?link, target, "creating dev symlink");
            unix_fs::symlink(target, &link)?;
        }
    }

    Ok(())
}
