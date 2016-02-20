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

    pub fn str(s: &str) -> Error {
        Error::new(io::Error::new(io::ErrorKind::Other, s))
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
impl std::convert::From<capnp::NotInSchema> for Error {
    fn from(e: capnp::NotInSchema) -> Error {
        Error::other(e)
    }
}

impl std::convert::From<::sqlite3::SqliteError> for Error {
    fn from(e: ::sqlite3::SqliteError) -> Error {
        Error::other(e)
   }
}

impl std::convert::From<::openssl::ssl::error::SslError> for Error {
    fn from(e: ::openssl::ssl::error::SslError) -> Error {
        Error::other(e)
   }
}

impl std::convert::From<::leveldb::database::error::Error> for Error {
    fn from(e: ::leveldb::database::error::Error) -> Error {
        Error::other(e)
   }
}

/*
impl<E> std::convert::From<E> for Error  where E: Into<Box<std::error::Error + Send + Sync>> {
    fn from(e: E) -> Error {
        Error::other(e)
   }
}*/

fn get_backtrace_now() -> String {
    let mut bt = vec![];
    backtrace::trace(&mut |frame| {
        let ip = frame.ip();
        //let symbol_address = frame.symbol_address();

        write!(bt, "@{}", ip as u64, ).ok();
        
        // Resolve this instruction pointer to a symbol name
        backtrace::resolve(ip, &mut |symbol| {
            if let Some(name) = symbol.name() {
                write!(bt, ": ").ok();
                let sname = String::from_utf8_lossy(name);
                //backtrace::demangle(&mut bt, &sname);
                write!(bt, "{}", sname).ok();
                write!(bt, "()").ok();
            }
            if let Some(filename) = symbol.filename() {
                write!(bt, ": {}", String::from_utf8_lossy(&filename)).ok();
            }
            if let Some(line) = symbol.lineno() {
                write!(bt, ":{}", line).ok();
            }
        });
        writeln!(bt, "").ok();
        true
    });
    String::from_utf8_lossy(&bt).to_string()
}
