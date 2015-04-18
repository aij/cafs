use std::path::Path;
use leveldb::database::Database;
use leveldb::database;
use leveldb::kv::KV;
use leveldb::options::{Options,WriteOptions,ReadOptions};
use sha256::Sha256;

pub use leveldb::database::error::Error;

pub struct StoragePoolLeveldb {
    db: Database<Sha256>,
    write_opts: WriteOptions,
}
impl StoragePoolLeveldb {
    pub fn open(path:&Path, create:bool) -> Result<StoragePoolLeveldb,database::error::Error> {
        let mut options = Options::new();
        options.create_if_missing = create;
        let db = try!(Database::open(path, options));
        Ok(StoragePoolLeveldb{db: db, write_opts: WriteOptions::new()})
    }

    pub fn get(&self, hash:Sha256) -> Result<Option<Vec<u8>>,Error> {
        self.db.get(ReadOptions::new(), hash)
    }
    pub fn put(&self, bytes:&[u8]) -> Result<Sha256,Error> {
        let sha = Sha256::of_bytes(bytes);
        try!(self.db.put(self.write_opts, sha.clone(), bytes));
        Ok(sha)
    }
}
