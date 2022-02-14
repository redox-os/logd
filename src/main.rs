extern crate chrono;
extern crate syscall;

use syscall::data::Packet;
use syscall::flag::CloneFlags;
use syscall::scheme::SchemeMut;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::io::{FromRawFd, RawFd};
use std::process;
use scheme::LogScheme;

mod scheme;

fn daemon(mut write: File) {
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

    write.write(&[0]).expect("logd: failed to write to pipe");

    let mut scheme = LogScheme::new(files);

    loop {
        let mut packet = Packet::default();
        if socket.read(&mut packet).expect("logd: failed to read events from log scheme") == 0 {
            break;
        }

        scheme.handle(&mut packet);

        socket.write(&packet).expect("logd: failed to write responses to log scheme");
    }
}

fn main() {
    let mut pipes = [0; 2];
    syscall::pipe2(&mut pipes, 0).expect("logd: failed to create pipe");

    let mut read = unsafe { File::from_raw_fd(pipes[0] as RawFd) };
    let write = unsafe { File::from_raw_fd(pipes[1] as RawFd) };

    let pid = unsafe { syscall::clone(CloneFlags::empty()).expect("logd: failed to fork") };
    if pid == 0 {
        drop(read);

        daemon(write);
    } else {
        drop(write);

        let mut res = [0];
        read.read(&mut res).expect("logd: failed to read from pipe");

        process::exit(res[0] as i32);
    }
}
