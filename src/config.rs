use std::env;
use std::path::{Path, PathBuf};
//use toml_config::ConfigFactory;

#[derive(RustcEncodable, RustcDecodable)]
pub struct VolDb {
    pub path: PathBuf,
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct StoragePool {
    pub path: PathBuf,
}

#[derive(RustcEncodable, RustcDecodable, Clone)]
pub enum EncType {
    None,
    AES256, // Covergent AES_256_CBC with SHA-2 256 as key.
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct Settings {
    //block_size: usize,
    pub enc_type: EncType,
    pub voldb: VolDb,
    pub pool: StoragePool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            enc_type: EncType::AES256,
            voldb: VolDb::default(),
            pool: StoragePool::default(),
        }
    }
}

// Path from environment variable
fn env_path(e: &str) -> Result<PathBuf, ::std::env::VarError> {
    let p = try!(env::var(e));
    Ok(Path::new(&p).to_path_buf())
}

// Path relative to the config dir.
pub fn rel_path(rel: &str) -> PathBuf {
    let mut h = PathBuf::new();
    if let Ok(home) = env_path("CAFS_HOME") {
        h.push(home)
    } else if let Some(home) = ::std::env::home_dir() {
        h.push(home);
        h.push(".cafs");
    } else {
        h.push("/etc/cafs")
    };
    h.push(rel);
    h
}
    
impl Default for VolDb {
    fn default() -> VolDb {
        let path = env_path("CAFS_VOLDB")
            .unwrap_or_else(|_| rel_path("storage/voldb.sqlite"));
        VolDb {
            path: path,
        }
    }
}

impl Default for StoragePool {
    fn default() -> StoragePool {
        let path = env_path("CAFS_STORAGE_POOL")
            .unwrap_or_else(|_| rel_path("storage/pool.leveldb"));
        StoragePool {
            path: path,
        }
    }
}

pub fn load() -> Settings {
    // TODO: Read from TOML file instead.
    Settings::default()
}
