use std::borrow::Cow;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::fmt;

use semver::VersionReq;
use Version;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub req: VersionReq,
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct CrateId<'a> {
    pub name: Cow<'a, str>,
    pub version: Version,
}

impl<'a> CrateId<'a> {
    pub fn to_owned(self) -> CrateId<'static> {
        CrateId {
            version: self.version,
            name: Cow::from(Cow::into_owned(self.name)),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Crate {
    pub name: String,
    #[serde(rename = "vers")]
    pub version: Version,
    #[serde(rename = "deps")]
    pub dependencies: Vec<Dependency>,
}

impl Crate {
    pub fn id(&self) -> CrateId {
        CrateId {
            name: self.name.as_str().into(),
            version: self.version.clone(),
        }
    }
}

impl<'a> fmt::Display for CrateId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}.{}.{}",
            self.name, self.version.major, self.version.minor, self.version.patch
        )?;
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
        Some(
            self.name
                .cmp(&other.name)
                .then_with(|| self.version.cmp(&other.version)),
        )
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
