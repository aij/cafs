use config;
use error::Result;
pub use storage_pool_leveldb::StoragePoolLeveldb as StoragePool;

pub fn open(conf: &config::Settings) -> Result<Box<StoragePool>> {
    Ok(Box::new(try!(StoragePool::open(&conf.pool.path, true))))
}

