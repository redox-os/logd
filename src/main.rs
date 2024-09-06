use redox_scheme::{RequestKind, SignalBehavior, Socket, V2};
use std::env;
use std::fs::OpenOptions;
use std::process;

use crate::scheme::LogScheme;

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

    let socket = Socket::<V2>::create("log").expect("logd: failed to create log scheme");

    libredox::call::setrens(0, 0).expect("logd: failed to enter null namespace");

    eprintln!("logd: ready for logging on log:");

    daemon.ready().expect("logd: failed to notify parent");

    let mut scheme = LogScheme::new(files);

    while let Some(request) = socket
        .next_request(SignalBehavior::Restart)
        .expect("logd: failed to read events from log scheme")
    {
        scheme.current_pid = request.context_id();

        let RequestKind::Call(request) = request.kind() else {
            continue;
        };

        let response = request.handle_scheme_mut(&mut scheme);
        socket
            .write_responses(&[response], SignalBehavior::Restart)
            .expect("logd: failed to write responses to log scheme");
    }
    process::exit(0);
}

fn main() {
    redox_daemon::Daemon::new(daemon).expect("logd: failed to daemonize");
}
