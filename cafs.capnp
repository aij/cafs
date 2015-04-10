@0xb362985d30df7877;

struct Reference {
  # A reference to a raw block of data. 
  struct RawBlockRef {
    #union {
      sha256 @0 : Data; # Data must be 256 bits (32 bytes).
    #}
  }
  struct BlockRef {
    rawblock @0 : RawBlockRef;
    size @1 : UInt64; # Optional decoded size or 0.
    enc :union {
      none @2 :Void;    
      aes @3 :Data; # Data must be 256 bits (32 bytes).
    }
    
  }
  struct DataRef {
    union {
      block @0 :BlockRef;
      indirect @1 :BlockRef;
      inline @2 :Data;  # Small blocks of data can be inlined. Data smaller than BlockRef should be inlined. Data bigger than the block size must not be inlined.
    }
  }
  struct VolumeRef {
    publicKey @0 :Data;
    volumeId @1 :Data;
    minSerial @2 :Int64;
  }
  union {
    file @0 :DataRef;
    directory @1 :DataRef;
    volume @2 :VolumeRef;
  }
}

struct VolumeRoot {
  header @0 :Data; # Must parse as valid VolumeHeader.
  signature @1 :Data;
}

struct VolumeHeader {
  volumeId @0 :Data;
  serial @1 :Int64;
  contents :union {
    public @2 :Reference; # For a publicly readable volume.
    private @3 :PrivateRef;
  }
  struct PrivateRef {
    ref @0 :Data; # After decrypting, decodes to a Reference.
    keys @1 :List(Data); # Key to decrypt ref, encrypted with each of the public keys that should have access. TODO: Define the format more precisely.
  }
}

# TODO: Use a compact form that avoids repeating as much data.
struct IndirectBlock {
  subblocks @0 :List(Reference.DataRef);
}

struct Directory {
  files @0 :List(Entry);

  struct Entry {
    name @0 :Text;
    ref @1 :Reference;
  }
}
