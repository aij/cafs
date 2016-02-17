
extern crate fuse;
extern crate cafs;

fn main () {
    let src = std::env::args().nth(1).unwrap();
    let mountpoint = std::env::args_os().nth(2).unwrap();
    fuse::mount(cafs::Fs::new(&src).unwrap(), &mountpoint, &[]);
}
