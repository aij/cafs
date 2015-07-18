//#![feature(core)]

use std;
use std::io;
use std::io::Read;
use std::cmp::min;
use std::boxed::Box;
use std::sync::Arc;
use std::fs;
use std::fs::File;
use std::path::Path;

use sha256::Sha256;
use db_key::Key;
use storage_pool_leveldb;
use proto;
use proto::reference::data_ref;
use Result;
use Error;
use OwnedMessage;

use capnp;
use capnp::message::MessageReader;


#[derive(Clone)]
pub struct Reader {
     // FIXME: Is Arc really right here? Probably not.
    storage: Arc<storage_pool_leveldb::StoragePoolLeveldb>, // TODO: Abstract this.
}

struct BlockReader<'a> {
    reader: Reader,
    bref: OwnedMessage<proto::reference::block_ref::Reader<'a>>,
    data: Option<Vec<u8>>,
    position: usize,
}

impl<'a> Read for BlockReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.data {
            None => {
                let d = {
                    let br = try!(capnp_result_to_io(self.bref.get()));
                    try!(self.reader.read_blockref_vec(&br))
                };
                self.data = Some(d);
                self.read(buf)
            },
            Some(ref data) => {
                let read = min(buf.len(), data.len()-self.position);
                // TODO: Use http://doc.rust-lang.org/std/primitive.slice.html#method.clone_from_slice once it is stable.
                let mut count = 0;
                while count < buf.len() && self.position < data.len() {
                    buf[count] = data[self.position];
                    count += 1;
                    self.position += 1;
                }
                assert_eq!(read, count);
                Ok(count)
            }
        }
    }
}

pub struct IndirectBlockReader<'a> {
    reader: Reader,
    indirect_block: OwnedMessage<proto::indirect_block::Reader<'a>>,
    next_index: u32,
    r: Box<Read>,
}

impl<'a> Read for IndirectBlockReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes = try!(self.r.read(buf));
        if bytes == 0 && buf.len() != 0 {
            let retry = { // Scope to limit the borrow.
                let indir = try!(capnp_result_to_io(self.indirect_block.get()));
                let subs = try!(capnp_result_to_io(indir.get_subblocks()));
                if self.next_index < subs.len() {
                    self.r = try!(self.reader.dataref_read(subs.get(self.next_index)));
                    self.next_index += 1;
                    true
                }
                else { false }
            };
            if retry {
                self.read(buf)
            } else {
                Ok(0)
            }
        }
        else { Ok(bytes) }
    }
}


impl Reader {
    pub fn new(s: storage_pool_leveldb::StoragePoolLeveldb) -> Reader {
        Reader{ storage: Arc::new(s) }
    }

    fn read_rawblock(&self, h:Sha256) -> Result<Vec<u8>> {
        match self.storage.get(h) {
            Ok(None) => Err(Error::other(NotFoundError{ what: "hash not found".to_string() })),
            Ok(Some(v)) => Ok(v),
            Err(e) => Err(Error::other(e)),
        }
    }
    fn read_blockref_vec(&self, r: &proto::reference::block_ref::Reader) -> Result<Vec<u8>> {
        use proto::reference::block_ref;
        let hb = try!(try!(r.get_rawblock()).get_sha256());
        let h = Sha256::from_u8(hb);
        let raw = try!(self.read_rawblock(h.clone()));
        match r.get_enc().which() {
            Ok(block_ref::enc::None(())) => {
                Ok(raw)
            }
            Ok(block_ref::enc::Aes(k)) => {
                use openssl::crypto::symm::{decrypt, Type};
                use AES256_IV;
                let key = try!(k);
                let plain = decrypt(Type::AES_256_CBC, key, AES256_IV, &raw);
                let hash = Sha256::of_bytes(&plain);
                if (r.get_size() != 0) {
                    assert_eq!(r.get_size(), plain.len() as u64);
                }
                assert_eq!(hash, Sha256::from_u8(key)); // FIXME
                Ok(plain)
            }
            Err(e) =>
                Err(Error::other(e)),
        }
    }

    fn read_indir<'a,'b>(&self, bref: &proto::reference::block_ref::Reader<'a>) -> Result<OwnedMessage<proto::indirect_block::Reader<'b>>> {
        let indir_bytes = try!(self.read_blockref_vec(bref));
        let mut cursor = io::Cursor::new(indir_bytes);
        let message_reader = try!(capnp::serialize_packed::read_message(&mut cursor, capnp::message::DEFAULT_READER_OPTIONS));
        Ok(OwnedMessage::new(message_reader))
    }

    pub fn dataref_read<'c,'b>(&'c self, dr: proto::reference::data_ref::Reader<'b>) -> Result<Box<Read>> {
        use ToOwnedMessage;
        match dr.which()  {
            Ok(data_ref::Block(Ok(b))) => {
                let block = try!(b.to_owned_message());
                Ok(Box::new(BlockReader{ reader: self.clone(), bref: block, data: None, position: 0 }))
            },
            Ok(data_ref::Inline(i)) =>
              Ok(Box::new(io::Cursor::new(try!(i).to_vec()))),
            
            Ok(data_ref::Indirect(ind)) => {
                let indir = try!(self.read_indir(&try!(ind)));
                Ok(Box::new(IndirectBlockReader{ reader: self.clone(), indirect_block: indir, next_index: 0, r: Box::new(io::empty()) }))
            },
            Err(e) =>
                Err(Error::other(e)),
            _ => unimplemented!()
        }
    }

    fn extract_file_data(&self, r: proto::reference::data_ref::Reader, create: bool, out: &Path) -> Result<()> {
        let mut file = try!(File::create(out));
        let mut read = try!(self.dataref_read(r));
        let bytes = try!(io::copy(&mut read, &mut file));
        Ok(())
    }

    pub fn extract_path(&self, r: proto::reference::Reader, create: bool, out: &Path) -> Result<()> {
        use proto::reference::{File,Directory,Volume};
        println!("Extracting to {}", out.display());
        match r.which() {
            Ok(File(Ok(dr))) =>
                self.extract_file_data(dr, create, out),
            Ok(Directory(Ok(dr))) => {
                let mut dir_read = try!(self.dataref_read(dr));
                let mut dir_bufread = io::BufReader::new(dir_read); // TODO: Implement natively.
                let message_reader = try!(capnp::serialize_packed::read_message(&mut dir_bufread, capnp::message::DEFAULT_READER_OPTIONS));
                let reader : proto::directory::Reader = try!(message_reader.get_root());
                if create {
                    try!(fs::create_dir(out));
                }
                for f in try!(reader.get_files()).iter() {
                    let path = out.join(try!(f.get_name()));
                    assert_eq!(path.parent(), Some(out)); //FIXME
                    try!(self.extract_path(try!(f.get_ref()), create, &path));
                }
                Ok(())
            }
                
            _ =>
                unimplemented!()
        }
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

fn capnp_error_to_io(e: capnp::Error) -> io::Error {
       io::Error::new(io::ErrorKind::Other, e)
}
fn capnp_result_to_io<T>(r: std::result::Result<T,capnp::Error>) -> io::Result<T> {
    match r {
        Ok(x) => Ok(x),
        Err(e) => Err(capnp_error_to_io(e))
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
