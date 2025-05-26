use core::fmt;
use std::sync::LazyLock;

use chrono::{DateTime, Local, Locale, format::StrftimeItems};

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Meta {
    pub name: String,
    pub permissions_or_attributes: Option<PermissionsOrAttributes>,
    pub date: Option<Date>,
    pub owner: Option<Owner>,
    pub file_type: FileType,
    pub size: Option<u64>,
    pub inode: Option<u64>,
    pub content: Option<Vec<Meta>>,
}

#[derive(Clone, Debug)]
pub struct Owner {
    pub user: u32,
    pub group: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Date {
    Date(DateTime<Local>),
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        static LOCALE: LazyLock<Locale> = LazyLock::new(|| {
            std::env::var("LC_TIME")
                .ok()
                .and_then(|s| s.split('.').next().map(|s| s.to_string()))
                .unwrap_or_else(|| "en_US".to_string())
                .parse()
                .unwrap_or(Locale::en_US)
        });

        match self {
            Date::Date(date) => {
                write!(
                    f,
                    "{}",
                    date.format_localized_with_items(StrftimeItems::new("%c"), *LOCALE)
                )
            } // Date::Invalid => write!(f, "Invalid date"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(windows, allow(dead_code))]
pub enum FileType {
    BlockDevice,
    CharDevice,
    Directory { uid: bool },
    File { uid: bool, exec: bool },
    SymLink { is_dir: bool },
    Pipe,
    Socket,
}
#[derive(Clone, Debug)]
pub enum PermissionsOrAttributes {
    Permissions(Permissions),
}

impl fmt::Display for PermissionsOrAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionsOrAttributes::Permissions(permissions) => write!(f, "{permissions}"),
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
pub struct Permissions {
    pub user_read: bool,
    pub user_write: bool,
    pub user_execute: bool,

    pub group_read: bool,
    pub group_write: bool,
    pub group_execute: bool,

    pub other_read: bool,
    pub other_write: bool,
    pub other_execute: bool,

    pub sticky: bool,
    pub setgid: bool,
    pub setuid: bool,
}

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let user = format!(
            "{}{}{}",
            if self.user_read { 'r' } else { '-' },
            if self.user_write { 'w' } else { '-' },
            if self.user_execute { 'x' } else { '-' }
        );
        let group = format!(
            "{}{}{}",
            if self.group_read { 'r' } else { '-' },
            if self.group_write { 'w' } else { '-' },
            if self.group_execute { 'x' } else { '-' }
        );
        let other = format!(
            "{}{}{}",
            if self.other_read { 'r' } else { '-' },
            if self.other_write { 'w' } else { '-' },
            if self.other_execute { 'x' } else { '-' }
        );

        write!(f, "{user}{group}{other}")
    }
}

impl Permissions {
    pub fn from_mode(mode: u16) -> Self {
        let user_read = mode & 0o400 != 0;
        let user_write = mode & 0o200 != 0;
        let user_execute = mode & 0o100 != 0;

        let group_read = mode & 0o040 != 0;
        let group_write = mode & 0o020 != 0;
        let group_execute = mode & 0o010 != 0;

        let other_read = mode & 0o004 != 0;
        let other_write = mode & 0o002 != 0;
        let other_execute = mode & 0o001 != 0;

        let sticky = mode & 0o1000 != 0;
        let setgid = mode & 0o2000 != 0;
        let setuid = mode & 0o4000 != 0;

        Self {
            user_read,
            user_write,
            user_execute,
            group_read,
            group_write,
            group_execute,
            other_read,
            other_write,
            other_execute,
            sticky,
            setgid,
            setuid,
        }
    }
}
