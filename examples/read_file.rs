// Read a regular file into a storage pool and print it's contents.

extern crate rustc_serialize;
extern crate docopt;
extern crate cafs;
extern crate capnp;

use docopt::Docopt;
use std::path::Path;
use std::io::Write;   

use cafs::storage_pool_leveldb::StoragePoolLeveldb;
use cafs::cafs_capnp;
use cafs::Sha256;

use capnp::{MessageBuilder, MallocMessageBuilder};

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: read_file <storage-pool> <hash>
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_storage_pool: String,
    arg_hash: String,
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
    let mut message = MallocMessageBuilder::new_default();
    let dataref = mk_dataref(args, &mut message);
    let res = reader.read_dataref(dataref, &mut ::std::io::stdout());
    match res {
        Ok(()) => (),
        Err(_) =>
            println_err!("{:?}", res),
    }
}

// FIXME: Can we avoid having to take a MallocMessageBuilder here?
fn mk_dataref<'a>(args: Args, message: &'a mut MallocMessageBuilder) -> cafs_capnp::reference::data_ref::Reader<'a> {
    let sha = Sha256::from_hex(&args.arg_hash);

    let mut dr = message.init_root::<cafs_capnp::reference::data_ref::Builder>();
    {
        let mut indir = dr.borrow().init_indirect();
        let mut rawblock = indir.init_rawblock();
        rawblock.set_sha256(sha.as_slice());
    }
    dr.as_reader()
}
