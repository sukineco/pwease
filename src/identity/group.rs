use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::ops::{Deref, DerefMut};

use clap::builder::ValueParserFactory;

/// System group.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    gid: u32,
    name: String,
}

impl Group {
    /// Get a group by the associated group structure.
    pub fn get(group: &libc::group) -> Self {
        Self {
            gid: group.gr_gid,
            name: unsafe { CStr::from_ptr(group.gr_name) }
                .to_string_lossy()
                .into_owned(),
        }
    }

    /// Get a group by its gid.
    pub fn from_gid(uid: u32) -> Option<Self> {
        let passwd = unsafe { libc::getgrgid(uid).as_ref() };
        passwd.map(Self::get)
    }

    /// Get a group by its name.
    pub fn from_name(name: impl AsRef<str>) -> Option<Self> {
        let name = CString::new(name.as_ref()).ok()?;
        let group = unsafe { libc::getgrnam(name.as_ptr()).as_ref() };
        group.map(Self::get)
    }
}

impl ValueParserFactory for Group {
    type Parser = fn(&str) -> clap::error::Result<Group>;

    fn value_parser() -> Self::Parser {
        |arg| {
            use clap::error::*;
            arg.parse::<u32>()
                .ok()
                .and_then(Group::from_gid)
                .or_else(|| Group::from_name(arg))
                .ok_or(Error::new(ErrorKind::ValueValidation))
        }
    }
}

/// A set of auxiliary groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Groups {
    groups: HashSet<Group>,
}

impl Groups {
    /// Create a new set of auxiliary groups from vector of groups.
    pub fn new(groups: impl IntoIterator<Item = Group>) -> Self {
        Self {
            groups: groups.into_iter().collect(),
        }
    }

    /// Create an empty set of auxiliary groups.
    pub fn empty() -> Self {
        Self {
            groups: HashSet::new(),
        }
    }

    /// Get current auxiliary groups.
    pub fn current() -> Self {
        let size = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
        let mut groups = Vec::with_capacity(size as usize);

        let len = unsafe { libc::getgroups(groups.capacity() as i32, groups.as_mut_ptr()) };
        assert!(len >= 0);
        unsafe { groups.set_len(len as usize) };

        Self::new(groups.into_iter().map(|grp| Group::from_gid(grp).unwrap()))
    }

    /// Set current auxiliary groups.
    pub fn try_login(&self) -> std::io::Result<()> {
        let groups = self
            .groups
            .iter()
            .map(|grp| grp.to_owned().gid)
            .collect::<Vec<u32>>();

        if unsafe { libc::setgroups(self.groups.len(), groups.as_ptr() as _) } < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Deref for Groups {
    type Target = HashSet<Group>;

    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}

impl DerefMut for Groups {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.groups
    }
}

impl IntoIterator for Groups {
    type Item = Group;
    type IntoIter = <HashSet<Group> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.groups.into_iter()
    }
}
