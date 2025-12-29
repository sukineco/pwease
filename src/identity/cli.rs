use clap::builder::{NonEmptyStringValueParser, TypedValueParser};

use crate::identity::{Group, User};

impl clap::builder::ValueParserFactory for User {
    type Parser = UserValueParser;

    fn value_parser() -> Self::Parser {
        UserValueParser
    }
}

#[derive(Debug, Clone)]
pub struct UserValueParser;

impl TypedValueParser for UserValueParser {
    type Value = User;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        use clap::error::*;

        let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
        err.insert(
            ContextKind::InvalidArg,
            ContextValue::String(arg.unwrap().to_string()),
        );
        err.insert(
            ContextKind::InvalidValue,
            ContextValue::String(value.to_string_lossy().to_string()),
        );

        match clap::value_parser!(u32).parse_ref(cmd, arg, value) {
            Ok(uid) => User::from_uid(uid).ok_or(err),
            Err(_) => match NonEmptyStringValueParser::new().parse_ref(cmd, arg, value) {
                Ok(name) => User::from_name(name).ok_or(err),
                Err(err) => Err(err),
            },
        }
    }
}

impl clap::builder::ValueParserFactory for Group {
    type Parser = GroupValueParser;

    fn value_parser() -> Self::Parser {
        GroupValueParser
    }
}

#[derive(Debug, Clone)]
pub struct GroupValueParser;

impl TypedValueParser for GroupValueParser {
    type Value = Group;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        use clap::error::*;

        let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
        err.insert(
            ContextKind::InvalidArg,
            ContextValue::String(arg.unwrap().to_string()),
        );
        err.insert(
            ContextKind::InvalidValue,
            ContextValue::String(value.to_string_lossy().to_string()),
        );

        match clap::value_parser!(u32).parse_ref(cmd, arg, value) {
            Ok(uid) => Group::from_gid(uid).ok_or(err),
            Err(_) => match NonEmptyStringValueParser::new().parse_ref(cmd, arg, value) {
                Ok(name) => Group::from_name(name).ok_or(err),
                Err(err) => Err(err),
            },
        }
    }
}
