// Read a regular file into a storage pool and print it's contents.

extern crate rustc_serialize;
extern crate docopt;
extern crate cafs;
extern crate capnp;
extern crate url;

use docopt::Docopt;
use std::path::Path;
use std::io;
use std::io::Write;
use url::Url;
use rustc_serialize::base64::FromBase64;

use cafs::storage_pool_leveldb::StoragePoolLeveldb;
use cafs::proto;
use cafs::Sha256;

use capnp::message;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: extract_ref <storage-pool> <reference> <destination>
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_storage_pool: String,
    arg_reference: String,
    arg_destination: String,
}


macro_rules! println_err(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);


fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    println_err!("{:?}", args);

    let stor = StoragePoolLeveldb::open(Path::new(&args.arg_storage_pool), false).unwrap();
    let reader = cafs::Reader::new(stor);
    let mut message = message::Builder::new_default();
    let r = mk_ref(&args, &mut message);
    let res = reader.extract_path(r, true, Path::new(&args.arg_destination));
    match res {
        Ok(()) => (),
        Err(_) =>
            println_err!("{:?}", res),
    }
}

fn mk_ref<'a, A>(args: &'a Args, message: &'a mut message::Builder<A>) -> proto::reference::Reader<'a> where A: capnp::message::Allocator {

    let r = &args.arg_reference;
    assert_eq!(&r[0..12], "cafs:///ref/");
    let b64 = &r[12..];
    println!("b64 = {}", b64);    
    let bytes = b64.from_base64().unwrap();
    println!("bytes = {:?}", bytes);

    let message_reader = capnp::serialize_packed::read_message(&mut io::Cursor::new(bytes), capnp::message::DEFAULT_READER_OPTIONS).unwrap();
    let reader : proto::reference::Reader = message_reader.get_root().unwrap();
    // FIXME: Is there a less stupid way to return a Reader?
    message.set_root(reader).unwrap();
    message.get_root::<proto::reference::Builder>().unwrap().as_reader()
}
