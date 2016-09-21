extern crate futures;
extern crate tokio_inotify;
extern crate tokio_core;

use std::path::Path;
use std::env;

use futures::stream::Stream;
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

    let inot = AsyncINotify::init(&evloop.handle()).unwrap();
    inot.add_watch(Path::new(&home_dir()),
                   tokio_inotify::IN_CREATE | tokio_inotify::IN_DELETE)
        .unwrap();

    let show_events = inot.for_each(|ev| {
        if ev.is_create() {
            println!("created {}", ev.name);
        } else if ev.is_delete() {
            println!("deleted {}", ev.name);
        }
        Ok(())
    });

    evloop.run(show_events).unwrap();
}
