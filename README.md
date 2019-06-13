# tokio-inotify

**NOTE: The `inotify` crate now offers a `Stream`-based API. It is recommended to use it directly.**

[![crates.io](https://img.shields.io/crates/v/tokio-inotify.svg)](https://crates.io/crates/tokio-inotify)

[Documentation](https://docs.rs/tokio-inotify/)

The `tokio_inotify` crate enables the use of inotify file descriptors in the `tokio` framework.
It builds on the [`inotify`](https://github.com/hannobraun/inotify-rs) crate by wrapping
the `INotify` type into a new type called `AsyncINotify`, and implementing
[`futures::stream::Stream`](http://alexcrichton.com/futures-rs/futures/stream/trait.Stream.html).

This means that you can consume `inotify::Event`s from the `AsyncINotify` object and act on them.

