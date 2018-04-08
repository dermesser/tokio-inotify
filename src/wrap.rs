#![allow(dead_code)]

use inotify::wrapper::{Event, INotify, Watch};
use futures::{Async, Poll};
use futures::stream::Stream;
use mio::event::Evented;
use mio::{Poll as MioPoll, PollOpt, Ready, Token};
use mio::unix::EventedFd;
use tokio::reactor::PollEvented2 as PollEvented;

use std::io;
use std::os::unix::io::RawFd;
use std::path::Path;

struct NonLifetimedEventedFd(RawFd);

impl Evented for NonLifetimedEventedFd {
    fn register(
        &self,
        poll: &MioPoll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        let evfd = EventedFd(&self.0);
        evfd.register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &MioPoll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        let evfd = EventedFd(&self.0);
        evfd.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &MioPoll) -> io::Result<()> {
        let evfd = EventedFd(&self.0);
        evfd.deregister(poll)
    }
}

/// Wraps an INotify object and provides asynchronous methods based on the inner object.
pub struct AsyncINotify {
    inner: INotify,
    io: PollEvented<NonLifetimedEventedFd>,

    cached_events: Vec<Event>,
}

impl AsyncINotify {
    /// Create a new inotify stream on the loop behind `handle`.
    pub fn init() -> io::Result<AsyncINotify> {
        AsyncINotify::init_with_flags(0)
    }

    /// Create a new inotify stream with the given inotify flags (`IN_NONBLOCK` or `IN_CLOEXEC`).
    pub fn init_with_flags(flags: i32) -> io::Result<AsyncINotify> {
        let inotify = try!(INotify::init_with_flags(flags));

        let pollev = PollEvented::new(NonLifetimedEventedFd(inotify.fd as RawFd));

        Ok(AsyncINotify {
            inner: inotify,
            io: pollev,
            cached_events: Vec::new(),
        })
    }

    /// Monitor `path` for the events in `mask`. For a list of events, see
    /// https://docs.rs/tokio-inotify/0.2.1/tokio_inotify/struct.AsyncINotify.html (items prefixed with
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
            // Only register interest once we're down to the last cached event.
            if self.cached_events.len() == 1 {
                self.io.clear_read_ready(Ready::readable())?;
            }
            return Ok(Async::Ready(self.cached_events.pop()));
        }

        match self.io.poll_read_ready(Ready::readable()) {
            Ok(Async::NotReady) => {
                self.io.clear_read_ready(Ready::readable())?;
                return Ok(Async::NotReady);
            }
            Ok(Async::Ready(_)) => (), // proceed
            Err(e) => return Err(e),
        }

        // the inner fd is non-blocking by default (set in the inotify crate)
        let events = try!(self.inner.available_events());

        // Only do vec operations if there are many events
        if events.len() < 1 {
            // If EWOULDBLOCK is returned, inotify returns an empty slice. Signal that we want
            // more.
            self.io.clear_read_ready(Ready::readable())?;
            Ok(Async::NotReady)
        } else if events.len() == 1 {
            self.io.clear_read_ready(Ready::readable())?;
            Ok(Async::Ready(Some(events[0].clone())))
        } else {
            // events.len() > 1
            self.cached_events.extend_from_slice(&events[1..]);
            Ok(Async::Ready(Some(events[0].clone())))
        }
    }
}
