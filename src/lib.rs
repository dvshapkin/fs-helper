mod fs_helper {
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::mpsc;
    use std::thread;

    /// Reads the directory recursively.
    /// First returns all files of current directory and then visit all subdirectories.
    pub struct ReadDir {
        root: PathBuf,
        rx: Option<mpsc::Receiver<PathBuf>>,
    }

    impl ReadDir {
        pub fn new<P: AsRef<Path>>(dir: P) -> io::Result<ReadDir> {
            Ok(ReadDir {
                root: fs::canonicalize(dir)?,
                rx: None,
            })
        }

        pub fn root(&self) -> &Path {
            &self.root
        }

        fn run(&mut self) {
            let (tx, rx) = mpsc::channel();
            self.rx = Some(rx);
            let root = PathBuf::from(self.root());
            thread::spawn(move || {
                Self::visit(root, &tx).unwrap()
            });
        }

        fn visit(dir: PathBuf, tx: &mpsc::Sender<PathBuf>) -> io::Result<()>
        {
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
}


#[cfg(test)]
mod tests {
    use crate::fs_helper::ReadDir;
    use std::env;

    #[test]
    fn read_dir_new() {
        let rd = ReadDir::new(".").unwrap();
        assert_eq!(rd.root(), env::current_dir().unwrap());
    }

    #[test]
    fn read_dir_next() {
        let rd = ReadDir::new(".").unwrap();
        for path in rd {
            println!("{}", path.display());
        }
    }

    mod utils {
        use std::fs;
        use std::path::Path;

        pub fn create_test_dir(dir: &Path) {
            if !dir.exists() {
                fs::create_dir(dir).unwrap();
            }
            for fname in ["file1.txt", "file2.txt", "file3.txt"] {
                fs::File::create(format!("{}/{}", dir.to_str().unwrap(), fname)).unwrap();
            }
        }

        pub fn clean(dir: &Path) {
            if dir.exists() {
                fs::remove_dir_all(dir).unwrap();
            }
        }
    }
}
