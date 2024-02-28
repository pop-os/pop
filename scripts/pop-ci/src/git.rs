use std::{
    fs, io,
    path::{Path, PathBuf},
    process, str,
};

use crate::util::{check_output, check_status};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GitBranch(String);

impl GitBranch {
    pub fn new(id: &str) -> Self {
        Self(id.to_owned())
    }

    pub fn id(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GitCommit(String);

impl GitCommit {
    pub fn new(id: &str) -> Self {
        Self(id.to_owned())
    }

    pub fn id(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GitRemote(String);

impl GitRemote {
    pub fn new(id: &str) -> Self {
        Self(id.to_owned())
    }

    pub fn origin() -> Self {
        Self::new("origin")
    }

    pub fn id(&self) -> &str {
        &self.0
    }
}

pub struct GitRepo(pub PathBuf);

impl GitRepo {
    pub fn new<P: AsRef<Path>>(dir: P) -> io::Result<Self> {
        fs::canonicalize(dir.as_ref()).map(Self)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn command(&self) -> process::Command {
        let mut command = process::Command::new("git");
        command.arg("-C").arg(&self.path());
        command
    }

    pub async fn async_fetch(&mut self, remote: &GitRemote) -> io::Result<()> {
        async_std::process::Command::new("git")
            .arg("-C")
            .arg(&self.path())
            .arg("fetch")
            .arg("--prune")
            .arg("--quiet")
            .arg("--")
            .arg(&remote.id())
            .status()
            .await
            .and_then(check_status)
    }

    pub fn heads(&self, remote: &GitRemote) -> io::Result<Vec<(GitBranch, GitCommit)>> {
        //TODO: allow slashes in remote
        if remote.id().contains("/") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "git remotes with slashes are not supported",
            ));
        }

        let prefix = format!("refs/remotes/{}/", remote.id());

        let output = self
            .command()
            .arg("for-each-ref")
            .arg("--format=%(objectname:short)\t%(refname)")
            .arg("--")
            .arg(&prefix)
            .stdout(process::Stdio::piped())
            .spawn()?
            .wait_with_output()
            .and_then(check_output)?;

        let stdout = str::from_utf8(&output.stdout)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        let mut heads = Vec::new();
        for line in stdout.lines() {
            let mut parts = line.split('\t');

            let commit_id = parts.next().ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "git for-each-ref missing commit",
            ))?;
            let commit = GitCommit::new(commit_id);

            let branch_id = parts
                .next()
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "git for-each-ref missing ref",
                ))?
                .strip_prefix(&prefix)
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "git for-each-ref ref did not start with expected prefix",
                ))?;
            if branch_id == "HEAD" {
                // Skip HEAD refs, they do not represent remote branches
                continue;
            }
            if branch_id.contains("/") {
                //TODO: allow slashes in branch names
                continue;
            }
            let branch = GitBranch::new(branch_id);

            //TODO: ensure that branch is not already in heads?
            heads.push((branch, commit));
        }

        Ok(heads)
    }

    pub fn archive<P: AsRef<Path>>(&self, commit: &GitCommit, archive: P) -> io::Result<()> {
        self.command()
            .arg("archive")
            .arg("-o")
            .arg(archive.as_ref())
            .arg("--")
            .arg(commit.id())
            .status()
            .and_then(check_status)
    }

    pub fn file_exists(&self, commit: &GitCommit, path: &str) -> io::Result<bool> {
        let status = self
            .command()
            .arg("cat-file")
            .arg("-e")
            .arg(format!("{}:{}", commit.id(), path))
            .status()?;
        Ok(status.success())
    }
}
