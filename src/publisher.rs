use std::path::Path;
use std::io;
use std::io::Read;
use std::fs;
use std::fs::{DirEntry, File, FileType, OpenOptions};

use capnp::message::{Builder, HeapAllocator, Reader};
use capnp::serialize_packed;
use openssl::crypto::pkey::PKey;

use Result;
use Error;
use VolDb;
use proto;
use storage_pool_leveldb;
use sha256::Sha256;

const BLOCK_SIZE:usize = 256*1024;


pub struct Publisher {
    storage: storage_pool_leveldb::StoragePoolLeveldb, // TODO: Abstract this.
    voldb: Box<VolDb>,
    options: Options,
}

pub struct Options {
    enc_type: EncType,
}

pub enum EncType {
    None,
    AES256, // Covergent AES_256_CBC with SHA-2 256 as key.
}

impl Publisher {
    pub fn new(s: storage_pool_leveldb::StoragePoolLeveldb, voldb: Box<VolDb>) -> Publisher {
        Publisher{ storage: s, voldb: voldb, options: Options{ enc_type: EncType::AES256 } }
    }
    fn save_raw_block(&self, block:&[u8]) -> Result<Sha256> {
        match self.storage.put(block) {
            Ok(x) => Ok(x),
            Err(e) => Err(Error::other(e))
        }
    }

    fn save_block(&self, block:&[u8]) -> Result<Box<Fn(&mut proto::reference::block_ref::Builder)>> {
        // On success, returns a function to set a block_ref::Builder to point to the stored data.
        let size = block.len() as u64;
        match self.options.enc_type {
            EncType::None => {
                let hash = try!(self.save_raw_block(block));
                Ok(Box::new(move |b: &mut proto::reference::block_ref::Builder| {
                    b.borrow().init_rawblock().set_sha256(hash.as_slice());
                    b.borrow().set_size(size);
                }))
            },
            EncType::AES256 => {
                use openssl::crypto::symm::{encrypt, Type};
                use AES256_IV;
                let key = Sha256::of_bytes(block);
                let ct = encrypt(Type::AES_256_CBC, key.as_slice(), AES256_IV, block);
                let hash = try!(self.save_raw_block(&ct));
                Ok(Box::new(move |b: &mut proto::reference::block_ref::Builder| {
                    b.borrow().init_rawblock().set_sha256(hash.as_slice());
                    b.borrow().get_enc().set_aes(key.as_slice());
                    b.borrow().set_size(size);
                }))
            }
        }
    }
    fn save_data<'a,'b>(&self, data:&[u8], dataref: &'b mut proto::reference::data_ref::Builder<'a>) -> Result<()> {
        // FIXME: Assumes data < BLOCK_SIZE
        let mut br = dataref.borrow().init_block();
        let f = try!(self.save_block(data));
        f(&mut br);
        Ok(())
    }
    /*
    fn save_block(&self, block:&[u8], res: &mut proto::reference::block_ref::Builder) -> Result<&proto::reference::block_ref::Builder, Error> {
    }

    fn save_data(&self, data: Reader, res: &mut proto::reference::data_ref::Builder) -> Result<proto::reference::data_ref::Builder, Error> {
        
    }
     */
    pub fn export_file(&self, path:&Path) -> Result<Box<Fn(&mut proto::reference::block_ref::Builder)>> {
        let mut f = try!(File::open(path));
        let mut buf = [0u8; BLOCK_SIZE];
        let mut message = Builder::new_default();
        {
        let mut indir = message.init_root::<proto::indirect_block::Builder>();
        let mut blocks = vec![];
        
        loop {
            // TODO: Do we need to do anything to ensure we read exactly
            // BLOCK_SIZE bytes unless we are at the end of the file?
            let n = try!(f.read(&mut buf));
            if n == 0 { break; }
            let fun = try!(self.save_block(&buf[0..n]));
            blocks.push(fun);
        }

        { // Use a Scope to limit lifetime of the borrow.
            let mut sub = indir.init_subblocks(blocks.len() as u32);
            for i in 0 .. blocks.len() {
                use db_key::Key;
                let mut block = sub.borrow().get(i as u32).init_block();
                blocks[i](&mut block);
            }
        }
        }
        let mut encoded: Vec<u8> = vec![];
        {
            serialize_packed::write_message(&mut encoded, &mut message);
        }
        self.save_block(&encoded)
    }

    pub fn save_file<'a,'b>(&self, path:&Path, refb: &'b mut proto::reference::Builder<'a>) -> Result<&'b mut proto::reference::Builder<'a>> {
        let fun = try!(self.export_file(path));
        {
            let mut dataref = refb.borrow().init_file();
            let mut br = dataref.borrow().init_indirect();
            fun(&mut br)

        }
        Ok(refb)
    }

    pub fn save_dir<'a, 'b>(&self, path:&Path, refb: &'b mut proto::reference::Builder<'a>) -> Result<&'b mut proto::reference::Builder<'a>> {
        
        let readir: Vec<DirEntry> =
            try!(fs::read_dir(path))
            .flat_map(|e| e) // FIXME: Ignores error entries.
            .collect();
        
        let mut message = Builder::new_default();
        {
            let mut dirb = message.init_root::<proto::directory::Builder>();
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

    fn save_path_with_type<'a, 'b>(&self, path:&Path, typ: FileType, refb: &'b mut proto::reference::Builder<'a>) -> Result<&'b mut proto::reference::Builder<'a>> {
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

    pub fn save_path<'a, 'b>(&self, path:&Path, refb: &'b mut proto::reference::Builder<'a>) -> Result<&'b mut proto::reference::Builder<'a>> {
        let md = try!(fs::symlink_metadata(path));
        self.save_path_with_type(path, md.file_type(), refb)
    }

    fn mk_volume_header(&self, key: &PKey, volid: &[u8], serial: i64, recipients: &[PKey], root: &Fn(&mut proto::reference::Builder) -> Result<()>) -> Result<Vec<u8>> {
        // If recipients is empty, volume is public.
        let mut message = Builder::new_default();
        {
            let mut vh = message.init_root::<proto::volume_header::Builder>();
            vh.set_volume_id(volid);
            vh.set_serial(serial);
            if 0 == recipients.len() {
                try!(root(&mut vh.get_contents().init_public()));
            } else {
                unimplemented!()
            }
        }
        let mut bytes = Vec::new();
        try!(::capnp::serialize::write_message(&mut bytes, &message));
        Ok(bytes)
    }
    pub fn save_volume(&self, key: &PKey, volid: &[u8], serial: i64, recipients: &[PKey], root: &Fn(&mut proto::reference::Builder) -> Result<()>) -> Result<()> {
        let to_sign = try!(self.mk_volume_header(key, volid, serial, recipients, root));
        let h = Sha256::of_bytes(&to_sign);
        // TODO: Do we want to add a prefix to the header before signing?
        let sig = key.sign(h.as_slice());
        let mut message = Builder::new_default();
        {
            let mut vr = message.init_root::<proto::volume_root::Builder>();
            vr.set_header(&to_sign);
            vr.set_signature(&sig);
        }
        let mut bytes = Vec::new();
        try!(::capnp::serialize_packed::write_message(&mut bytes, &message));
        self.voldb.save_volume(key, &bytes)
    }
}
