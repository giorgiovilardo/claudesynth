use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("invalid version format: {0}")]
    InvalidFormat(String),
}

/// Validated semver-like version (major.minor.patch).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::InvalidFormat(s.to_string()));
        }
        let major = parts[0]
            .parse()
            .map_err(|_| VersionError::InvalidFormat(s.to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| VersionError::InvalidFormat(s.to_string()))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| VersionError::InvalidFormat(s.to_string()))?;
        Ok(Version {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

impl From<Version> for String {
    fn from(v: Version) -> Self {
        v.to_string()
    }
}

impl TryFrom<String> for Version {
    type Error = VersionError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_version() {
        let v: Version = "2.1.76".parse().unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 76);
    }

    #[test]
    fn parse_zero_version() {
        let v: Version = "0.0.0".parse().unwrap();
        assert_eq!(
            v,
            Version {
                major: 0,
                minor: 0,
                patch: 0
            }
        );
    }

    #[test]
    fn reject_two_parts() {
        assert!("2.1".parse::<Version>().is_err());
    }

    #[test]
    fn reject_four_parts() {
        assert!("2.1.3.4".parse::<Version>().is_err());
    }

    #[test]
    fn reject_non_numeric() {
        assert!("a.b.c".parse::<Version>().is_err());
    }

    #[test]
    fn reject_empty() {
        assert!("".parse::<Version>().is_err());
    }

    #[test]
    fn display_roundtrip() {
        let v: Version = "2.1.76".parse().unwrap();
        assert_eq!(v.to_string(), "2.1.76");
        let v2: Version = v.to_string().parse().unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn ordering() {
        let v1: Version = "1.0.0".parse().unwrap();
        let v2: Version = "1.0.1".parse().unwrap();
        let v3: Version = "1.1.0".parse().unwrap();
        let v4: Version = "2.0.0".parse().unwrap();
        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
    }

    #[test]
    fn ordering_equal() {
        let v1: Version = "1.2.3".parse().unwrap();
        let v2: Version = "1.2.3".parse().unwrap();
        assert_eq!(v1.cmp(&v2), std::cmp::Ordering::Equal);
    }

    #[test]
    fn serde_roundtrip() {
        let v: Version = "2.1.76".parse().unwrap();
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"2.1.76\"");
        let v2: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn serde_rejects_invalid() {
        let result = serde_json::from_str::<Version>("\"not.a.version\"");
        assert!(result.is_err());
    }
}
