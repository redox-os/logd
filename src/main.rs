extern crate syscall;

use syscall::data::Packet;
use syscall::scheme::SchemeMut;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::process;
use scheme::LogScheme;

mod scheme;

fn daemon(daemon: redox_daemon::Daemon) -> ! {
    let mut files = Vec::new();
    for arg in env::args().skip(1) {
        eprintln!("logd: opening {:?}", arg);
        match OpenOptions::new().write(true).open(&arg) {
            Ok(file) => files.push(file),
            Err(err) => eprintln!("logd: failed to open {:?}: {:?}", arg, err),
        }
    }

    let mut socket = File::create(":log").expect("logd: failed to create log scheme");

    syscall::setrens(0, 0).expect("logd: failed to enter null namespace");

    eprintln!("logd: ready for logging on log:");

    daemon.ready().expect("logd: failed to notify parent");

    let mut scheme = LogScheme::new(files);

    loop {
        let mut packet = Packet::default();
        if socket.read(&mut packet).expect("logd: failed to read events from log scheme") == 0 {
            break;
        }

        scheme.current_pid = packet.pid;
        scheme.handle(&mut packet);

        socket.write(&packet).expect("logd: failed to write responses to log scheme");
    }
    process::exit(0);
}

fn main() {
    redox_daemon::Daemon::new(daemon).expect("logd: failed to daemonize");
}
