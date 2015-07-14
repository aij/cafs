extern crate backtrace;

pub type Result<T> = ::std::result::Result<T, Error>;

use std;
use std::io;
use std::io::Write;
use capnp;

#[derive(Debug)]
pub struct Error {
    ioerr: io::Error,
    backtrace: String,
}

impl Error {
    pub fn new(e: io::Error) -> Error {
        panic!("cafs error created! {:?}", e);
        Error { ioerr: e, backtrace: get_backtrace_now() }
    }

    pub fn other<E> (e: E) -> Error where E: Into<Box<std::error::Error + Send + Sync>> {
        Error::new(io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn get_backtrace_str(&self) -> &str {
        &self.backtrace
    }
}

impl std::convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::new(e)
    }
}

impl std::convert::From<Error> for io::Error {
    fn from(e: Error) -> io::Error {
        e.ioerr
    }
}

//impl std::convert::From<std::error::Error> for Error {
impl std::convert::From<capnp::Error> for Error {
    fn from(e: capnp::Error) -> Error {
        Error::other(e)
    }
}

    
fn get_backtrace_now() -> String {
    let mut bt = vec![];
    backtrace::trace(&mut |frame| {
        let ip = frame.ip();
        let symbol_address = frame.symbol_address();

        write!(bt, "@{}", ip as u64, );
        
        // Resolve this instruction pointer to a symbol name
        backtrace::resolve(ip, &mut |symbol| {
            if let Some(name) = symbol.name() {
                write!(bt, ": ");
                let sname = String::from_utf8_lossy(name);
                //backtrace::demangle(&mut bt, &sname);
                write!(bt, "{}", sname);
                write!(bt, "()");
            }
            if let Some(filename) = symbol.filename() {
                write!(bt, ": {}", String::from_utf8_lossy(&filename));
            }
            if let Some(line) = symbol.lineno() {
                write!(bt, ":{}", line);
            }
        });
        writeln!(bt, "");
        true
    });
    String::from_utf8_lossy(&bt).to_string()
}
