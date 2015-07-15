extern crate leveldb;
extern crate db_key;
extern crate openssl;
extern crate capnp;
extern crate rustc_serialize;

use rustc_serialize::base64;
use rustc_serialize::base64::ToBase64;
use std::fmt;
use std::io;
use capnp::message::MessageBuilder;

pub mod storage_pool_leveldb;
mod sha256;
mod cafs_publish;
mod reader;
mod error;

#[allow(dead_code)]
pub mod cafs_capnp {
    include!("schema/cafs_capnp.rs");
}

pub use cafs_publish::Publisher;
pub use reader::Reader;
pub use sha256::Sha256;

pub use error::{Result, Error};

impl<'a> fmt::Display for cafs_capnp::reference::Reader<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //use cafs_capnp::reference::Builder;
        use capnp::traits::CastableTo;
        let mut message = capnp::message::MallocMessageBuilder::new_default();
        message.set_root(self.clone());
        let bytes = message_to_bytes(&message);
        println!("bytes = {:?}", bytes);

        let b64 = bytes.to_base64(base64::URL_SAFE);
        write!(f, "cafs:///ref/{}", b64)
    }
}

pub fn message_to_bytes<M>(message: &M) -> Vec<u8> where M: capnp::MessageBuilder {
    let mut encoded: Vec<u8> = vec![];
    capnp::serialize_packed::write_message(&mut encoded, message);
    encoded
}

pub struct OwnedMessage<T> {
    message: ::capnp::OwnedSpaceMessageReader,
    phantom_data: ::std::marker::PhantomData<T>
}

impl <'a, T> OwnedMessage <T> where T: ::capnp::traits::FromPointerReader<'a> {
    pub fn new(mr: ::capnp::OwnedSpaceMessageReader) -> OwnedMessage<T> {
        OwnedMessage { message: mr, phantom_data: ::std::marker::PhantomData }
    }
    pub fn get(&'a self) -> ::capnp::Result<T> {
        use capnp::MessageReader;
        self.message.get_root()
    }
}


trait ToOwnedMessage<T> {
    fn to_owned_message(self) -> capnp::Result<OwnedMessage<T>>;
}

impl<'a,'b> ToOwnedMessage<cafs_capnp::reference::block_ref::Reader<'a>> for cafs_capnp::reference::block_ref::Reader<'b> {
    fn to_owned_message(self) -> capnp::Result<OwnedMessage<cafs_capnp::reference::block_ref::Reader<'a>>> {
        let mut buffer = Vec::new();
        {
            let mut message = ::capnp::MallocMessageBuilder::new_default();
            message.set_root(self);
            capnp::serialize::write_message(&mut buffer, &message);
        }
        let mr = capnp::serialize::read_message(&mut io::Cursor::new(buffer), capnp::message::DEFAULT_READER_OPTIONS);
        Ok(OwnedMessage::new(try!(mr)))
    }
}
