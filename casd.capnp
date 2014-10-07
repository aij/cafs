@0x82a1cee98cd20637;

enum HashAlg {
  sha256 @0;
  sha512 @1;
}

struct BlockName {
 alg @0 :HashAlg;
 hash @1 :Data;
}


interface Casd {
  size @0 (names:BlockName) -> (size: UInt64);
  read @1 (names:BlockName, startAt :UInt64 = 0, amount :UInt64 = 0xffffffffffffffff)
       -> (data: Data);
  write @2 (names:List(BlockName), data:Data);

}

interface CasdPrivileged extends(Casd) {
  struct BlockInfo {
    size @0 :UInt64;
    hashes @1 :List(BlockName);
  }
  list @0 () -> (blocks :List(BlockInfo));
}
