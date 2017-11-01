use std::fmt;
use std::cmp::{PartialEq, PartialOrd, Ord, Eq, Ordering};

use semver::VersionReq;
use Version;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub req: VersionReq,
}

#[derive(Debug, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct CrateId {
    pub name: String,
    pub version: Version
}

#[derive(Deserialize, Debug)]
pub struct Crate {
    pub name: String,
    #[serde(rename="vers")]
    pub version: Version,
    #[serde(rename="deps")]
    pub dependencies: Vec<Dependency>,
}

impl Crate {
    pub fn id(&self) -> CrateId {
        CrateId {
            name: self.name.clone(),
            version: self.version.clone(),
        }
    }
}

impl fmt::Display for CrateId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}.{}.{}",
            self.name, self.version.major, self.version.minor, self.version.patch)?;
        if !self.version.pre.is_empty() {
            write!(f, "-")?;
        }
        for pre in &self.version.pre {
            write!(f, "{}", pre)?;
        }
        if !self.version.build.is_empty() {
            write!(f, "+")?;
        }
        for build in &self.version.build {
            write!(f, "{}", build)?;
        }
        Ok(())
    }
}

impl PartialOrd for Crate {
    fn partial_cmp(&self, other: &Crate) -> Option<Ordering> {
        Some(self.name.cmp(&other.name).then_with(|| self.version.cmp(&other.version)))
    }
}

impl Ord for Crate {
    fn cmp(&self, other: &Crate) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Crate {
    fn eq(&self, other: &Crate) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl Eq for Crate {}

