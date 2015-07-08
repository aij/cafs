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
use cafs_capnp;
use cafs_capnp::reference::data_ref;
use Result;
use Error;

use capnp;
use capnp::message::MessageReader;

//extern crate core;

#[derive(Clone)]
pub struct Reader {
     // FIXME: Is Arc really right here? Probably not.
    storage: Arc<storage_pool_leveldb::StoragePoolLeveldb>, // TODO: Abstract this.
}

struct BlockReader<'a> {
    reader: Reader,
    bref: cafs_capnp::reference::block_ref::Reader<'a>,
    data: Option<Vec<u8>>,
    position: usize,
}

impl<'a> Read for BlockReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.data {
            None => {
                let d = try!(self.reader.read_blockref_vec(self.bref));
                self.data = Some(d);
                self.read(buf)
            },
            Some(ref data) => {
                let read = min(buf.len(), data.len()-self.position);
                // TODO: Use http://doc.rust-lang.org/std/primitive.slice.html#method.clone_from_slice once it is stable.
                let mut count = 0;
                while count < buf.len() && self.position <= data.len() {
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
    indirect_block: cafs_capnp::indirect_block::Reader<'a>,
    index: u32,
    r: Box<Read>,
}

impl<'a> Read for IndirectBlockReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes = try!(self.r.read(buf));
        if bytes == 0 && buf.len() != 0 {
            self.index += 1;
            let subs = try!(self.indirect_block.get_subblocks());
            if self.index < subs.len() {
                self.r = try!(self.reader.dataref_read(subs.get(self.index)));
                self.read(buf)
            }
            else { Ok(0) }
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
    fn read_blockref_vec(&self, r: cafs_capnp::reference::block_ref::Reader) -> Result<Vec<u8>> {
        assert!(!r.get_enc().has_aes()); // FIXME: Implement.
        let hb = try!(try!(r.get_rawblock()).get_sha256());
        let h = Sha256::from_u8(hb);
        let raw = try!(self.read_rawblock(h));
        Ok(raw)
    }
    fn read_blockref(&self, r: cafs_capnp::reference::block_ref::Reader, out: &mut io::Write) -> Result<()> {
        try!(out.write_all(&try!(self.read_blockref_vec(r))));
        Ok(())
    }

     /*
    FIXME: lifetimes are not working out.
    fn read_indir<'a,'b>(&'a self, bref: cafs_capnp::reference::block_ref::Reader) -> Result<cafs_capnp::indirect_block::Reader<'b>> {
        let indir_bytes = try!(self.read_blockref_vec(bref));
        let mut cursor = io::Cursor::new(indir_bytes);
        let message_reader = try!(capnp::serialize_packed::read_message(&mut cursor, capnp::message::DEFAULT_READER_OPTIONS));
        let reader: cafs_capnp::indirect_block::Reader<'b> = try!(message_reader.get_root());
        Ok(reader)
    }*/
    
    pub fn read_dataref(&self, r: cafs_capnp::reference::data_ref::Reader, out: &mut io::Write) -> Result<()> {
        match r.which() {
            Ok(data_ref::Block(b)) =>
                try!(self.read_blockref(try!(b), out)),
            Ok(data_ref::Inline(i)) =>
               try!(out.write_all(&try!(i))),
            Ok(data_ref::Indirect(ind)) => {
                let indir_bytes = try!(self.read_blockref_vec(try!(ind)));
                let message_reader = try!(capnp::serialize_packed::read_message(&mut io::Cursor::new(indir_bytes), capnp::message::DEFAULT_READER_OPTIONS));
                let reader : cafs_capnp::indirect_block::Reader = try!(message_reader.get_root());
                //FIXME: above should be let reader = try!(self.read_indir(try!(ind)));
                let subs_r = reader.get_subblocks();
                let subs = try!(subs_r);
                for sb in subs.iter() {
                    try!(self.read_dataref(sb, out));
                }
            },
            Err(e) =>
                return Err(Error::new(io::Error::new(io::ErrorKind::Other, e)))
        }
        Ok(())
    }

    fn dataref_read<'c,'b>(&'c self, dr: cafs_capnp::reference::data_ref::Reader<'b>) -> Result<Box<Read>> {
        let w /*: Result<cafs_capnp::reference::data_ref::WhichReader<'b>, ::capnp::NotInSchema>*/ = dr.which();
        unimplemented!();
        /*match w  {
            Ok(data_ref::Block(Ok(b))) => {
                let block = b.clone();
                Ok(Box::new(BlockReader{ reader: self.clone(), bref: block, data: None, position: 0 }))
            },
            Ok(data_ref::Inline(i)) =>
                Ok(Box::new(io::Cursor::new(try!(i)))),
            Ok(data_ref::Indirect(ind)) => {
                let indir_bytes = try!(self.read_blockref_vec(try!(ind)));
                let message_reader = try!(capnp::serialize_packed::read_message(&mut io::Cursor::new(indir_bytes), capnp::message::DEFAULT_READER_OPTIONS));
                let b : cafs_capnp::indirect_block::Reader = try!(message_reader.get_root());
                //let b = try!(self.read_indir(try!(ind)));
                Ok(Box::new(IndirectBlockReader{ reader: self.clone(), indirect_block: b, index: 0, r: Box::new(io::empty()) }))
            },
            Err(e) =>
                Err(io::Error::new(io::ErrorKind::Other, e))
        }
         */
    }

    pub fn read_dataref_vec(&self, r: cafs_capnp::reference::data_ref::Reader) -> Result<Vec<u8>> {
        let mut out = vec![];
        try!(self.read_dataref(r, &mut out));
        Ok(out)
    }

    fn extract_file_data(&self, r: cafs_capnp::reference::data_ref::Reader, create: bool, out: &Path) -> Result<()> {
        let mut file = try!(File::create(out));
        self.read_dataref(r, &mut file)
    }

    pub fn extract_path(&self, r: cafs_capnp::reference::Reader, create: bool, out: &Path) -> Result<()> {
        use cafs_capnp::reference::{File,Directory,Volume};
        println!("Extracting to {}", out.display());
        match r.which() {
            Ok(File(Ok(dr))) =>
                self.extract_file_data(dr, create, out),
            Ok(Directory(Ok(dr))) => {
                // TODO: Shouldn't need to store entire directory in memory.
                let dir_bytes = try!(self.read_dataref_vec(dr));
                println!("got bytes");
                let message_reader = try!(capnp::serialize_packed::read_message(&mut io::Cursor::new(dir_bytes), capnp::message::DEFAULT_READER_OPTIONS));
                let reader : cafs_capnp::directory::Reader = try!(message_reader.get_root());
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
