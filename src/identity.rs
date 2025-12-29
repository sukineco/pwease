use std::ffi::CString;

mod cli;

#[derive(Debug, Clone)]
pub struct User {
    uid: u32,
    gid: u32,
}

impl User {
    pub fn from_uid(uid: u32) -> Option<Self> {
        let passwd = unsafe { libc::getpwuid(uid) };
        if passwd.is_null() {
            None
        } else {
            let gid = unsafe { *passwd }.pw_gid;
            Some(Self { uid, gid })
        }
    }

    pub fn from_name(name: String) -> Option<Self> {
        let passwd = unsafe { libc::getpwnam(CString::new(name).unwrap().as_ptr()) };
        if passwd.is_null() {
            None
        } else {
            let uid = unsafe { *passwd }.pw_uid;
            let gid = unsafe { *passwd }.pw_gid;
            Some(Self { uid, gid })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Group {
    gid: u32,
}

impl Group {
    pub fn from_gid(gid: u32) -> Option<Self> {
        let groups = unsafe { libc::getgrgid(gid) };
        if groups.is_null() {
            None
        } else {
            Some(Self { gid })
        }
    }

    pub fn from_name(name: String) -> Option<Self> {
        let groups = unsafe { libc::getgrnam(CString::new(name).unwrap().as_ptr()) };
        if groups.is_null() {
            None
        } else {
            let gid = unsafe { *groups }.gr_gid;
            Some(Self { gid })
        }
    }
}

pub fn get_user() -> User {
    User::from_uid(unsafe { libc::getuid() }).unwrap()
}

pub fn set_user(user: User) -> std::io::Result<()> {
    unsafe {
        if libc::setresuid(user.uid, user.uid, user.uid) < 0 {
            return Err(std::io::Error::last_os_error());
        }

        if libc::setresgid(user.gid, user.gid, user.gid) < 0 {
            return Err(std::io::Error::last_os_error());
        }
    }

    Ok(())
}

pub fn get_groups() -> std::io::Result<Vec<Group>> {
    let mut groups = Vec::with_capacity(65536);

    unsafe {
        // [NOTE] Group is repr(transparent) so it's safe to case it to u32
        let len = libc::getgroups(groups.capacity() as i32, groups.as_mut_ptr() as _);
        if len < 0 {
            return Err(std::io::Error::last_os_error());
        } else {
            groups.set_len(len as usize);
        }
    }

    Ok(groups)
}

pub fn set_groups(mut groups: Vec<Group>, extend: bool) -> std::io::Result<()> {
    let new_groups = if extend {
        let mut grps = get_groups()?;
        grps.append(&mut groups);
        drop(groups);
        grps
    } else {
        groups
    };

    unsafe {
        // [NOTE] Group is repr(transparent) so it's safe to case it to u32
        if libc::setgroups(new_groups.len(), new_groups.as_ptr() as _) < 0 {
            return Err(std::io::Error::last_os_error());
        }
    }

    Ok(())
}
