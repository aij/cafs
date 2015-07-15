// Store a regular file into a storage pool and print the hash of the IndirectBlock

extern crate rustc_serialize;
extern crate docopt;
extern crate capnp;
extern crate cafs;

use docopt::Docopt;
use std::path::Path;
use capnp::{MessageBuilder, MallocMessageBuilder};

use cafs::storage_pool_leveldb::StoragePoolLeveldb;
use cafs::proto;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: store_path [-c] <path> <storage-pool>
       store_path -c <storage-pool>

Options:
    -c, --create  Create new storage pool.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_path: String,
    arg_storage_pool: String,
    flag_create: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    let stor = StoragePoolLeveldb::open(Path::new(&args.arg_storage_pool), args.flag_create).unwrap();
    if (args.arg_path != "") {
        let publisher = cafs::Publisher::new(stor);
        let mut message = MallocMessageBuilder::new_default();
        let mut fref = message.init_root::<proto::reference::Builder>();
        let res = publisher.save_path(Path::new(&args.arg_path), &mut fref);
        match res {
            Ok(fref) => {
                println!("Success!");
                println!("{}", fref.borrow().as_reader())
            },
            Err(e) =>
                println!("Error: {:?}", e),
        }
    }
}
