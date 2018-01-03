extern crate chrono;
extern crate syscall;

use syscall::data::Packet;
use syscall::scheme::SchemeMut;
use std::fs::File;
use std::io::{Read, Write};
use scheme::LogScheme;

mod scheme;

fn main() {
    if unsafe { syscall::clone(0).unwrap() } == 0 {
        let mut socket = File::create(":log").expect("logd: failed to create log scheme");
        let mut scheme = LogScheme::new();

        syscall::setrens(0, 0).expect("logd: failed to enter null namespace");

        loop {
            let mut packet = Packet::default();
            socket.read(&mut packet).expect("logd: failed to read events from log scheme");

            scheme.handle(&mut packet);

            socket.write(&packet).expect("logd: failed to write responses to log scheme");
        }
    }
}
