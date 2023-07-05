use crate::Config;
use crate::Limits;
use crate::Recordlist;

/// Application filesystem.
pub struct Fs {
    dir: std::path::PathBuf,
}

/// Marker for types that are serialized to or deserialized from the filesystem.
pub trait Io: Default + ToString + std::str::FromStr {
    const FILENAME: &'static str;
}
impl Io for Config {
    const FILENAME: &'static str = ".ledger.json";
}
impl Io for Recordlist {
    const FILENAME: &'static str = "ledger.jsonl";
}
impl Io for Limits {
    const FILENAME: &'static str = "limits.json";
}

impl Fs {
    pub fn new<P>(dir: P) -> Self
    where
        P: Into<std::path::PathBuf>,
    {
        Self { dir: dir.into() }
    }

    /// Returns the working directory.
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    pub fn is_repo(&self) -> bool {
        self.path::<Config>().exists()
    }

    /// Returns the path which `T` will be serialized to and deserialized from.
    pub fn path<T>(&self) -> std::path::PathBuf
    where
        T: Io,
    {
        self.dir.join(T::FILENAME)
    }

    /// Deserializes `T` from disk. Returns `T::default()` if `T`'s file does
    /// not exist.
    pub fn read<T>(&self) -> Result<T, ReadError>
    where
        T: Io,
        <T as std::str::FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        match std::fs::read_to_string(self.path::<T>()) {
            Ok(s) => s
                .parse()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                .map_err(ReadError::Serde),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(T::default()),
                _ => Err(ReadError::Io(e)),
            },
        }
    }

    pub fn write<T>(&self, obj: &T) -> std::io::Result<()>
    where
        T: Io,
    {
        std::fs::write(self.path::<T>(), obj.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] Box<dyn std::error::Error + Send + Sync>),
    // This box can be removed once specialization stabilizes.
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    /// Returns a filesystem object anchored at a temporary directory. The `Fs`
    /// must not outlive the returned `TempDir`.
    fn tempfs() -> (Fs, tempfile::TempDir) {
        let td = tempfile::TempDir::new().unwrap();
        let fs = Fs::new(td.path());
        (fs, td)
    }

    #[test]
    fn path() {
        let (fs, _td) = tempfs();

        let a = fs.path::<Config>();
        let b = fs.path::<Recordlist>();
        let c = fs.path::<Limits>();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn config() {
        let (fs, _td) = tempfs();

        // Read nonexistent config.
        assert_eq!(fs.is_repo(), false);
        assert_eq!(fs.read::<Config>().unwrap(), Config::default());

        // Read config.
        let s = r#"{"unsignedIsNegative": true}"#;
        let config = s.parse::<Config>().unwrap();
        std::fs::write(fs.path::<Config>(), s).unwrap();
        assert_eq!(fs.is_repo(), true);
        assert_eq!(fs.read::<Config>().unwrap(), config);

        // Write config.
        fs.write(&config).unwrap();
        assert_eq!(
            std::fs::read_to_string(fs.path::<Config>()).unwrap(),
            indoc!(
                r#"
                {
                  "firstIndexInDate": 0,
                  "limAccountType": null,
                  "unsignedIsNegative": true,
                  "useColoredOutput": false,
                  "useUnicodeSymbols": false
                }
                "#
            )
        );
    }
}
