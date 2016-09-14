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
    pub fn init(handle: &Handle) -> io::Result<AsyncINotify> {
        AsyncINotify::init_with_flags(handle, 0)
    }

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

    pub fn add_watch(&self, path: &Path, mask: u32) -> io::Result<Watch> {
        self.inner.add_watch(path, mask)
    }

    pub fn rm_watch(&self, watch: Watch) -> io::Result<()> {
        self.inner.rm_watch(watch)
    }

    pub fn close(self) -> io::Result<()> {
        self.inner.close()
    }

    fn available_events(&mut self) -> io::Result<&[Event]> {
        self.inner.available_events()
    }
}

impl Stream for AsyncINotify {
    type Item = Event;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // BUG-ish: This returns cached events in a reversed order. Usually, that shouldn't be a
        // problem though.
        if self.cached_events.len() > 0 {
            return Ok(Async::Ready(self.cached_events.pop()));
        }

        // the inner fd is non-blocking by default (set in the inotify crate)
        let events = try!(self.inner.available_events());

        // Only do vec operations if there are many events
        if events.len() < 1 {
            Ok(Async::NotReady)
        } else if events.len() == 1 {
            Ok(Async::Ready(Some(events[0].clone())))
        } else {
            // events.len() > 1
            self.cached_events.extend_from_slice(&events[1..]);
            Ok(Async::Ready(Some(events[0].clone())))
        }
    }
}
