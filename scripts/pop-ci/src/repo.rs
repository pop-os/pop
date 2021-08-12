use std::{
    collections::BTreeMap,
    path::PathBuf,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Arch(String);

impl Arch {
    pub fn new(id: &str) -> Self {
        Self(id.to_owned())
    }

    pub fn id(&self) -> &str {
        &self.0
    }

    pub fn build_all(&self) -> bool {
        //TODO: test if host arch?
        self.id() == "amd64"
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Package {
    pub rebuilt: bool,
    pub changes: BTreeMap<String, PathBuf>,
    pub dscs: BTreeMap<String, PathBuf>,
    pub tars: BTreeMap<String, PathBuf>,
    pub archs: Vec<Arch>,
    pub debs: BTreeMap<String, PathBuf>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pocket(String);

impl Pocket {
    pub fn new(id: &str) -> Self {
        Self(id.to_owned())
    }

    pub fn id(&self) -> &str {
        &self.0
    }
}

// This list has every Pop!_OS release
static SUITE_VERSIONS: &'static [(&'static str, &'static str)] = &[
    ("artful", "17.10"),
    ("bionic", "18.04"),
    ("cosmic", "18.10"),
    ("disco", "19.04"),
    ("eoan", "19.10"),
    ("focal", "20.04"),
    ("groovy", "20.10"),
    ("hirsute", "21.04"),
    ("impish", "21.10"),
];

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Suite(&'static str, &'static str);

impl Suite {
    pub fn new(id: &str) -> Option<Self> {
        for (codename, version) in SUITE_VERSIONS.iter() {
            if *codename == id {
                return Some(Self(codename, version));
            }
        }
        None
    }

    pub fn id(&self) -> &str {
        self.0
    }

    pub fn version(&self) -> &str {
        self.1
    }
}
