#![allow(dead_code)]

use inotify::wrapper::{INotify, Watch, Event};
use futures::{Async, Poll};
use futures::stream::Stream;
use mio::deprecated::unix::Io;
use tokio_core::reactor::{Handle, PollEvented};

use std::path::Path;
use std::io;

/// Wraps an INotify object and provides asynchronous methods based on the inner object.
pub struct AsyncINotify {
    inner: INotify,
    io: PollEvented<Io>,

    cached_events: Vec<Event>,
}

impl AsyncINotify {
    /// Create a new inotify stream on the loop behind `handle`.
    pub fn init(handle: &Handle) -> io::Result<AsyncINotify> {
        AsyncINotify::init_with_flags(handle, 0)
    }

    /// Create a new inotify stream with the given inotify flags (`IN_NONBLOCK` or `IN_CLOEXEC`).
    pub fn init_with_flags(handle: &Handle, flags: isize) -> io::Result<AsyncINotify> {
        let inotify = try!(INotify::init_with_flags(flags));
        let evfd = Io::from_raw_fd(inotify.fd);

        let pollev = try!(PollEvented::new(evfd, handle));

        Ok(AsyncINotify {
            inner: inotify,
            io: pollev,
            cached_events: Vec::new(),
        })
    }

    /// Monitor `path` for the events in `mask`. For a list of events, see
    /// https://dermesser.github.io/tokio-inotify/doc/inotify/ffi/index.html (items prefixed with
    /// "Event")
    pub fn add_watch(&self, path: &Path, mask: u32) -> io::Result<Watch> {
        self.inner.add_watch(path, mask)
    }

    /// Remove an element currently watched.
    pub fn rm_watch(&self, watch: Watch) -> io::Result<()> {
        self.inner.rm_watch(watch)
    }

    /// Close the underlying file descriptor and remove it from the event loop.
    pub fn close(self) -> io::Result<()> {
        // FD is removed from loop by PollEvented::drop()
        self.inner.close()
    }
}

impl Stream for AsyncINotify {
    type Item = Event;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // BUG-ish: This returns cached events in a reversed order. Usually, that shouldn't be a
        // problem though.
        if self.cached_events.len() > 0 {
            if self.cached_events.len() == 1 {
                self.io.need_read()
            }
            return Ok(Async::Ready(self.cached_events.pop()));
        }

        match self.io.poll_read() {
            Async::NotReady => {
                return Ok(Async::NotReady);
            }
            Async::Ready(_) => (), // proceed
        }

        // the inner fd is non-blocking by default (set in the inotify crate)
        let events = try!(self.inner.available_events());

        // Only do vec operations if there are many events
        if events.len() < 1 {
            // If EWOULDBLOCK is returned, inotify returns an empty slice. Signal that we want
            // more.
            self.io.need_read();
            Ok(Async::NotReady)
        } else if events.len() == 1 {
            self.io.need_read();
            Ok(Async::Ready(Some(events[0].clone())))
        } else {
            // events.len() > 1
            self.cached_events.extend_from_slice(&events[1..]);
            Ok(Async::Ready(Some(events[0].clone())))
        }
    }
}
