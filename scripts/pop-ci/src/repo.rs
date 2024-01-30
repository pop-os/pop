use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::config::{DEV_REPOS, POP_FOCAL_REPOS};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Arch(&'static str);

impl Arch {
    pub fn id(&self) -> &str {
        &self.0
    }

    pub fn build_all(&self) -> bool {
        self.id() == "amd64"
    }

    pub fn build_linux_any(&self) -> bool {
        self.id() == "amd64" || self.id() == "arm64"
    }

    pub fn ubuntu_mirror(&self, release: &str) -> &'static str {
        if self.id() == "amd64" || self.id() == "i386" {
            if release == "focal" {
                "http://us.archive.ubuntu.com/ubuntu"
            } else {
                "http://apt.pop-os.org/ubuntu"
            }
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

#[derive(Clone)]
pub struct RepoInfo {
    pub key: PathBuf,
    pub release: &'static str,
    pub staging: &'static str,
    pub dput: Option<&'static str>,
    pub archs: &'static [Arch],
}

impl RepoInfo {
    pub fn new(suite: &Suite, dev: bool) -> Self {
        const ARCHS: &'static [Arch] = &[Arch("amd64"), Arch("i386"), Arch("arm64")];

        const DEV_ARCHS: &'static [Arch] = &[Arch("amd64"), Arch("i386")];

        if dev {
            // Launchpad for all Ubuntu releases
            return Self {
                key: fs::canonicalize("scripts/.ppa-dev.asc").expect("failed to find dev PPA key"),
                release: "http://ppa.launchpad.net/system76-dev/stable/ubuntu",
                staging: "http://ppa.launchpad.net/system76-dev/pre-stable/ubuntu",
                dput: Some("ppa:system76-dev/pre-stable"),
                archs: DEV_ARCHS,
            };
        }

        match suite.id() {
            // Launchpad used prior to Pop 21.10
            "bionic" | "focal" => Self {
                key: fs::canonicalize("scripts/.ppa.asc").expect("failed to find PPA key"),
                release: "http://ppa.launchpad.net/system76/pop/ubuntu",
                staging: "http://ppa.launchpad.net/system76/proposed/ubuntu",
                dput: Some("ppa:system76/proposed"),
                archs: DEV_ARCHS,
            },
            // Disable arm64 for noble temporarily
            "noble" => Self {
                key: fs::canonicalize("scripts/.iso.asc").expect("failed to find ISO key"),
                release: "http://apt.pop-os.org/release",
                staging: "http://apt.pop-os.org/staging/master",
                dput: None,
                archs: DEV_ARCHS,
            },
            // apt.pop-os.org for Pop 21.10 and later
            _ => Self {
                key: fs::canonicalize("scripts/.iso.asc").expect("failed to find ISO key"),
                release: "http://apt.pop-os.org/release",
                staging: "http://apt.pop-os.org/staging/master",
                dput: None,
                archs: ARCHS,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SuiteWildcard {
    None,
    Focal,
    All,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SuiteDistro {
    All,
    Pop,
    Ubuntu,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Suite(&'static str, &'static str, SuiteWildcard, SuiteDistro);

impl Suite {
    // This list has every supported Pop!_OS and Ubuntu release
    pub const ALL: &'static [Self] = &[
        Self("bionic", "18.04", SuiteWildcard::None, SuiteDistro::All),
        Self("focal", "20.04", SuiteWildcard::Focal, SuiteDistro::All),
        Self("jammy", "22.04", SuiteWildcard::All, SuiteDistro::All),
        Self("lunar", "23.04", SuiteWildcard::None, SuiteDistro::Ubuntu),
        Self("mantic", "23.10", SuiteWildcard::None, SuiteDistro::Ubuntu),
        Self("noble", "24.04", SuiteWildcard::All, SuiteDistro::All),
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

    pub fn wildcard(&self, repo_name: &str) -> bool {
        match &self.2 {
            SuiteWildcard::None => false,
            SuiteWildcard::Focal => {
                DEV_REPOS.contains(&repo_name) || POP_FOCAL_REPOS.contains(&repo_name)
            }
            SuiteWildcard::All => true,
        }
    }

    pub fn distro(&self) -> &SuiteDistro {
        &self.3
    }
}
