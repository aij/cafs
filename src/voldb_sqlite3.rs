use std::path::Path;
use std::io;

use sqlite3;
use sqlite3::{DatabaseConnection, Query, ResultRowAccess, StatementUpdate, SqliteResult, SqliteError};
use capnp;
use capnp::MessageReader;

use Result;
use VolDb;
use Sha256;
use OwnedMessage;
use PKey;
use proto;

pub struct VolDbSqlite {
    db: DatabaseConnection,
}

impl VolDbSqlite {
    pub fn open(path: &Path, create: bool) -> Result<VolDbSqlite> {
        // FIXME: Check create.
        let mut conn = try!(sqlite3::access::open(&path.to_string_lossy(), None));
        try!(VolDbSqlite::init_db(&mut conn));
        Ok(VolDbSqlite{ db: conn })
    }
    fn init_db(db: &mut DatabaseConnection) -> Result<()> {
        try!(db.exec("CREATE TABLE IF NOT EXISTS volumes (
                          key BLOB,
                          volid BLOB,
                          serial INTEGER,
                          volume_root BLOB,
                          PRIMARY KEY (key, volid, serial)
                      )"));
        Ok(())
    }
}
impl VolDb for VolDbSqlite {
    fn save_volume(&self, key: &PKey, volume_root_bytes: &[u8]) -> Result<()> {
        let mut ins = try!(self.db.prepare("INSERT INTO volumes (key, volid, serial, volume_root) \
                                            VALUES ($1, $2, $3, $4)"));
        let fingerprint = Sha256::pkey_fingerprint(key);
        let mr = try!(capnp::serialize_packed::read_message(&mut io::Cursor::new(volume_root_bytes), capnp::message::DEFAULT_READER_OPTIONS));
        let vol = try!(mr.get_root::<proto::volume_root::Reader>());
        let vh_bytes = try!(vol.get_header());
        let mut cursor = io::Cursor::new(vh_bytes);
        let vh_m = try!(::capnp::serialize::read_message(&mut cursor, ::capnp::message::DEFAULT_READER_OPTIONS));
        let vh = try!(vh_m.get_root::<proto::volume_header::Reader>());
        let volid = try!(vh.get_volume_id());
        let serial = vh.get_serial();
        // TODO: Check signature?
        let changed = try!(ins.update(&[&fingerprint.as_slice(), &volid, &serial, &volume_root_bytes]));
        assert_eq!(changed, 1); // FIXME
        Ok(())
    }
    
    fn find_latest<'a,'b,'c>(&self, key: &'a PKey, volid: &'b [u8], min_serial: i64) -> Result<Option<OwnedMessage<proto::volume_root::Reader<'c>>>> {
        let mut q = try!(self.db.prepare("SELECT volume_root FROM volumes \
                                            WHERE key == $1 AND volid == $2 AND serial > $3 \
                                            ORDER BY serial DESCENDING \
                                            LIMIT 1"));
        let fingerprint = Sha256::pkey_fingerprint(key);
        let mut bytes: Option<Vec<u8>> = None;
        try!(q.query(&[&fingerprint.as_slice(), &volid, &min_serial],
                     &mut |row| {
                         bytes = Some(row.get(0));
                         Ok(())
                     }));
        match bytes {
            Some(b) => {
                let mr = try!(capnp::serialize_packed::read_message(&mut io::Cursor::new(b), capnp::message::DEFAULT_READER_OPTIONS));
                Ok(Some(OwnedMessage::new(mr)))
            },
            None => Ok(None)
        }
    }
    //fn find<'a,'b,'c>(key: &'a PKey, volid: &'b [u8], serial: u64) -> Result<OwnedMessage<proto::volume_root::Reader<'c>>>;

}
