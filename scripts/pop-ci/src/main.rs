use clap::{App, Arg};
use pop_ci::{
    cache::Cache,
    git::{GitCommit, GitRemote, GitRepo},
    repo::{Arch, Package, Pocket, Suite},
    util::{check_output, check_status},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fmt::Write,
    fs,
    io,
    path::{Path, PathBuf},
    process,
    str,
};

macro_rules! bold {
    ($arg:tt) => (concat!("\x1B[1m", $arg, "\x1B[0m"));
}

static DEV_REPOS: &'static [&'static str] = &[
    "accountsservice",
    "amd-ppt-bin",
    "bcmwl",
    "distinst",
    "dwarves",
    "firmware-manager",
    "fwupd",
    "gdm3",
    "gnome-desktop3",
    "gnome-settings-daemon",
    "gnome-shell-extension-system76-power",
    "hidpi-daemon",
    "libxmlb",
    "linux",
    "linux-firmware",
    "mesa",
    "nvidia-graphics-drivers",
    "system76-acpi-dkms",
    "system76-dkms",
    "system76-driver",
    "system76-firmware",
    "system76-io-dkms",
    "system76-keyboard-configurator",
    "system76-oled",
    "system76-power",
    "system76-wallpapers",
    "ubuntu-drivers-common",
    "virtualbox",
];

//TODO: limit jobs?
async fn async_fetch_repos(repos: &BTreeMap<String, PathBuf>, remote: &GitRemote) {
    use futures::stream::StreamExt;

    let mut futures = futures::stream::FuturesUnordered::new();
    for (name, repo_path) in repos.iter() {
        let mut repo = match GitRepo::new(repo_path) {
            Ok(ok) => ok,
            Err(err) => {
                eprintln!("{}: failed to open git repo: {}", name, err);
                process::exit(1);
            }
        };

        futures.push(async move {
            (
                name,
                repo.async_fetch(remote).await
            )
        });
    }

    while let Some((name, res)) = futures.next().await {
        if let Err(err) = res {
            eprintln!("{}: failed to fetch git repo: {}", name, err);
            process::exit(1);
        }
    }
}

fn github_status_inner(repo_name: &str, commit: &GitCommit, context: &str, description: &str, state: &str, target_url: &str) -> io::Result<()> {
    let github_token = fs::read_to_string("scripts/.github_token")?;

    let mut data = BTreeMap::<&str, &str>::new();
    data.insert("context", context);
    data.insert("description", description);
    data.insert("state", state);
    data.insert("target_url", target_url);

    let url = format!("https://api.github.com/repos/pop-os/{}/statuses/{}", repo_name, commit.id());

    process::Command::new("curl")
        .arg("--silent")
        .arg("--show-error")
        .arg("--header").arg(format!("Authorization: token {}", github_token))
        .arg("--data-raw").arg(json::stringify(data))
        .arg(url)
        .status()
        .and_then(check_status)
}

