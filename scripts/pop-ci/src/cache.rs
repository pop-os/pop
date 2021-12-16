use std::{
    collections::BTreeMap,
    fs,
    io,
    path::{Path, PathBuf},
    thread,
};

pub struct Cache {
    path: PathBuf,
    cleaned: bool
}

impl Cache {
    pub fn new<P: AsRef<Path>, F: Fn(&str) -> bool>(path: P, retain: F) -> io::Result<Self> {
        let path = path.as_ref();
        if ! path.is_dir() {
            fs::create_dir_all(&path)?;
        }
        let path = fs::canonicalize(path)?;
        let mut cleaned = false;
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
                cleaned = true;
            }
        }
        Ok(Self{ path, cleaned })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn cleaned(&self) -> bool {
        self.cleaned
    }

    pub fn child<F: Fn(&str) -> bool>(&self, name: &str, retain: F) -> io::Result<Self> {
        Self::new(self.path().join(name), retain)
    }

    fn build_inner(&mut self, name: &str, force: bool) -> io::Result<(PathBuf, Option<PathBuf>)> {
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
                return Ok((path, None));
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

        Ok((path, Some(partial_path)))
    }

    pub fn build<F: Fn(&Path) -> io::Result<()>>(&mut self, name: &str, force: bool, f: F) -> io::Result<(PathBuf, bool)> {
        let (path, partial_path_opt) = self.build_inner(name, force)?;
        match partial_path_opt {
            Some(partial_path) => {
                f(&partial_path)?;

                fs::rename(partial_path, &path)?;

                Ok((path, true))
            },
            None => Ok((path, false))
        }
    }

    pub fn build_parallel<
        F: Fn(&Path) -> io::Result<()> + Send + 'static
    >(&mut self, names: BTreeMap<String, F>, force: bool) -> BTreeMap<String, io::Result<(PathBuf, bool)>> {
        let mut threads = BTreeMap::new();
        let mut results = BTreeMap::new();
        for (name, f) in names {
            match self.build_inner(&name, force) {
                Ok((path, partial_path_opt)) => match partial_path_opt {
                    Some(partial_path) => {
                        threads.insert(name, thread::spawn(move || {
                            f(&partial_path)?;
                            fs::rename(partial_path, &path)?;
                            Ok(path)
                        }));
                    },
                    None => {
                        results.insert(name, Ok((path, false)));
                    }
                },
                Err(err) => {
                    results.insert(name, Err(err));
                }
            }
        }

        for (name, thread) in threads {
            match thread.join().unwrap() {
                Ok(path) => {
                    results.insert(name, Ok((path, true)));
                },
                Err(err) => {
                    results.insert(name, Err(err));
                }
            }
        }

        results
    }
}
