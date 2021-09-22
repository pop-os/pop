use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Arch(&'static str);

impl Arch {
    // These are the supported architectures on an aarch64 build server
    #[cfg(target_arch = "aarch64")]
    pub const ALL: &'static [Self] = &[
        Self("arm64"),
    ];

    // These are the supported architectures on a x86_64 build server
    #[cfg(target_arch = "x86_64")]
    pub const ALL: &'static [Self] = &[
        Self("amd64"),
        Self("i386"),
    ];

    pub fn id(&self) -> &str {
        &self.0
    }

    pub fn build_all(&self) -> bool {
        self.id() == "amd64"
    }

    pub fn ubuntu_mirror(&self) -> &'static str {
        if self.id() == "amd64" || self.id() == "i386" {
            "http://archive.ubuntu.com/ubuntu"
        } else {
            "http://ports.ubuntu.com/ubuntu-ports"
        }
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

pub struct RepoInfo {
    pub key: PathBuf,
    pub release: &'static str,
    pub staging: &'static str,
    pub dput: Option<&'static str>,
}

impl RepoInfo {
    pub fn new(suite: &Suite, dev: bool) -> Self {
        if dev {
            // Launchpad for all Ubuntu releases
            return Self {
                key: fs::canonicalize("scripts/.ppa-dev.asc").expect("failed to find dev PPA key"),
                release: "http://ppa.launchpad.net/system76-dev/stable/ubuntu",
                staging: "http://ppa.launchpad.net/system76-dev/pre-stable/ubuntu",
                dput: Some("ppa:system76-dev/pre-stable"),
            }
        }

        match suite.id() {
            // Launchpad used prior to Pop 21.10
            "bionic" | "focal" | "hirsute" => Self {
                key: fs::canonicalize("scripts/.ppa.asc").expect("failed to find PPA key"),
                release: "http://ppa.launchpad.net/system76/pop/ubuntu",
                staging: "http://ppa.launchpad.net/system76/proposed/ubuntu",
                dput: Some("ppa:system76/proposed"),
            },
            // apt.pop-os.org for Pop 21.10 and later
            _ => Self {
                key: fs::canonicalize("scripts/.iso.asc").expect("failed to find ISO key"),
                release: "http://apt.pop-os.org/release",
                staging: "http://apt.pop-os.org/staging/master",
                dput: None,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Suite(&'static str, &'static str);

impl Suite {
    // This list has every supported Pop!_OS and Ubuntu release
    pub const ALL: &'static [Self] = &[
        Self("bionic", "18.04"),
        Self("focal", "20.04"),
        Self("hirsute", "21.04"),
        Self("impish", "21.10"),
    ];

    pub fn new(id: &str) -> Option<Self> {
        for suite in Self::ALL.iter() {
            if suite.id() == id {
                return Some(suite.clone());
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
