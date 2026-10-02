use clap::builder::{IntoResettable, OsStr, Resettable, ValueParserFactory};
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};

use crate::identity::group::{Group, Groups};

/// System user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    uid: u32,
    gid: u32,
    name: String,
    shell: PathBuf,
    home: PathBuf,
}

impl User {
    /// Get the root user.
    pub fn root() -> Self {
        Self::from_uid(0).expect("uid 0 does not exist somehow")
    }

    /// Get the current user.
    pub fn current() -> Self {
        let uid = unsafe { libc::getuid() };
        User::from_uid(uid).expect("current user does not exist somehow")
    }

    /// Get a user by the associated passwd structure.
    pub fn get(passwd: &libc::passwd) -> Self {
        Self {
            uid: passwd.pw_uid,
            gid: passwd.pw_gid,
            name: unsafe { CStr::from_ptr(passwd.pw_name) }
                .to_string_lossy()
                .into_owned(),
            shell: unsafe { CStr::from_ptr(passwd.pw_shell) }
                .to_string_lossy()
                .into_owned()
                .into(),
            home: unsafe { CStr::from_ptr(passwd.pw_dir) }
                .to_string_lossy()
                .into_owned()
                .into(),
        }
    }

    /// Get a user by its uid.
    pub fn from_uid(uid: u32) -> Option<Self> {
        let passwd = unsafe { libc::getpwuid(uid).as_ref() };
        passwd.map(Self::get)
    }

    /// Get a user by its name.
    pub fn from_name(name: impl AsRef<str>) -> Option<Self> {
        let name = CString::new(name.as_ref()).ok()?;
        let passwd = unsafe { libc::getpwnam(name.as_ptr()).as_ref() };
        passwd.map(Self::get)
    }

    /// Check if user is the root user.
    pub fn is_root(&self) -> bool {
        self.uid == 0
    }

    /// Get the login shell of the user.
    pub fn shell(&self) -> &Path {
        &self.shell
    }

    /// Get the home directory path of the user.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Get user's supplementary groups.
    pub fn groups(&self) -> Groups {
        let name = CString::new(self.name.as_bytes()).unwrap();

        let mut ngroups = 0i32;
        unsafe {
            libc::getgrouplist(
                name.as_ptr(),
                self.gid,
                std::ptr::null_mut(),
                &mut ngroups as *mut i32,
            );
        }

        let mut groups = Vec::with_capacity(ngroups as usize);
        unsafe {
            libc::getgrouplist(
                name.as_ptr(),
                self.gid,
                groups.as_mut_ptr(),
                &mut ngroups as *mut i32,
            );
            groups.set_len(ngroups as usize);
        }

        Groups::new(groups.into_iter().map(|grp| Group::from_gid(grp).unwrap()))
    }

    /// Set user as the current user via `setuid` and `setgid`.
    pub fn try_login(&self) -> std::io::Result<()> {
        if unsafe { libc::setgid(self.gid) } < 0 {
            return Err(std::io::Error::last_os_error());
        }

        if unsafe { libc::setuid(self.uid) } < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }
}

impl IntoResettable<OsStr> for User {
    fn into_resettable(self) -> Resettable<OsStr> {
        // NOTE(leak): Leaking is fine-ish ig cuz it's only used in clap builder.
        OsStr::from(self.name.clone().leak() as &_).into_resettable()
    }
}

impl ValueParserFactory for User {
    type Parser = fn(&str) -> clap::error::Result<User>;

    fn value_parser() -> Self::Parser {
        |arg| {
            use clap::error::*;
            arg.parse::<u32>()
                .ok()
                .and_then(User::from_uid)
                .or_else(|| User::from_name(arg))
                .ok_or(Error::new(ErrorKind::ValueValidation))
        }
    }
}
