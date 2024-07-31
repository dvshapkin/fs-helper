use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

/// ReadDir iterator reads the directory recursively.
/// First returns all files of current directory and then visit all subdirectories.
/// Implemented with threads now (yield operator not implemented yet)!
pub struct ReadDir {
    root: PathBuf,
    rx: Option<mpsc::Receiver<PathBuf>>,
}

impl ReadDir {
    /// Attempts to create a new iterator.
    ///
    /// # Arguments:
    ///
    /// * `dir` - root directory.
    pub fn try_new<P: AsRef<Path>>(dir: P) -> io::Result<ReadDir> {
        Ok(ReadDir {
            root: fs::canonicalize(dir)?,
            rx: None,
        })
    }

    /// Returns a root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn run(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        let root = PathBuf::from(self.root());
        thread::spawn(move || Self::visit(root, &tx).unwrap());
    }

    fn visit(dir: PathBuf, tx: &mpsc::Sender<PathBuf>) -> io::Result<()> {
        let mut sub_dirs: Vec<PathBuf> = Vec::new();
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                sub_dirs.push(path)
            } else {
                let err = format!("{}", path.display());
                if let Err(e) = tx.send(path) {
                    println!("E_R_R_O_R_E: {}: {}", e, err);
                }
            }
        }
        for sub_dir in sub_dirs {
            Self::visit(sub_dir, tx)?;
        }
        Ok(())
    }
}

impl Iterator for ReadDir {
    type Item = PathBuf;

    /// Advances the iterator and returns the next value.
    fn next(&mut self) -> Option<Self::Item> {
        if self.rx.is_none() {
            self.run();
        }
        if let Some(receiver) = &self.rx {
            if let Ok(path) = receiver.recv() {
                return Some(path);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::ReadDir;
    use std::env;

    #[test]
    fn read_dir_new() {
        let rd = ReadDir::try_new(".").unwrap();
        assert_eq!(rd.root(), env::current_dir().unwrap());
    }

    #[test]
    fn read_dir_next() {
        let dir = "/tmp/fs-helper-test";
        utils::create_test_dir(dir);

        let rd = ReadDir::try_new(dir).unwrap();
        for path in rd {
            println!("{}", path.display());
        }

        utils::clean(dir);
    }

    mod utils {
        use std::fmt::Debug;
        use std::fs;
        use std::path::{Path, PathBuf};

        pub fn create_test_dir<P: AsRef<Path> + Debug>(dir: P) {
            // first level
            if !dir.as_ref().exists() {
                fs::create_dir(&dir).unwrap();
            }
            for fname in ["file01.txt", "file02.txt", "file03.txt"] {
                fs::File::create(format!("{}/{}", dir.as_ref().display(), fname)).unwrap();
            }
            // second level
            let sub_dir = PathBuf::from(format!("{}/{}", dir.as_ref().display(), "subdir1"));
            if !sub_dir.exists() {
                fs::create_dir(&sub_dir).unwrap();
            }
            for fname in ["file11.txt", "file12.txt", "file13.txt"] {
                fs::File::create(format!("{}/{}", sub_dir.display(), fname)).unwrap();
            }
            // third level
            let sub_dir = PathBuf::from(format!("{}/{}", sub_dir.display(), "subdir2"));
            if !sub_dir.exists() {
                fs::create_dir(&sub_dir).unwrap();
            }
            for fname in [
                "file21.txt",
                "file22.txt",
                "file23.txt",
                "file24.txt",
                "file25.txt",
            ] {
                fs::File::create(format!("{}/{}", sub_dir.display(), fname)).unwrap();
            }
        }

        pub fn clean<P: AsRef<Path>>(dir: P) {
            if dir.as_ref().exists() {
                fs::remove_dir_all(&dir).unwrap();
            }
        }
    }
}
