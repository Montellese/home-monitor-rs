use std::fmt;
use std::str::FromStr;

use macaddr::{MacAddr6, MacAddr8, ParseError};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

// NOTE: this is basically a copy of macaddr::MacAddr with additional serde support
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum MacAddr {
    V6(MacAddr6),
    V8(MacAddr8),
}

impl MacAddr {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            MacAddr::V6(addr) => addr.as_bytes(),
            MacAddr::V8(addr) => addr.as_bytes(),
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            MacAddr::V6(addr) => addr.is_nil(),
            MacAddr::V8(addr) => addr.is_nil(),
        }
    }
}

impl FromStr for MacAddr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match MacAddr6::from_str(s) {
            Ok(addr) => return Ok(MacAddr::V6(addr)),
            Err(err @ ParseError::InvalidCharacter(..)) => return Err(err),
            Err(ParseError::InvalidLength(..)) => {}
        };

        match MacAddr8::from_str(s) {
            Ok(addr) => Ok(MacAddr::V8(addr)),
            Err(err) => Err(err),
        }
    }
}

impl Serialize for MacAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for MacAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MacAddr::V6(v6) => fmt::Display::fmt(v6, f),
            MacAddr::V8(v8) => fmt::Display::fmt(v8, f),
        }
    }
}
