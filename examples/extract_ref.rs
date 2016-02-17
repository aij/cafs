// Read a regular file into a storage pool and print it's contents.

extern crate rustc_serialize;
extern crate docopt;
extern crate cafs;
extern crate capnp;
extern crate url;

use docopt::Docopt;
use std::path::Path;
use std::io::Write;
use std::str::FromStr;

use cafs::storage_pool_leveldb::StoragePoolLeveldb;
use cafs::proto;
use cafs::OwnedMessage;


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
    let r = OwnedMessage::<proto::reference::Reader>::from_str(&args.arg_reference).unwrap();
    let res = reader.extract_path(r.get().unwrap(), true, Path::new(&args.arg_destination));
    match res {
        Ok(()) => (),
        Err(_) =>
            println_err!("{:?}", res),
    }
}