fn main() {
    let matches = App::new("pop-ci")
        .arg(
            Arg::with_name("dev")
                .long("dev")
                .help("Build for Ubuntu instead of Pop!_OS")
        )
        .arg(
            Arg::with_name("launchpad")
                .long("launchpad")
                .help("Upload to launchpad after build")
        )
        .arg(
            Arg::with_name("publish")
                .long("publish")
                .help("Publish to apt.pop-os.org after build")
        )
        .arg(
            Arg::with_name("retry")
                .long("retry")
                .takes_value(true)
                .help("Matching builds will be retried")
        )
        .get_matches();

    let dev = matches.is_present("dev");
    let launchpad = matches.is_present("launchpad");
    let publish = matches.is_present("publish");
    let mut retry = Vec::new();
    if let Some(retry_string) = matches.value_of("retry") {
        for retry_key in retry_string.split(' ') {
            retry.push(retry_key.to_string());
        }
    }

    let debemail = env::var("DEBEMAIL").expect("DEBEMAIL not set");
    let debfullname = env::var("DEBFULLNAME").expect("DEBFULLNAME not set");

    let all_suites = [
        Suite::new("bionic").unwrap(),
        Suite::new("focal").unwrap(),
        Suite::new("hirsute").unwrap(),
        Suite::new("impish").unwrap(),
    ];

    let all_archs = [
        Arch::new("amd64"),
        Arch::new("i386"),
    ];

    for suite in all_suites.iter() {
        for arch in all_archs.iter() {
            let chroot = Path::new("/srv").join("chroot").join(format!(
                "{}-{}-sbuild",
                suite.id(), arch.id()
            ));

            if ! chroot.is_dir() {
                process::Command::new("sudo")
                    .arg("sbuild-createchroot")
                    .arg("--include=gnupg")
                    .arg("--components=main,restricted,universe,multiverse")
                    .arg(format!("--arch={}", arch.id()))
                    .arg(suite.id())
                    .arg(&chroot)
                    .arg("http://archive.ubuntu.com/ubuntu")
                    .status()
                    .and_then(check_status)
                    .expect("failed to create sbuild chroot");
            }

            process::Command::new("sudo")
                .arg("sbuild-update")
                .arg("--update")
                .arg("--dist-upgrade")
                .arg("--clean")
                .arg("--autoclean")
                .arg("--autoremove")
                .arg(format!("--arch={}", arch.id()))
                .arg(suite.id())
                .status()
                .and_then(check_status)
                .expect("failed to update sbuild chroot");
        }
    }

    let (ppa_key, ppa_release, ppa_proposed) = if dev {
        (
            fs::canonicalize("scripts/.ppa-dev.asc").expect("failed to find PPA key"),
            "system76-dev/stable",
            "system76-dev/pre-stable",
        )
    } else {
        (
            fs::canonicalize("scripts/.ppa.asc").expect("failed to find PPA key"),
            "system76/pop",
            "system76/proposed",
        )
    };

    let mut repos = BTreeMap::new();
    for entry_res in fs::read_dir(".").expect("failed to read directory") {
        let entry = entry_res.expect("failed to read directory entry");

        let path = entry.path();
        if ! path.is_dir() {
            // Skip if not a folder
            continue;
        }

        if ! path.join("debian").is_dir() {
            // Skip if not a debian package
            //TODO: do this check on git archives instead?
            continue;
        }

        let file_name = entry.file_name()
            .into_string()
            .expect("filename is not utf-8");

        if dev && ! DEV_REPOS.contains(&file_name.as_str()) {
            // Skip if building dev repos and this is not one of them
            continue;
        }

        assert_eq!(repos.insert(file_name, path), None);
    }

    let remote = GitRemote::origin();
    {
        eprintln!(bold!("ci: fetching {} repos in parallel"), repos.len());
        async_std::task::block_on(
            async_fetch_repos(&repos, &remote)
        );
    }

    let cache_path = if dev {
        "_build/ci-dev"
    } else {
        "_build/ci"
    };
    let cache = Cache::new(cache_path, |name| {
        name == "git" || name == "apt" || name == "log"
    }).expect("failed to open build cache");

    let git_cache = cache.child("git", |name| {
        repos.contains_key(name)
    }).expect("failed to open git cache");

    let mut logs = BTreeMap::<String, (PathBuf, bool)>::new();
    let mut pocket_logs = BTreeMap::<Pocket, BTreeMap<String, (PathBuf, bool)>>::new();
    let mut pocket_packages = BTreeMap::<Pocket, BTreeMap<Suite, BTreeMap<String, (GitCommit, Package)>>>::new();
    for (repo_name, repo_path) in repos.iter() {
        eprintln!(bold!("{}"), repo_name);

        let repo = GitRepo::new(repo_path).expect("failed to open git repo");
        let heads = repo.heads(&remote).expect("failed to determine git repo heads");

        let mut pockets = BTreeMap::<(Pocket, Suite), GitCommit>::new();
        for (branch, commit) in heads.iter() {
            let mut parts = branch.id().splitn(2, '_');
            let pocket = Pocket::new(parts.next().unwrap());
            let pattern_opt = parts.next();

            for suite in all_suites.iter() {
                let key = (pocket.clone(), suite.clone());
                let insert = if let Some(pattern) = pattern_opt {
                    // Insert pattern entry if pattern matches
                    pattern == suite.id()
                } else {
                    // Only insert wildcard entry if no others are found
                    !pockets.contains_key(&key)
                };
                if insert {
                    // Allow overwrite
                    pockets.insert(key, commit.clone());
                }
            }
        }

        let mut builds = BTreeMap::<GitCommit, (BTreeSet<Suite>, BTreeSet<Pocket>)>::new();
        for ((pocket, suite), commit) in pockets.iter() {
            let entry = builds.entry(commit.clone())
                .or_insert((BTreeSet::new(), BTreeSet::new()));
            entry.0.insert(suite.clone());
            entry.1.insert(pocket.clone());
        }

        let repo_cache = git_cache.child(&repo_name, |name| {
            builds.contains_key(&GitCommit::new(name))
        }).expect("failed to open repo cache");

        for (commit, (suites, pockets)) in builds.iter() {
            let commit_name = {
                let mut join = String::new();
                for pocket in pockets.iter() {
                    if ! join.is_empty() {
                        join.push(' ');
                    }
                    join.push_str(pocket.id());
                }
                format!("{} ({})", commit.id(), join)
            };

            eprintln!(bold!("{}: {}"), repo_name, commit_name);

            let mut commit_cache = repo_cache.child(commit.id(), |name| {
                name == "archive.tar.gz" || Suite::new(name).map_or(false, |suite| suites.contains(&suite))
            }).expect("failed to open commit cache");

            let (archive_tar, archive_rebuilt) = commit_cache.build("archive.tar.gz", false, |path| {
                repo.archive(&commit, path)
            }).expect("failed to build git archive");

            let commit_timestamp = {
                //TODO: better error handling
                let output = repo.command()
                    .arg("log")
                    .arg("-1")
                    .arg("--pretty=format:%ct")
                    .arg(commit.id())
                    .stdout(process::Stdio::piped())
                    .spawn().unwrap()
                    .wait_with_output()
                    .and_then(check_output).unwrap();
                str::from_utf8(&output.stdout).unwrap().trim().to_owned()
            };

            let commit_datetime = {
                //TODO: better error handling
                let output = repo.command()
                    .arg("log")
                    .arg("-1")
                    .arg("--pretty=format:%cD")
                    .arg(commit.id())
                    .stdout(process::Stdio::piped())
                    .spawn().unwrap()
                    .wait_with_output()
                    .and_then(check_output).unwrap();
                str::from_utf8(&output.stdout).unwrap().trim().to_owned()
            };

            for suite in suites.iter() {
                let suite_name = format!("{} ({})", suite.id(), suite.version());

                eprintln!(bold!("{}: {}: {}"), repo_name, commit_name, suite_name);

                let mut suite_cache = commit_cache.child(suite.id(), |name| {
                    name == "source" || all_archs.iter().any(|arch| arch.id() == name)
                }).expect("failed to open suite cache");

                let mut source_retry = false;
                for retry_key in &[
                    repo_name.clone(),
                    format!("git:{}", commit.id()),
                    format!("dist:{}", suite.id()),
                ] {
                    if retry.contains(&retry_key) {
                        source_retry = true;
                        break;
                    }
                }

                let source_log_name = format!(
                    "{}_{}_{}_{}.log",
                    repo_name, commit.id(), suite.id(), "source"
                );
                let source_log_path = cache.path().join("log").join(&source_log_name);
                if source_log_path.is_file() && !source_retry {
                    eprintln!(bold!("{}: {}: {}: source already failed"), repo_name, commit_name, suite_name);
                    assert_eq!(logs.insert(source_log_name.clone(), (source_log_path.clone(), false)), None);
                    for pocket in pockets.iter() {
                        assert_eq!(
                            pocket_logs.entry(pocket.clone())
                                .or_insert(BTreeMap::new())
                                .insert(source_log_name.clone(), (source_log_path.clone(), false)),
                            None
                        );
                    }
                    continue;
                }

                let github_status = |step: &str, status: &str| {
                    let target_url = match env::var("BUILD_URL") {
                        Ok(some) => some,
                        Err(_) => return,
                    };

                    eprintln!(
                        bold!("{}: {}: {}: {} github status {}"),
                        repo_name, commit_name, suite_name, step, status
                    );

                    let (context, description) = if dev {
                        (
                            format!("ubuntu/staging/{}", step),
                            format!("Ubuntu Staging {}", step),
                        )
                    } else {
                        (
                            format!("pop-os/staging/{}", step),
                            format!("Pop!_OS Staging {}", step),
                        )
                    };

                    match github_status_inner(
                        repo_name,
                        commit,
                        &context,
                        &description,
                        status,
                        &target_url
                    ) {
                        Ok(()) => (),
                        Err(err) => eprintln!(
                            bold!("{}: {}: {}: {} github status {} failed: {}"),
                            repo_name, commit_name, suite_name, step, status, err
                        )
                    }
                };

                let source_res = suite_cache.build("source", archive_rebuilt, |path| {
                    eprintln!(bold!("{}: {}: {}: source building"), repo_name, commit_name, suite_name);
                    github_status("source", "pending");
                    fs::create_dir(&path)?;

                    let archive = path.join("archive");
                    fs::create_dir(&archive)?;
                    process::Command::new("tar")
                        .arg("--extract")
                        .arg("-f").arg(&archive_tar)
                        .arg("-C").arg(&archive)
                        .status()
                        .and_then(check_status)?;

                    let changelog_source = {
                        let output = process::Command::new("dpkg-parsechangelog")
                            .arg("--show-field").arg("Source")
                            .current_dir(&archive)
                            .stdout(process::Stdio::piped())
                            .spawn()?
                            .wait_with_output()
                            .and_then(check_output)?;
                        str::from_utf8(&output.stdout).map_err(|err| {
                            io::Error::new(io::ErrorKind::InvalidData, err)
                        })?.trim().to_owned()
                    };

                    let changelog_version = {
                        let output = process::Command::new("dpkg-parsechangelog")
                            .arg("--show-field").arg("Version")
                            .current_dir(&archive)
                            .stdout(process::Stdio::piped())
                            .spawn()?
                            .wait_with_output()
                            .and_then(check_output)?;
                        str::from_utf8(&output.stdout).map_err(|err| {
                            io::Error::new(io::ErrorKind::InvalidData, err)
                        })?.trim().to_owned()
                    };

                    let version = format!(
                        "{}~{}~{}~{}{}",
                        changelog_version,
                        commit_timestamp,
                        suite.version(),
                        &commit.id()[..7],
                        if dev { "~dev" } else { "" }
                    );

                    let mut changelog = String::new();
                    writeln!(changelog,
                        "{} ({}) {}; urgency=medium",
                        changelog_source, version, suite.id()
                    ).unwrap();
                    writeln!(changelog).unwrap();
                    writeln!(changelog, "  * Auto Build").unwrap();
                    writeln!(changelog).unwrap();
                    writeln!(changelog,
                        " -- {} <{}>  {}",
                        debfullname, debemail, commit_datetime
                    ).unwrap();

                    let changelog_path = if repo_name == "linux" {
                        // linux has a different changelog path
                        let changelog_path = archive.join("debian.master").join("changelog");
                        // linux needs all entries to be present
                        changelog.push('\n');
                        changelog.push_str(&fs::read_to_string(&changelog_path)?);
                        changelog_path
                    } else {
                        archive.join("debian").join("changelog")
                    };
                    fs::write(&changelog_path, changelog)?;

                    if archive.join("debian").join("patches").join("series").exists() {
                        process::Command::new("quilt")
                            .arg("push")
                            .arg("-a")
                            .current_dir(&archive)
                            .env("QUILT_PATCHES", "debian/patches")
                            .status()
                            .and_then(check_status)?;
                    }

                    process::Command::new("debuild")
                        .arg("--preserve-envvar").arg("PATH")
                        .arg("--set-envvar").arg(format!("SOURCE_DATE_EPOCH={}", commit_timestamp))
                        .arg("--no-lintian")
                        .arg("--no-tgz-check")
                        .arg("-d")
                        .arg("-S")
                        .current_dir(&archive)
                        .status()
                        .and_then(check_status)?;

                    Ok(())
                });

                let (source, source_rebuilt) = match source_res {
                    Ok(ok) => {
                        eprintln!(bold!("{}: {}: {}: source built"), repo_name, commit_name, suite_name);
                        if ok.1 {
                            github_status("source", "success");
                        }
                        ok
                    },
                    Err(err) => {
                        eprintln!(bold!("{}: {}: {}: source failed: {}"), repo_name, commit_name, suite_name, err);
                        github_status("source", "failure");

                        let partial_source_dir = suite_cache.path().join("partial.source");
                        if partial_source_dir.is_dir() {
                            for entry_res in fs::read_dir(&partial_source_dir).expect("failed to read partial source directory") {
                                let entry = entry_res.expect("failed to read partial source entry");
                                let file_name = entry.file_name()
                                    .into_string()
                                    .expect("partial source filename is not utf-8");
                                if file_name.ends_with("_source.build") {
                                    assert_eq!(logs.insert(source_log_name.clone(), (entry.path(), true)), None);
                                    for pocket in pockets.iter() {
                                        assert_eq!(
                                            pocket_logs.entry(pocket.clone())
                                                .or_insert(BTreeMap::new())
                                                .insert(source_log_name.clone(), (entry.path(), true)),
                                            None
                                        );
                                    }
                                }
                            }
                        }

                        continue;
                    }
                };

                let mut package = Package {
                    rebuilt: source_rebuilt,
                    changes: BTreeMap::new(),
                    dscs: BTreeMap::new(),
                    tars: BTreeMap::new(),
                    archs: Vec::new(),
                    debs: BTreeMap::new(),
                };

                for entry_res in fs::read_dir(&source).expect("failed to read suite source directory") {
                    let entry = entry_res.expect("failed to read suite source entry");
                    let file_name = entry.file_name()
                        .into_string()
                        .expect("suite source filename is not utf-8");
                    if file_name.ends_with(".changes") {
                        assert_eq!(package.changes.insert(file_name, entry.path()), None);
                    } else if file_name.ends_with(".dsc") {
                        assert_eq!(package.dscs.insert(file_name, entry.path()), None);
                    } else if file_name.ends_with(".tar.xz") {
                        assert_eq!(package.tars.insert(file_name, entry.path()), None);
                    }
                }

                if package.changes.len() != 1 {
                    eprintln!(bold!("{}: {}: {}: found {} .changes files instead of 1"), repo_name, commit_name, suite_name, package.changes.len());
                    continue;
                }
                let (_changes_name, _changes_path) = package.changes.iter().next().unwrap();
                //TODO: locate other files using changes file

                if package.dscs.len() != 1 {
                    eprintln!(bold!("{}: {}: {}: found {} .dsc files instead of 1"), repo_name, commit_name, suite_name, package.dscs.len());
                    continue;
                }
                let (_dsc_name, dsc_path) = package.dscs.iter().next().unwrap();

                let dsc = fs::read_to_string(&dsc_path).expect("failed to read .dsc file");
                for line in dsc.lines() {
                    if line.starts_with("Architecture: ") {
                        for arch in all_archs.iter() {
                            for part in line.split(' ') {
                                if part == arch.id()
                                || part == "any"
                                || part == "linux-any"
                                || (part == "all" && arch.build_all())
                                {
                                    package.archs.push(arch.clone());
                                    break;
                                }
                            }
                        }
                    }
                }

                for arch in package.archs.iter() {
                    let mut binary_retry = source_retry;
                    for retry_key in &[
                        format!("arch:{}", arch.id()),
                    ] {
                        if retry.contains(&retry_key) {
                            binary_retry = true;
                            break;
                        }
                    }

                    let binary_log_name = format!(
                        "{}_{}_{}_{}.log",
                        repo_name, commit.id(), suite.id(), arch.id()
                    );
                    let binary_log_path = cache.path().join("log").join(&binary_log_name);
                    if binary_log_path.is_file() && !binary_retry {
                        //TODO: rebuild capability
                        eprintln!(bold!("{}: {}: {}: {}: binary already failed"), repo_name, commit_name, suite_name, arch.id());
                        assert_eq!(logs.insert(binary_log_name.clone(), (binary_log_path.clone(), false)), None);
                        for pocket in pockets.iter() {
                            assert_eq!(
                                pocket_logs.entry(pocket.clone())
                                    .or_insert(BTreeMap::new())
                                    .insert(binary_log_name.clone(), (binary_log_path.clone(), false)),
                                None
                            );
                        }
                        continue;
                    }

                    let binary_res = suite_cache.build(arch.id(), source_rebuilt, |path| {
                        eprintln!(bold!("{}: {}: {}: {}: binary building"), repo_name, commit_name, suite_name, arch.id());
                        github_status(&format!("binary-{}", arch.id()), "pending");
                        fs::create_dir(&path)?;

                        process::Command::new("sbuild")
                            .arg("--verbose")
                            .arg(if arch.build_all() { "--arch-all" } else { "--no-arch-all" })
                            .arg(format!("--arch={}", arch.id()))
                            .arg(format!("--dist={}", suite.id()))
                            .arg(format!("--extra-repository=deb http://us.archive.ubuntu.com/ubuntu/ {}-updates main restricted universe multiverse", suite.id()))
                            .arg(format!("--extra-repository=deb-src http://us.archive.ubuntu.com/ubuntu/ {}-updates main restricted universe multiverse", suite.id()))
                            .arg(format!("--extra-repository=deb http://us.archive.ubuntu.com/ubuntu/ {}-security main restricted universe multiverse", suite.id()))
                            .arg(format!("--extra-repository=deb-src http://us.archive.ubuntu.com/ubuntu/ {}-security main restricted universe multiverse", suite.id()))
                            .arg(format!("--extra-repository=deb http://ppa.launchpad.net/{}/ubuntu {} main", ppa_release, suite.id()))
                            .arg(format!("--extra-repository=deb-src http://ppa.launchpad.net/{}/ubuntu {} main", ppa_release, suite.id()))
                            .arg(format!("--extra-repository=deb http://ppa.launchpad.net/{}/ubuntu {} main", ppa_proposed, suite.id()))
                            .arg(format!("--extra-repository=deb-src http://ppa.launchpad.net/{}/ubuntu {} main", ppa_proposed, suite.id()))
                            .arg(format!("--extra-repository-key={}", ppa_key.display()))
                            .arg("--no-apt-distupgrade")
                            .arg("--no-run-autopkgtest")
                            .arg("--no-run-lintian")
                            .arg("--no-run-piuparts")
                            .arg(&dsc_path)
                            .current_dir(&path)
                            .status()
                            .and_then(check_status)
                    });

                    let (binary, binary_rebuilt) = match binary_res {
                        Ok(ok) => {
                            eprintln!(bold!("{}: {}: {}: {}: binary built"), repo_name, commit_name, suite_name, arch.id());
                            if ok.1 {
                                github_status(&format!("binary-{}", arch.id()), "success");
                            }
                            ok
                        },
                        Err(err) => {
                            eprintln!(bold!("{}: {}: {}: {}: binary failed: {}"), repo_name, commit_name, suite_name, arch.id(), err);
                            github_status(&format!("binary-{}", arch.id()), "failure");

                            let partial_binary_dir = suite_cache.path().join(format!("partial.{}", arch.id()));
                            if partial_binary_dir.is_dir() {
                                for entry_res in fs::read_dir(&partial_binary_dir).expect("failed to read partial binary directory") {
                                    let entry = entry_res.expect("failed to read partial binary entry");
                                    let file_name = entry.file_name()
                                        .into_string()
                                        .expect("partial binary filename is not utf-8");
                                    if file_name.ends_with(&format!("_{}.build", arch.id())) {
                                        assert_eq!(logs.insert(binary_log_name.clone(), (entry.path(), true)), None);
                                        for pocket in pockets.iter() {
                                            assert_eq!(
                                                pocket_logs.entry(pocket.clone())
                                                    .or_insert(BTreeMap::new())
                                                    .insert(binary_log_name.clone(), (entry.path(), true)),
                                                None
                                            );
                                        }
                                    }
                                }
                            }

                            continue;
                        }
                    };

                    if binary_rebuilt {
                        package.rebuilt = true;
                    }

                    for entry_res in fs::read_dir(&binary).expect("failed to read suite binary directory") {
                        let entry = entry_res.expect("failed to read suite binary entry");
                        let file_name = entry.file_name()
                            .into_string()
                            .expect("suite binary filename is not utf-8");
                        if file_name.ends_with(".deb") {
                            assert_eq!(package.debs.insert(file_name, entry.path()), None);
                        }
                    }
                }

                for pocket in pockets.iter() {
                    assert_eq!(
                        pocket_packages.entry(pocket.clone())
                            .or_insert(BTreeMap::new())
                            .entry(suite.clone())
                            .or_insert(BTreeMap::new())
                            .insert(repo_name.clone(), (commit.clone(), package.clone())),
                        None
                    );
                }
            }
        }
    }

    let apt_cache = cache.child("apt", |name| {
        pocket_packages.contains_key(&Pocket::new(name))
    }).expect("failed to open apt cache");

    for (pocket, suite_packages) in pocket_packages.iter() {
        eprintln!(bold!("pocket: {}"), pocket.id());

        let pocket_cache = apt_cache.child(pocket.id(), |name| {
            name == "dists" || name == "pool"
        }).expect("failed to open pocket cache");

        let pool_cache = pocket_cache.child("pool", |name| {
            Suite::new(name).map_or(false, |suite| suite_packages.contains_key(&suite))
        }).expect("failed to open pool cache");

        let mut dists_cache = pocket_cache.child("dists", |name| {
            Suite::new(name).map_or(false, |suite| suite_packages.contains_key(&suite))
        }).expect("failed to open dists cache");

        let mut pool_rebuilt = false;
        for (suite, repo_packages) in suite_packages.iter() {
            eprintln!(bold!("  suite: {} ({})"), suite.id(), suite.version());

            let suite_pool_cache = pool_cache.child(suite.id(), |name| {
                repo_packages.contains_key(name)
            }).expect("failed to open suite pool cache");

            for (repo_name, (commit, package)) in repo_packages.iter() {
                eprintln!(bold!("    package: {}: {}"), repo_name, commit.id());

                let mut repo_pool_cache = suite_pool_cache.child(repo_name, |name| {
                    name == commit.id()
                }).expect("failed to open repo cache");

                let (_, repo_pool_rebuilt) = repo_pool_cache.build(commit.id(), package.rebuilt, |path| {
                    fs::create_dir(&path)?;

                    for (dsc_name, dsc_path) in package.dscs.iter() {
                        eprintln!("      dsc: {}", dsc_name);
                        fs::copy(dsc_path, path.join(dsc_name))?;
                    }

                    for (tar_name, tar_path) in package.tars.iter() {
                        eprintln!("      tar: {}", tar_name);
                        fs::copy(tar_path, path.join(tar_name))?;
                    }

                    for (deb_name, deb_path) in package.debs.iter() {
                        eprintln!("      deb: {}", deb_name);
                        fs::copy(deb_path, path.join(deb_name))?;
                    }

                    Ok(())
                }).expect("failed to build commit cache");

                if repo_pool_rebuilt {
                    pool_rebuilt = true;
                }

                if pocket.id() == "master" && launchpad {
                    for (changes_name, changes_path) in package.changes.iter() {
                        if ! changes_name.ends_with("_source.changes") {
                            // We can only upload source changes
                            continue;
                        }

                        let ppa_upload_name = changes_name.replace("_source.changes", "_source.ppa.upload");
                        let ppa_upload_path = changes_path.parent().unwrap().join(ppa_upload_name);
                        //TODO: allow reupload
                        if ppa_upload_path.exists() {
                            // Skip if already uploaded
                            continue;
                        }

                        eprintln!(bold!("      launchpad upload to {}"), ppa_proposed);
                        let dput_res = process::Command::new("dput")
                            .arg(&format!("ppa:{}", ppa_proposed))
                            .arg(&changes_path)
                            .status()
                            .and_then(check_status);
                        match dput_res {
                            Ok(()) => {
                                eprintln!(bold!("      launchpad upload complete"));
                            },
                            Err(err) => {
                                eprintln!(bold!("      launchpad upload failed: {}"), err);
                            },
                        }
                    }
                }
            }

            dists_cache.build(suite.id(), pool_rebuilt, |path| {
                fs::create_dir(&path)?;

                let pool_relative = Path::new("pool").join(suite.id());
                let main_dir = path.join("main");
                fs::create_dir(&main_dir)?;

                {
                    let source_dir = main_dir.join("source");
                    fs::create_dir(&source_dir)?;

                    let source_output = process::Command::new("apt-ftparchive")
                            .arg("-qq")
                            .arg("sources")
                            .arg(&pool_relative)
                            .current_dir(&pocket_cache.path())
                            .stdout(process::Stdio::piped())
                            .spawn()?
                            .wait_with_output()
                            .and_then(check_output)?;

                    let source_file = source_dir.join("Sources");
                    fs::write(&source_file, &source_output.stdout)?;
                    process::Command::new("gzip")
                        .arg("--keep")
                        .arg(&source_file)
                        .status()
                        .and_then(check_status)?;

                    let mut release = String::new();
                    writeln!(release, "Archive: {}", suite.id()).unwrap();
                    writeln!(release, "Version: {}", suite.version()).unwrap();
                    writeln!(release, "Component: main").unwrap();
                    writeln!(release, "Origin: pop-os-staging-{}", pocket.id()).unwrap();
                    writeln!(release, "Label: Pop!_OS Staging {}", pocket.id()).unwrap();
                    writeln!(release, "Architecture: source").unwrap();

                    let release_file = source_dir.join("Release");
                    fs::write(&release_file, release)?;
                }

                let mut archs_string = String::new();
                for arch in all_archs.iter() {
                    let binary_dir = main_dir.join(format!("binary-{}", arch.id()));
                    fs::create_dir(&binary_dir)?;

                    let packages_output = process::Command::new("apt-ftparchive")
                            .arg("-qq")
                            .arg("--arch").arg(arch.id())
                            .arg("packages")
                            .arg(&pool_relative)
                            .current_dir(&pocket_cache.path())
                            .stdout(process::Stdio::piped())
                            .spawn()?
                            .wait_with_output()
                            .and_then(check_output)?;

                    let packages_file = binary_dir.join("Packages");
                    fs::write(&packages_file, &packages_output.stdout)?;
                    process::Command::new("gzip")
                        .arg("--keep")
                        .arg(&packages_file)
                        .status()
                        .and_then(check_status)?;

                    let mut release = String::new();
                    writeln!(release, "Archive: {}", suite.id()).unwrap();
                    writeln!(release, "Version: {}", suite.version()).unwrap();
                    writeln!(release, "Component: main").unwrap();
                    writeln!(release, "Origin: pop-os-staging-{}", pocket.id()).unwrap();
                    writeln!(release, "Label: Pop!_OS Staging {}", pocket.id()).unwrap();
                    writeln!(release, "Architecture: {}", arch.id()).unwrap();

                    let release_file = binary_dir.join("Release");
                    fs::write(&release_file, release)?;

                    if ! archs_string.is_empty() {
                        archs_string.push(' ');
                    }
                    archs_string.push_str(arch.id());
                }

                let release_output = process::Command::new("apt-ftparchive")
                    .arg("-o").arg(format!("APT::FTPArchive::Release::Origin=pop-os-staging-{}", pocket.id()))
                    .arg("-o").arg(format!("APT::FTPArchive::Release::Label=Pop!_OS Staging {}", pocket.id()))
                    .arg("-o").arg(format!("APT::FTPArchive::Release::Suite={}", suite.id()))
                    .arg("-o").arg(format!("APT::FTPArchive::Release::Version={}", suite.version()))
                    .arg("-o").arg(format!("APT::FTPArchive::Release::Codename={}", suite.id()))
                    .arg("-o").arg(format!("APT::FTPArchive::Release::Architectures={}", archs_string))
                    .arg("-o").arg("APT::FTPArchive::Release::Components=main")
                    .arg("-o").arg(format!("APT::FTPArchive::Release::Description=Pop!_OS Staging {} {} {}", suite.id(), suite.version(), pocket.id()))
                    .arg("release").arg(".")
                    .current_dir(&path)
                    .stdout(process::Stdio::piped())
                    .spawn()?
                    .wait_with_output()
                    .and_then(check_output)?;
                let release_file = path.join("Release");
                fs::write(&release_file, &release_output.stdout)?;

                process::Command::new("gpg")
                    .arg("--clearsign")
                    .arg("--local-user").arg(&debemail)
                    .arg("--batch").arg("--yes")
                    .arg("--digest-algo").arg("sha512")
                    .arg("-o").arg(path.join("InRelease"))
                    .arg(&release_file)
                    .status()
                    .and_then(check_status)?;

                process::Command::new("gpg")
                    .arg("-abs")
                    .arg("--local-user").arg(&debemail)
                    .arg("--batch").arg("--yes")
                    .arg("--digest-algo").arg("sha512")
                    .arg("-o").arg(path.join("Release.gpg"))
                    .arg(&release_file)
                    .status()
                    .and_then(check_status)?;

                Ok(())
            }).expect("failed to build suite dists cache");
        }
    }

    if publish {
        let mut rsync_args = vec![
            "--recursive",
            "--times",
            "--links",
            "--safe-links",
            "--hard-links",
            "--stats",
        ];

        if dev {
            rsync_args.push("--rsh=ssh");
            rsync_args.push("./_build/ci-dev/apt/");
            rsync_args.push("ubuntu@apt.pop-os.org:/var/www/html/staging-ubuntu/");
        } else {
            rsync_args.push("--rsh=ssh");
            rsync_args.push("./_build/ci/apt/");
            rsync_args.push("ubuntu@apt.pop-os.org:/var/www/html/staging/");
        }

        // Publish new package data (without changing release data)
        process::Command::new("rsync")
            .arg("--exclude").arg("Packages*")
            .arg("--exclude").arg("Sources*")
            .arg("--exclude").arg("Release*")
            .arg("--exclude").arg("InRelease")
            .args(&rsync_args)
            .status()
            .and_then(check_status)
            .expect("failed to publish new package data");

        // Publish new release data and delete old package data
        process::Command::new("rsync")
            .arg("--delete")
            .arg("--delete-after")
            .args(&rsync_args)
            .status()
            .and_then(check_status)
            .expect("failed to publish new release data");
    }

    let mut log_cache = cache.child("log", |name| {
        logs.contains_key(name) || pocket_logs.contains_key(&Pocket::new(name))
    }).expect("failed to open log cache");

    for (log_name, (log_path, log_rebuilt)) in logs.iter() {
        log_cache.build(log_name, *log_rebuilt, |path| {
            fs::copy(log_path, path)?;
            Ok(())
        }).expect("failed to build log cache");
    }

    for (pocket, logs) in pocket_logs.iter() {
        let mut pocket_log_cache = log_cache.child(pocket.id(), |name| {
            logs.contains_key(name)
        }).expect("failed to open pocket log cache");

        for (log_name, (log_path, log_rebuilt)) in logs.iter() {
            pocket_log_cache.build(log_name, *log_rebuilt, |path| {
                fs::copy(log_path, path)?;
                Ok(())
            }).expect("failed to build pocket log cache");
        }
    }
}
