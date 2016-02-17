extern crate leveldb;
extern crate db_key;
extern crate openssl;
extern crate capnp;
extern crate rustc_serialize;
extern crate sqlite3;
extern crate fuse;
extern crate time;
//extern crate toml_config;

use rustc_serialize::base64;
use rustc_serialize::base64::ToBase64;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use capnp::message::Builder;

pub mod config;
pub mod storage_pool_leveldb;
mod voldb_sqlite3;
pub use voldb_sqlite3::VolDbSqlite;
mod sha256;
mod publisher;
mod reader;
mod error;
mod voldb;
mod storage;
mod fs;
pub use fs::Fs;

#[allow(dead_code)]
pub mod proto {
    include!("schema/cafs_capnp.rs");
}
pub use proto as cafs_capnp; // Because the generated code refers to itself as ::cafs_capnp rather than ::cafs::proto.


pub use publisher::Publisher;
pub use reader::Reader;
pub use sha256::Sha256;
pub use openssl::crypto::pkey::PKey;
pub use voldb::VolDb;
pub use error::{Result, Error};

static AES256_IV:[u8;16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub fn private_key_dir() -> PathBuf {
    let mut h = ::std::env::home_dir().unwrap_or_else(|| Path::new("/etc").to_path_buf());
    h.push(".cafs");
    h.push("private_keys");
    h
}

impl<'a> fmt::Display for proto::reference::Reader<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //use proto::reference::Builder;
        //use capnp::traits::CastableTo;
        let mut message = Builder::new_default();
        message.set_root(self.clone());
        let bytes = message_to_bytes(&message);
        println!("bytes = {:?}", bytes);

        let b64 = bytes.to_base64(base64::URL_SAFE);
        write!(f, "cafs:///ref/{}", b64)
    }
}

pub fn message_to_bytes<A>(message: &Builder<A>) -> Vec<u8> where A: capnp::message::Allocator {
    let mut encoded: Vec<u8> = vec![];
    capnp::serialize_packed::write_message(&mut encoded, message);
    encoded
}


pub struct OwnedMessage<T> {
    message: ::capnp::message::Reader<::capnp::serialize::OwnedSegments>,
    phantom_data: ::std::marker::PhantomData<T>
}

impl <'a, T> OwnedMessage <T> where T: ::capnp::traits::FromPointerReader<'a> {
    pub fn new(mr: ::capnp::message::Reader<::capnp::serialize::OwnedSegments>) -> OwnedMessage<T> {
        OwnedMessage { message: mr, phantom_data: ::std::marker::PhantomData }
    }
    pub fn get(&'a self) -> ::capnp::Result<T> {
        use capnp::message::Reader;
        self.message.get_root()
    }
}


trait ToOwnedMessage<T> {
    fn to_owned_message(self) -> capnp::Result<OwnedMessage<T>>;
}

impl<'a,'b> ToOwnedMessage<proto::reference::block_ref::Reader<'a>> for proto::reference::block_ref::Reader<'b> {
    fn to_owned_message(self) -> capnp::Result<OwnedMessage<proto::reference::block_ref::Reader<'a>>> {
        let mut buffer = Vec::new();
        {
            let mut message = Builder::new_default();
            message.set_root(self);
            capnp::serialize::write_message(&mut buffer, &message);
        }
        let mr = capnp::serialize::read_message(&mut io::Cursor::new(buffer), capnp::message::DEFAULT_READER_OPTIONS);
        Ok(OwnedMessage::new(try!(mr)))
    }
}


impl<'a> std::str::FromStr for OwnedMessage<proto::reference::Reader<'a>> {

    type Err = Error;
    
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use rustc_serialize::base64::FromBase64;
       assert_eq!(&s[0..12], "cafs:///ref/");
        let b64 = &s[12..];
        println!("b64 = {}", b64);    
        let bytes = b64.from_base64().unwrap();
        println!("bytes = {:?}", bytes);

        let message_reader = try!(capnp::serialize_packed::read_message(&mut io::Cursor::new(bytes), capnp::message::DEFAULT_READER_OPTIONS));
        Ok(OwnedMessage::new(message_reader))
/*        let reader : proto::reference::Reader = message_reader.get_root().unwrap();
        // FIXME: Is there a less stupid way to return a Reader?
        message.set_root(reader).unwrap();
        message.get_root::<proto::reference::Builder>().unwrap().as_reader()
*/
    }
}
