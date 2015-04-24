//#![feature(core)]

use std;
use std::io;

use sha256::Sha256;
use db_key::Key;
use storage_pool_leveldb;
use cafs_capnp;

use capnp;

//extern crate core;

pub struct Reader {
    storage: storage_pool_leveldb::StoragePoolLeveldb, // TODO: Abstract this.
}

impl Reader {
    pub fn new(s: storage_pool_leveldb::StoragePoolLeveldb) -> Reader {
        Reader{ storage: s }
    }

    fn read_rawblock(&self, h:Sha256) -> io::Result<Vec<u8>> {
        match self.storage.get(h) {
            Ok(None) => Err(io::Error::new(io::ErrorKind::Other, NotFoundError{ what: "hash not found".to_string() })),
            Ok(Some(v)) => Ok(v),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }
    fn read_blockref_vec(&self, r: cafs_capnp::reference::block_ref::Reader) -> io::Result<Vec<u8>> {
        assert!(!r.get_enc().has_aes()); // FIXME: Implement.
        let hb = try!(capnp_result_to_io(try!(capnp_result_to_io(r.get_rawblock())).get_sha256()));
        let h = Sha256::from_u8(hb);
        let raw = try!(self.read_rawblock(h));
        Ok(raw)
    }
    fn read_blockref(&self, r: cafs_capnp::reference::block_ref::Reader, out: &mut io::Write) -> io::Result<()> {
        out.write_all(&try!(self.read_blockref_vec(r)))
    }
    /*
    fn read_dataref(&self, r: cafs_capnp::reference::data_ref::Reader, out: &io::Write) -> io::Result<()> {
        use cafs_capnp::reference::data_ref;
        match r.which() {
            Ok(data_ref::Block(b)) =>
                self.read_blockref(try!(b), out),
            Ok(data_ref::Inline(i)) =>
               out.write_all(&try!(i)),
            Ok(data_ref::Indirect(ind)) => {
                let indir_bytes = try!(self.read_blockref_vec(ind));
                let is = capnp::io::ArrayInputStream::new(&indir_bytes);
                let reader = try!(capnp::serialize_packed::new_reader(is, capnp::message::DEFAULT_READER_OPTIONS)).get_root();
                for sb in try!(reader.get_subblocks()) {
                    try!(self.read_dataref(self, sb, out));
                }
            },
            Err(e) =>
                return capnp_result_to_io(Err(e))
        }
        Ok(())
    }
*/
}

fn capnp_error_to_io(e: capnp::Error) -> io::Error {
       io::Error::new(io::ErrorKind::Other, e)
}
fn capnp_result_to_io<T>(r: Result<T,capnp::Error>) -> io::Result<T> {
    match r {
        Ok(x) => Ok(x),
        Err(e) => Err(capnp_error_to_io(e))
    }
}

#[derive(Debug)]
struct NotFoundError{ what: String }

impl std::error::Error for NotFoundError {
    fn description(&self) -> &str {
        &self.what
    }

    fn cause(&self) -> Option<&std::error::Error> { None }
}

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Not found: {}", self.what)
    }
}
/*
pub struct CapnpError(capnp::Error);

impl core::convert::From<capnp::Error> for CapnpError {
    fn from(e:capnp::Error) -> CapnpError {
        CapnpError(e)
    }
}

impl core::convert::From<CapnpError> for io::Error {
    fn from(e:CapnpError) -> io::Error {
        io::Error::new(io::ErrorKind::Other, e)
    }
}
*/
