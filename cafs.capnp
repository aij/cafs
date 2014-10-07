

struct EncKey {
  union {
    none @0 :Void;
    aes @1 :Data;
  }
}


struct Reference {
  struct BlockRef {
    block @0 :BlockName;
    key @1 :EncKey;
  }
  struct DataRef {
    union {
      block @0 :BlockRef;
      indirect @1 :BlockRef;
      inline @2 :Data;  // Small blocks of data can be inlined. Data smaller than BlockRef should be inlined. Data bigger than the block size must not be inlined.
    }
  }
  union {
    file @0 :DataRef;
    directory @1 :DataRef;
    volume @2 :DataRef;
  }
}