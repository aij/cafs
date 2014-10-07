#!/usr/bin/env python3

import os, tempfile, hashlib

# Note: This is the start of a prototype version of CASD. I doubt
# Python would perform well enough for a serious implementation, so it
# should probably be rewritten in Go or Rust once it has stabilized.

default_hash = hashlib.sha256

class StoragePool:
    def __init__(self, db, perm, scratch):
        self.db_path = db
        self.perm = perm
        self.scratch = scratch

    def store_data(self, data, hashes):
        hash = default_hash(data)
        # TODO: Check if hash exists.
        # TODO: Compute other supported hashes.
        self.__write_data(data, hash.hexdigest())

    def __write_data(self, data, name):
        with tempfile.NamedTemporaryFile(dir=self.scratch) as tmp:
            tmp.file.write(data)
            tmp.file.close()
            os.rename(tmp.name, os.path.join(self.perm, name))
            # TODO: Handle errors

    def __filename(self, hash):
        return os.path.join(self.perm, hash.hexdigest())


    def size(self, hash):
        f = self.__filename(hash)
        try:
            return os.stat(f).st_size
        except FileNotFoundError:
            return 0

    def read_file(self, hash):
        return open(self.__filename(hash))
