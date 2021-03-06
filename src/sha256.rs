use db_key::Key;
use openssl::crypto::hash;
use std::fmt;

use rustc_serialize::hex::{ToHex, FromHex};

use PKey;

#[derive(Clone)]
pub struct Sha256{
    bits : [u8;32]
}

impl Key for Sha256 {
    fn from_u8(k: &[u8]) -> Self {
        assert!(k.len() == 32);
        Sha256{bits:
               // FIXME: This is dumb.
               [ k[0], k[1], k[2], k[3], k[4], k[5], k[6], k[7], k[8], k[9], k[10], k[11], k[12], k[13], k[14], k[15], k[16], k[17], k[18], k[19], k[20], k[21], k[22], k[23], k[24], k[25], k[26], k[27], k[28], k[29], k[30], k[31] ]
        }
    }
    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(&self.bits)
    }
}

impl Sha256 {
    pub fn of_bytes(b:&[u8]) -> Self {
        let v = hash::hash(hash::Type::SHA256, b);
        Sha256::from_u8(&v[0..32])
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.bits
    }
    pub fn from_hex(s: &str) -> Self {
        match s.from_hex() {
            Ok(v) => Sha256::from_u8(&v),
            Err(e) => panic!("Parse error from hex: {}", e)
        }
    }
    pub fn pkey_fingerprint(k: &PKey) -> Self {
        // This is consistent with `openssl rsa -in foo.pem  -pubout -outform DER | sha256sum -`
        Sha256::of_bytes(&k.save_pub())
    }
}

impl PartialEq<Sha256> for Sha256 {
    fn eq(&self, other: &Sha256) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl Eq for Sha256 {}

impl ToHex for Sha256 {
    fn to_hex(&self) -> String {
        self.bits.to_hex()
    }
}

impl fmt::Display for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(f.write_str(&"Sha256:"));
        f.write_str(&self.to_hex())
    }

}

impl fmt::Debug for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
   }
}
