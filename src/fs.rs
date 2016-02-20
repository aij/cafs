
// TODO: Why does fuse crate require libfuse-dev? It's not a wrapper...
//thread '<main>' panicked at 'called `Result::unwrap()` on an `Err` value: "`\"pkg-config\" \"--libs\" \"--cflags\" \"fuse\"` did not exit successfully: exit code: 1\n--- stderr\nPackage fuse was not found in the pkg-config search path.\nPerhaps you should add the directory containing `fuse.pc\'\nto the PKG_CONFIG_PATH environment variable\nNo package \'fuse\' found\n"', ../src/libcore/result.rs:741

use fuse;
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory, ReplyOpen};
use reader::Reader;
use std::path::Path;
use std::str::FromStr;
use error::{Error, Result};
use proto;
use OwnedMessage;
use time::Timespec;
use std::collections::HashMap;
use std::io::Read;
use config;
use reader;
use storage;

enum InodeData {
    FileContents(Box<Read>),
    DirContents//()  FIXME
}

struct Inode<'a> {
    refr: OwnedMessage<proto::reference::Reader<'a>>,
    data: Option<InodeData>,
}

pub struct Fs<'a> {
    settings: config::Settings,
    reader: reader::Reader,
    inodes: HashMap<u64, Inode<'a>>,
    //root: OwnedMessage<proto::reference::Reader<'a>>,
}

fn get_filekind(r: &proto::reference::Reader) -> Result<fuse::FileType> {
    use proto::reference::{File,Directory,Volume};
    match r.which() {
        Ok(Directory(Ok(_))) => Ok(FileType::Directory),
        Ok(File(Ok(_))) => Ok(FileType::RegularFile),
        _ => Err(Error::str("Unimplemented file type or error"))
    }
}

impl <'a> Fs<'a> {
    pub fn new(url: &str) -> Result<Fs> {
        let r = try!(OwnedMessage::<proto::reference::Reader>::from_str(url));
        let mut m = HashMap::new();
        m.insert(1, Inode{ refr: r, data: None });
        // TODO: Check root inode reference is valid.
        let settings = config::load();
        let pool = try!(storage::open(&settings));
        Ok(Fs{
            settings: settings,
            reader: Reader::new(*pool),
            inodes: m
        })
    }
    
    fn getattr(ino: u64, inode: &Inode) -> Result<fuse::FileAttr> {
        let r = &try!(inode.refr.get());
        let notime = Timespec::new(0, 0); // TODO
        let kind = try!(get_filekind(r));
        
        Ok(FileAttr {
            ino: ino,
            size: 42, // TODO
            blocks: 42, // TODO
            atime: notime,
            mtime: notime,
            ctime: notime,
            crtime: notime,
            kind: kind,
            perm: 0o777,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        })
    }

    fn readdir(&mut self, ino: u64, offset: u64, mut reply: ReplyDirectory) -> Result<()> {
        let inode = match self.inodes.get(&ino) {
            Some(i) => i,
            None => return Err(Error::str("readdir: inode not found FIXME"))
        };
        match try!(try!(inode.refr.get()).which()) {
            proto::reference::Directory(dir) => {
                let dr = try!(reader::DirectoryReader::new(&self.reader, try!(dir)));
                for (i, d) in (&dr).into_iter().enumerate().skip(offset as usize) {
                    println!("readir: {:?}: ", i);
                    let r = try!(d.get_ref());
                    let kind = try!(get_filekind(&r));
                    let name = try!(d.get_name());
                    // TODO: Generate inode number for entry.
                    if reply.add(57, 1 + i as u64, kind, name) {
                        break;
                    }
                    //let full = reply.add()
                }
                Ok(reply.ok())
            }
            _ => Ok(()) // FIXME: Not
        }

    }

}


impl <'a> Filesystem for Fs<'a> {
    fn lookup (&mut self, _req: &Request, parent: u64, name: &Path, reply: ReplyEntry) {
        println!("lookup");
    }

    fn getattr (&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr {}", ino);
        let ttl = Timespec::new(0, 0); // TODO
        match self.inodes.get(&ino) {
            Some(inode) => {
                let r = Fs::getattr(ino, inode).unwrap();
                reply.attr(&ttl, &r);
                println!("getattr replied");
            },
            None =>
                println!("gettatr inode {} not found", ino)
        }
        //reply.
    }

    fn read (&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, _size: u32, reply: ReplyData) {
        println!("read");
    }

    fn readdir (&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, mut reply: ReplyDirectory) {
        println!("readdir");
        self.readdir(ino, offset, reply).unwrap()
    }

    /*fn opendir(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        println!("opendir");
    }*/
}
