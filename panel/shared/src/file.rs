use std::{
    fmt::{self, Display},
    num::ParseIntError,
    str::FromStr,
    sync::Arc,
};

use leptos_router::{IntoParam, ParamsError};
use serde::{de::Visitor, Deserialize};

/// A unique identifier for a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FileId(u64);

impl FileId {
    /// Generate a new random file id.
    pub fn random() -> Self {
        Self(rand::random())
    }
}

impl Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl FromStr for FileId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(s, 16).map(FileId)
    }
}

impl IntoParam for FileId {
    fn into_param(value: Option<&str>, name: &str) -> Result<Self, ParamsError> {
        let Some(value) = value else {
            return Err(ParamsError::MissingParam(name.to_string()));
        };

        FileId::from_str(value).map_err(|err| ParamsError::Params(Arc::new(err)))
    }
}

impl<'de> Deserialize<'de> for FileId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = FileId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a hex-encoded FileId")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                FileId::from_str(v).map_err(|_| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &"a 64-bit hex-encoded integer",
                    )
                })
            }
        }
        deserializer.deserialize_str(V)
    }
}
