use std::path::Path;
use std::io;
use std::io::Read;
use std::fs::{File, OpenOptions};
use std::io::Error;

use capnp::{MessageBuilder, MallocMessageBuilder};
use capnp::serialize_packed;
use capnp::io::BufferedOutputStreamWrapper;

use cafs_capnp;
use storage_pool_leveldb;
use sha256::Sha256;

const BLOCK_SIZE:usize = 256*1024;


pub struct Publisher {
    storage: storage_pool_leveldb::StoragePoolLeveldb, // TODO: Abstract this.
}

impl Publisher {
    pub fn new(s: storage_pool_leveldb::StoragePoolLeveldb) -> Publisher {
        Publisher{ storage: s }
    }
    fn save_block(&self, block:&[u8]) -> Result<Sha256, Error> {
        match self.storage.put(block) {
            Ok(x) => Ok(x),
            Err(e) => Err(Error::new(io::ErrorKind::Other, e))
        }
    }

    pub fn export_file(&self, path:&Path) -> Result<Sha256, Error> {
        let mut f = try!(File::open(path));
        let mut buf = [0u8; BLOCK_SIZE];
        let mut message = MallocMessageBuilder::new_default();
        {
        let mut indir = message.init_root::<cafs_capnp::indirect_block::Builder>();
        let mut blocks = vec![];
        
        loop {
            // TODO: Do we need to do anything to ensure we read exactly
            // BLOCK_SIZE bytes unless we are at the end of the file?
            let n = try!(f.read(&mut buf));
            if n == 0 { break; }
            let hash = try!(self.save_block(&buf[0..n]));
            blocks.push((hash,n));
        }

        { // Use a Scope to limit lifetime of the borrow.
            let mut sub = indir.init_subblocks(blocks.len() as u32);
            for i in 0 .. blocks.len() {
                use db_key::Key;
                let mut block = sub.borrow().get(i as u32).init_block();
                {
                    let mut rawblock = block.borrow().init_rawblock();
                    //blocks[i].0.as_slice(|s| rawblock.set_sha256(s));
                    rawblock.set_sha256(blocks[i].0.as_slice());
                }
                block.set_size(blocks[i].1 as u64);
            }
        }
        }
        let mut encoded: Vec<u8> = vec![];
        {
            let mut bos = BufferedOutputStreamWrapper::new(&mut encoded);
            serialize_packed::write_packed_message(&mut bos, &mut message);
        }
        self.save_block(&encoded)
    }
}
