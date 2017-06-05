* Fix issue with             `infoLog.set_len(512 - 1); // subtract 1 to skip the trailing null character` done only conditionally...
* c_string crate vs this:
    ```
    macro_rules! cstr {
  ($s:expr) => (
    concat!($s, "\0") as *const str as *const [c_char] as *const c_char
  );
}``` (from https://github.com/rust-lang/rfcs/issues/400)
