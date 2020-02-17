/*!
# Background

This library lets you get a BottlerocketRelease struct that represents the data in the /etc/os-release file, or another file you point to.
The VERSION_ID is returned as a semver::Version for convenience.
*/

const DEFAULT_RELEASE_FILE: &str = "/etc/os-release";

use log::debug;
use semver::Version;
use serde::Deserialize;
use snafu::ResultExt;
use std::fs;
use std::path::Path;

/// BottlerocketRelease represents the data found in the release file.
#[derive(Debug, Deserialize, Clone)]
pub struct BottlerocketRelease {
    pub pretty_name: String,
    pub variant_id: String,
    pub version_id: Version,
    pub build_id: String,
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub enum Error {
        #[snafu(display("Unable to read release file '{}': {}", path.display(), source))]
        ReadReleaseFile { path: PathBuf, source: io::Error },

        #[snafu(display("Unable to load release data from file '{}': {}", path.display(), source))]
        LoadReleaseData { path: PathBuf, source: envy::Error },
    }
}
pub use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

impl BottlerocketRelease {
    pub fn new() -> Result<Self> {
        Self::from_file(DEFAULT_RELEASE_FILE)
    }

    fn from_file<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let release_data = fs::read_to_string(path).context(error::ReadReleaseFile { path })?;

        // Split and process each line
        let pairs: Vec<(String, String)> = release_data
            .lines()
            .filter_map(|line| {
                // Allow for comments
                if line.starts_with("#") {
                    return None;
                }

                // Split out KEY=VALUE; if there is no "=" we skip the line
                let mut parts = line.splitn(2, '=');
                let key = parts.next().expect("split returned zero parts");
                let mut value = match parts.next() {
                    Some(part) => part,
                    None => return None,
                };
                debug!("Found os-release value {}={}", key, value);

                // If the value was quoted (unnecessary in this file) then remove the quotes
                if value.starts_with('"') {
                    value = &value[1..];
                }
                if value.ends_with('"') {
                    value = &value[..value.len() - 1];
                }

                Some((key.to_owned(), value.to_owned()))
            })
            .collect();

        envy::from_iter(pairs).context(error::LoadReleaseData { path })
    }
}
