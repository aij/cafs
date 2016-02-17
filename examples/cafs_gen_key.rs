extern crate docopt;
extern crate cafs;
extern crate rustc_serialize;

use docopt::Docopt;
use std::fs::File;

use cafs::{PKey, Sha256};

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: cafs_gen_key [--name=<name>]

Options:
    -n, --name=<key_name>  Specify a name for the new key. [default: default]
    -s, --size=<num>  Specify the key size. [default: 4096]
";

#[derive(RustcDecodable, Debug)]
struct Args {
    flag_name: String,
    flag_size: usize,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
    let mut path = cafs::private_key_dir();
    path.push(format!("{}.pem", args.flag_name));
    // FIXME: Open O_EXCL and 0600
    let mut w = File::create(&path).unwrap();
    let mut key = PKey::new();
    key.gen(args.flag_size);
    key.write_pem(&mut w);
    let fingerprint = Sha256::pkey_fingerprint(&key);
    println!("Wrote key {} to {}", fingerprint, path.display());
}
