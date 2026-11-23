# Stable storage

The task is to implement, in Rust, stable storage based on a filesystem
directory. The stable storage shall implement the `StableStorage` trait which
defines a key-value storage having the following properties:

* The `put()` method implements the `store` operation described above.
* The `get()` methods implements the `retrieve` operation described above.
* The `remove()` methods implements the `remove` operation described above.
* The storage supports keys that are at most 255 bytes long, and values that are
at most 65535 bytes long.
* An attempt to store an invalid key or an invalid input returns `Err` with an
error message of your choosing. The attempt does not result in a malfunction of
the storage.
* If a key has been successfully inserted or updated (i.e., the `put()` ended
with `Ok` or `remove` ended) before a crash, after the restart the storage
returns the most recently stored value for that key (or `None` if the most
recent operation for that key was `remove`). This includes crashes of the
operating system too.
* Values are stored atomically: either the whole value is stored under a key or
removed, or the value in not inserted/updated/removed at all.
* The `get()` method returns `None` for a key that has never been inserted.
* The `remove()` method returns `true` if the call actually removed a key and
its data and `false` otherwise.

Your implementation shall provide also the `build_stable_storage()` function,
which returns a trait object that implements the stable storage.

The execution time and number of accessed files for any stable storage operation
(including recovery) should be independent of the number of keys kept in the
storage (assuming that the time it takes to read, write or remove a file in a
directory is independent of the number of files in that directory).

If a filesystem operation fails (for example due to insufficient disk space)
your solution shall panic. It must not make the stable storage enter an invalid
state.

You can assume that the directory provided to the `build_stable_storage()`
function already exists and that it will be used exclusively by your stable
storage instance.

You can store all keys in memory, but not values.

You are allowed to create subdirectories within the provided directory. You are
not allowed to touch any other directories.

You can use the `sha2` and `base64` crates, as seen in
[`Cargo.toml`](Cargo.toml). If you use `sha2`, you can assume that there will be
no SHA-256 collisions.
