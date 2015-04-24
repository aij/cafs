// Store a regular file into a storage pool and print the hash of the IndirectBlock

extern crate rustc_serialize;
extern crate docopt;
extern crate cafs;

use docopt::Docopt;
use std::path::Path;

use cafs::storage_pool_leveldb::StoragePoolLeveldb;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: cp [-c] <file> <storage-pool>
       cp -c <storage-pool>

Options:
    -c, --create  Create new storage pool.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_file: String,
    arg_storage_pool: String,
    flag_create: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    let stor = StoragePoolLeveldb::open(Path::new(&args.arg_storage_pool), args.flag_create).unwrap();
    if (args.arg_file != "") {
        let publisher = cafs::Publisher::new(stor);
        let res = publisher.export_file(Path::new(&args.arg_file));
        match res {
            Ok(h) =>
                println!("{}", h),
            Err(_) =>
                println!("{:?}", res),
        }
    }
}
