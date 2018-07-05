use semver;
use std::ops::Deref;
use std::hash;
use std::cmp::{PartialEq, PartialOrd, Ord, Eq, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version(semver::Version);

impl Version {
    pub fn new(v: semver::Version) -> Self {
        Version(v)
    }
}

impl Deref for Version {
    type Target = semver::Version;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl hash::Hash for Version {
    fn hash<H: hash::Hasher>(&self, h: &mut H) {
        self.0.hash(h);
        self.0.build.hash(h);
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        self.0 == other.0 && self.0.build == other.0.build
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        self.0.cmp(&other.0).then_with(|| {
            self.0.build.cmp(&other.0.build)
        })
    }
}
