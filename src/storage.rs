use config;
use voldb_sqlite3::VolDbSqlite;
use error::Result;
pub use storage_pool_leveldb::StoragePoolLeveldb as StoragePool;

pub fn open(conf: &config::StoragePool) -> Result<Box<StoragePool>> {
    Ok(Box::new(try!(StoragePool::open(&conf.path, true))))
}

