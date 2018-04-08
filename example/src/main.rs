extern crate futures;
extern crate tokio;
extern crate tokio_inotify;

use std::path::Path;
use std::env;

use futures::stream::Stream;
use futures::prelude::*;
use tokio_inotify::AsyncINotify;

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
    let inot = AsyncINotify::init().unwrap();
    inot.add_watch(
        Path::new(&home_dir()),
        tokio_inotify::IN_CREATE | tokio_inotify::IN_DELETE,
    ).unwrap();

    let show_events = inot.for_each(|ev| {
        tokio::spawn(futures::future::poll_fn(move || {
            if ev.is_create() {
                println!("created {:?}", ev.name);
            } else if ev.is_delete() {
                println!("deleted {:?}", ev.name);
            }
            Ok(Async::Ready(()))
        }));
        Ok(())
    }).map_err(|_| ());

    tokio::run(show_events);
}
