
// TODO: Why does fuse crate require libfuse-dev? It's not a wrapper...
//thread '<main>' panicked at 'called `Result::unwrap()` on an `Err` value: "`\"pkg-config\" \"--libs\" \"--cflags\" \"fuse\"` did not exit successfully: exit code: 1\n--- stderr\nPackage fuse was not found in the pkg-config search path.\nPerhaps you should add the directory containing `fuse.pc\'\nto the PKG_CONFIG_PATH environment variable\nNo package \'fuse\' found\n"', ../src/libcore/result.rs:741

use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};
use reader::Reader;
use std::path::Path;
use std::str::FromStr;
use error::Result;
use proto;
use OwnedMessage;
use time::Timespec;
    
pub struct Fs {
    thing: usize
}

impl Fs {
    pub fn new(url: &str) -> Result<Fs> {
        let r = try!(OwnedMessage::<proto::reference::Reader>::from_str(url));
        Ok(Fs{ thing: 5 })
    }
}


impl Filesystem for Fs {
    fn lookup (&mut self, _req: &Request, parent: u64, name: &Path, reply: ReplyEntry) {
        println!("lookup");
    }

    fn getattr (&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        let time = Timespec::new(0, 0); // TODO
        //reply.
        println!("getattr");
    }

    fn read (&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, _size: u32, reply: ReplyData) {
        println!("read");
    }

    fn readdir (&mut self, _req: &Request, ino: u64, _fh: u64, offset: u64, mut reply: ReplyDirectory) {
        println!("readdir");
    }

}
