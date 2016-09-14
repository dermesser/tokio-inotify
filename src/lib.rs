extern crate futures;
extern crate inotify;
extern crate mio;
extern crate tokio_core;

mod wrap;

pub use inotify::ffi::*;

pub use wrap::AsyncINotify;
