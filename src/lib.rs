extern crate futures;
extern crate inotify;
extern crate mio;
extern crate tokio;

mod wrap;

pub use inotify::ffi::*;

pub use wrap::AsyncINotify;
