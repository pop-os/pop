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

fn main() {
    let debemail = env::var("DEBEMAIL").expect("DEBEMAIL not set");
    let debfullname = env::var("DEBFULLNAME").expect("DEBFULLNAME not set");

    let all_suites = [
        Suite::new("impish").unwrap(),
    ];

    let all_archs = [
        Arch::new("amd64"),
        Arch::new("i386"),
    ];

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

        assert_eq!(repos.insert(file_name, path), None);
    }

    let remote = GitRemote::origin();
    {
        println!("Fetching {} repos in parallel", repos.len());
        async_std::task::block_on(
            async_fetch_repos(&repos, &remote)
        );
    }

    let cache = Cache::new("_build/ci", |name| {
        name == "git" || name == "apt"
    }).expect("failed to open build cache");

    let git_cache = cache.child("git", |name| {
        repos.contains_key(name)
    }).expect("failed to open git cache");

    let mut pocket_packages = BTreeMap::<Pocket, BTreeMap<Suite, BTreeMap<String, (GitCommit, Package)>>>::new();
    for (repo_name, repo_path) in repos.iter() {
        println!("{}", repo_name);

        let repo = GitRepo::new(repo_path).expect("failed to open git repo");
        let heads = repo.heads(&remote).expect("failed to determine git repo heads");

        let mut pockets = BTreeMap::<(Pocket, Suite), GitCommit>::new();
        for (branch, commit) in heads.iter() {
            println!("  {}: {}", branch.id(), commit.id());

            let mut parts = branch.id().splitn(2, '_');
            let pocket = Pocket::new(parts.next().unwrap());
            let pattern_opt = parts.next();

            for suite in all_suites.iter() {
                let key = (pocket.clone(), suite.clone());
                let insert = if let Some(pattern) = pattern_opt {
                    // Insert pattern entry if pattern matches
                    pattern == suite.id()
                } else {
                    // Do not insert wildcard entry if others are found
                    pockets.contains_key(&key)
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

        //TODO: this may run multiple times if multiple pockets point to the same commit
        for (commit, (suites, pockets)) in builds.iter() {
            println!("  {}", commit.id());

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
                println!("    {} ({})", suite.id(), suite.version());

                let mut suite_cache = commit_cache.child(suite.id(), |name| {
                    name == "source" || all_archs.iter().any(|arch| arch.id() == name)
                }).expect("failed to open suite cache");

                let (source, source_rebuilt) = suite_cache.build("source", archive_rebuilt, |path| {
                    println!("################ SOURCE BUILD ################");
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
                        "{}~{}~{}~{}",
                        changelog_version,
                        commit_timestamp,
                        suite.version(),
                        &commit.id()[..7]
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

                    fs::write(archive.join("debian").join("changelog"), changelog)?;

                    process::Command::new("debuild")
                        .arg("--preserve-envvar").arg("PATH")
                        .arg("--set-envvar").arg(format!("SOURCE_DATE_EPOCH={}", commit_timestamp))
                        .arg("--no-tgz-check")
                        .arg("-d")
                        .arg("-S")
                        .current_dir(&archive)
                        .status()
                        .and_then(check_status)?;

                    Ok(())
                }).expect("failed to build suite source");

                let mut package = Package {
                    dscs: BTreeMap::new(),
                    tars: BTreeMap::new(),
                    debs: BTreeMap::new(),
                    rebuilt: source_rebuilt,
                };

                for entry_res in fs::read_dir(&source).expect("failed to read suite source directory") {
                    let entry = entry_res.expect("failed to read suite source entry");
                    let file_name = entry.file_name()
                        .into_string()
                        .expect("suite source filename is not utf-8");
                    if file_name.ends_with(".dsc") {
                        assert_eq!(package.dscs.insert(file_name, entry.path()), None);
                    } else if file_name.ends_with(".tar.xz") {
                        assert_eq!(package.tars.insert(file_name, entry.path()), None);
                    }
                }
                if package.dscs.len() != 1 {
                    eprintln!("    found {} .dsc files instead of 1", package.dscs.len());
                    continue;
                }

                //TODO: find archs that are actually used
                for arch in all_archs.iter() {
                    let (_, dsc_path) = package.dscs.iter().next().unwrap();
                    //TODO: force rebuild based on source changes
                    let (binary, binary_rebuilt) = suite_cache.build(arch.id(), source_rebuilt, |path| {
                        println!("################ BINARY BUILD ################");
                        fs::create_dir(&path)?;

                        let ppa_key = fs::canonicalize("scripts/.ppa.asc")?;
                        let ppa_release = "system76/pop";
                        let ppa_proposed = "system76/proposed";

                        process::Command::new("sbuild")
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
                    }).expect("failed to build suite binary");

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
        println!("pocket: {}", pocket.id());

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
            println!("  suite: {} ({})", suite.id(), suite.version());

            let suite_pool_cache = pool_cache.child(suite.id(), |name| {
                repo_packages.contains_key(name)
            }).expect("failed to open suite pool cache");

            for (repo_name, (commit, package)) in repo_packages.iter() {
                println!("    repo: {}", repo_name);

                let mut repo_pool_cache = suite_pool_cache.child(repo_name, |name| {
                    name == commit.id()
                }).expect("failed to open repo cache");

                let (_, repo_pool_rebuilt) = repo_pool_cache.build(commit.id(), package.rebuilt, |path| {
                    fs::create_dir(&path)?;

                    for (dsc_name, dsc_path) in package.dscs.iter() {
                        println!("      dsc: {}", dsc_name);
                        fs::copy(dsc_path, path.join(dsc_name))?;
                    }

                    for (tar_name, tar_path) in package.tars.iter() {
                        println!("      tar: {}", tar_name);
                        fs::copy(tar_path, path.join(tar_name))?;
                    }

                    for (deb_name, deb_path) in package.debs.iter() {
                        println!("      deb: {}", deb_name);
                        fs::copy(deb_path, path.join(deb_name))?;
                    }

                    Ok(())
                }).expect("failed to build commit cache");

                if repo_pool_rebuilt {
                    pool_rebuilt = true;
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
}
