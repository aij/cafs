// Store a regular file into a storage pool and print the hash of the IndirectBlock

extern crate rustc_serialize;
extern crate docopt;
extern crate capnp;
extern crate cafs;

use docopt::Docopt;
use std::path::Path;
use capnp::message::Builder;

use cafs::proto;

// Write the Docopt usage string.
static USAGE: &'static str = "
Usage: store_path <path>
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_path: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    if args.arg_path != "" {
        let publisher = cafs::Publisher::new_default().unwrap();
        let mut message = Builder::new_default();
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
