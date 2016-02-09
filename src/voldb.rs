use config;
use voldb_sqlite3::VolDbSqlite;
use proto;

use PKey;
use OwnedMessage;
use error::Result;

pub trait VolDb {
    fn save_volume(&self, key: &PKey, volume_root_bytes: &[u8]) -> Result<()>;
    fn find_latest<'a,'b,'c>(&self, key: &'a PKey, volid: &'b [u8], min_serial: i64) -> Result<Option<OwnedMessage<proto::volume_root::Reader<'c>>>>;
}

pub fn open(conf: &config::Settings) -> Result<Box<VolDb>> {
    Ok(Box::new(try!(VolDbSqlite::open(&conf.voldb.path, true))))
}

