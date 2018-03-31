extern crate futures;
extern crate tokio_core;
extern crate tokio_inotify;

use std::path::Path;
use std::env;

use futures::stream::Stream;
use futures::prelude::*;
use tokio_inotify::AsyncINotify;
use tokio_core::reactor::Core;

fn home_dir() -> String {
    env::var("HOME").unwrap_or_else(|_| {
        env::var("USER")
            .map(|u| {
                let mut d = "/home/".to_string();
                d.push_str(&u);
                d
            })
            .unwrap()
    })
}

fn main() {
    let mut evloop = Core::new().unwrap();
    let handle = evloop.handle();

    let inot = AsyncINotify::init(&evloop.handle()).unwrap();
    inot.add_watch(
        Path::new(&home_dir()),
        tokio_inotify::IN_CREATE | tokio_inotify::IN_DELETE,
    ).unwrap();

    let show_events = inot.for_each(|ev| {
        handle.spawn(futures::future::poll_fn(move || {
            if ev.is_create() {
                println!("created {}", ev.name);
            } else if ev.is_delete() {
                println!("deleted {}", ev.name);
            }
            Ok(Async::Ready(()))
        }));
        Ok(())
    });

    evloop.run(show_events).unwrap();
}
