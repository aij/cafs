use std::path::Path;
use std::io;
use std::io::Read;
use std::fs;
use std::fs::{DirEntry, File, FileType, OpenOptions};
use std::io::Error;

use capnp::{MessageBuilder, MallocMessageBuilder};
use capnp::serialize_packed;

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
    fn save_raw_block(&self, block:&[u8]) -> Result<Sha256, Error> {
        match self.storage.put(block) {
            Ok(x) => Ok(x),
            Err(e) => Err(Error::new(io::ErrorKind::Other, e))
        }
    }

    fn save_data<'a,'b>(&self, data:&[u8], dataref: &'b mut cafs_capnp::reference::data_ref::Builder<'a>) -> Result<&'b cafs_capnp::reference::data_ref::Builder<'a>, Error> {
        // FIXME: Assumes data < BLOCK_SIZE
        let hash = try!(self.save_raw_block(data));
        {
            let mut rawblock = dataref.borrow().init_indirect().init_rawblock();
            rawblock.set_sha256(hash.as_slice());
        };
        Ok(dataref)
    }
    /*
    fn save_block(&self, block:&[u8], res: &mut cafs_capnp::reference::block_ref::Builder) -> Result<&cafs_capnp::reference::block_ref::Builder, Error> {
    }

    fn save_data(&self, data: Reader, res: &mut cafs_capnp::reference::data_ref::Builder) -> Result<cafs_capnp::reference::data_ref::Builder, Error> {
        
    }
     */
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
            let hash = try!(self.save_raw_block(&buf[0..n]));
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
            serialize_packed::write_message(&mut encoded, &mut message);
        }
        self.save_raw_block(&encoded)
    }

    pub fn save_file<'a,'b>(&self, path:&Path, refb: &'b mut cafs_capnp::reference::Builder<'a>) -> Result<&'b mut cafs_capnp::reference::Builder<'a>, Error> {
        //let mut message = MallocMessageBuilder::new_default();
        let hash = try!(self.export_file(path));
        //let mut refb = message.init_root::<cafs_capnp::reference::Builder>();
        {
            let mut dataref = refb.borrow().init_file();
            let mut rawblock = dataref.borrow().init_indirect().init_rawblock();
            rawblock.set_sha256(hash.as_slice());

        }
        Ok(refb)
    }

    pub fn save_dir<'a, 'b>(&self, path:&Path, refb: &'b mut cafs_capnp::reference::Builder<'a>) -> Result<&'b mut cafs_capnp::reference::Builder<'a>, Error> {
        
        let readir: Vec<DirEntry> =
            try!(fs::read_dir(path))
            .flat_map(|e| e) // FIXME: Ignores error entries.
            .collect();
        
        let mut message = MallocMessageBuilder::new_default();
        {
            let mut dirb = message.init_root::<cafs_capnp::directory::Builder>();
            let mut files = dirb.init_files(readir.len() as u32);

            for (i, dentry) in readir.iter().enumerate() {
                let ft = try!(dentry.file_type());
                let mut entry = files.borrow().get(i as u32);
                entry.set_name(&dentry.file_name().to_string_lossy());
                let mut fref = entry.init_ref();
                self.save_path_with_type(&dentry.path(), ft, &mut fref);
            }
        }
        let mut dirbits: Vec<u8> = vec![];
        serialize_packed::write_message(&mut dirbits, &mut message);
        {
            let mut dref = refb.borrow().init_directory();
            try!(self.save_data(&dirbits, &mut dref));
        }
        Ok(refb)
    }

    fn save_path_with_type<'a, 'b>(&self, path:&Path, typ: FileType, refb: &'b mut cafs_capnp::reference::Builder<'a>) -> Result<&'b mut cafs_capnp::reference::Builder<'a>, Error> {
        if typ.is_dir() {
            self.save_dir(path, refb)
        } else if typ.is_file() {
            self.save_file(path, refb)
        } else if typ.is_symlink() {
            panic!("Symlinks are not yet supported.")
        } else {
            panic!("Unexpected/unknown file type.")
        }
     
    }

    pub fn save_path<'a, 'b>(&self, path:&Path, refb: &'b mut cafs_capnp::reference::Builder<'a>) -> Result<&'b mut cafs_capnp::reference::Builder<'a>, Error> {
        let md = try!(fs::symlink_metadata(path));
        self.save_path_with_type(path, md.file_type(), refb)
    }
}
