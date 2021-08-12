use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

pub struct Cache(PathBuf);

impl Cache {
    pub fn new<P: AsRef<Path>, F: Fn(&str) -> bool>(path: P, retain: F) -> io::Result<Self> {
        let path = path.as_ref();
        if ! path.is_dir() {
            fs::create_dir_all(&path)?;
        }
        let path = fs::canonicalize(path)?;
        for entry_res in fs::read_dir(&path)? {
            let entry = entry_res?;
            let file_name = entry.file_name().into_string().map_err(|err| io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse file_name: {:?}", err)
            ))?;
            if ! retain(&file_name) {
                let entry_path = entry.path();
                eprintln!("Cache::new: removing {}", entry_path.display());
                //TODO: rename before removing
                if entry_path.is_dir() {
                    fs::remove_dir_all(&entry_path)?;
                } else {
                    fs::remove_file(&entry_path)?;
                }
            }
        }
        Ok(Self(path))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn child<F: Fn(&str) -> bool>(&self, name: &str, retain: F) -> io::Result<Self> {
        Self::new(self.path().join(name), retain)
    }

    pub fn build<F: Fn(&Path) -> io::Result<()>>(&mut self, name: &str, force: bool, f: F) -> io::Result<(PathBuf, bool)> {
        let partial_prefix = "partial.";
        if name.starts_with(partial_prefix) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "name starts with '{}': {:?}",
                    partial_prefix,
                    name
                )
            ));
        }

        let path = self.path().join(name);
        if path.exists() {
            if force {
                eprintln!("Cache::build: forcing rebuild of {}", path.display());
                //TODO: rename before removing
                if path.is_dir() {
                    fs::remove_dir_all(&path)?;
                } else {
                    fs::remove_file(&path)?;
                }
            } else {
                return Ok((path, false));
            }
        }

        let partial_path = self.path().join(format!("{}{}", partial_prefix, name));
        if partial_path.exists() {
            //TODO: should we automatically remove?
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "partial data already exists: {:?}",
                    partial_path
                )
            ));
        }

        f(&partial_path)?;

        fs::rename(partial_path, &path)?;

        Ok((path, true))
    }
}
