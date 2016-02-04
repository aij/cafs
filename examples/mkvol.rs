// Store a regular file into a storage pool and print the hash of the IndirectBlock

extern crate rustc_serialize;
extern crate docopt;
extern crate capnp;
extern crate cafs;

use docopt::Docopt;
use std::path::Path;
use std::fs::File;

use cafs::storage_pool_leveldb::StoragePoolLeveldb;
use cafs::proto;
use cafs::{PKey, VolDbSqlite};

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: mkvol [options] <path> <storage-pool> <voldb> <volume-name> [<serial>]

Options:
    --create-storage  Create new storage pool.
    --create-voldb  Create new Volume DB.
    -k, --key-name=<key>  Specify which key to use. [default: default]
    --help  Show help.
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_path: String,
    arg_storage_pool: String,
    arg_voldb: String,
    arg_volume_name: String,
    arg_serial: Option<i64>,
    flag_create_storage: bool,
    flag_create_voldb: bool,
    flag_key_name: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    let stor = StoragePoolLeveldb::open(Path::new(&args.arg_storage_pool), args.flag_create_storage).unwrap();
    let voldb = VolDbSqlite::open(Path::new(&args.arg_voldb), args.flag_create_voldb).unwrap();
    let key = load_key(args.flag_key_name).unwrap();

    let publisher = cafs::Publisher::new(stor, Box::new(voldb));
    let volid = args.arg_volume_name.into_bytes();
    let serial = args.arg_serial.unwrap_or(0);
    {
        let res = save_path_to_vol(publisher, &key, &volid, serial, &[], Path::new(&args.arg_path));
        match res {
            Ok(()) => {
                println!("Success!");
            },
            Err(e) =>
                println!("Error: {:?}", e),
        }
    }
}

fn load_key(name: String) -> cafs::Result<PKey> {
    let mut path = cafs::private_key_dir();
    path.push(format!("{}.pem", name));
    let mut f = try!(File::open(&path));
    Ok(try!(PKey::private_key_from_pem(&mut f)))
}

fn save_path_to_vol(publisher: cafs::Publisher, key: &PKey, volid: &[u8], serial: i64, recipients: &[PKey], path: &Path) -> cafs::Result<()> {
    publisher.save_volume(key, volid, serial, &[],
                          &|r| {
                              try!(publisher.save_path(path, r));
                              Ok(())
                          })
}
