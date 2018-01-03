use chrono::Local;
use std::collections::BTreeMap;
use std::io::{self, Write};
use syscall::error::*;
use syscall::scheme::SchemeMut;

pub struct LogHandle {
    context: Box<[u8]>,
    buf: Vec<u8>,
}

pub struct LogScheme {
    next_id: usize,
    handles: BTreeMap<usize, LogHandle>,
}

impl LogScheme {
    pub fn new() -> Self {
        LogScheme {
            next_id: 0,
            handles: BTreeMap::new(),
        }
    }
}

impl SchemeMut for LogScheme {
    fn open(&mut self, path: &[u8], _flags: usize, _uid: u32, _gid: u32) -> Result<usize> {
        let id = self.next_id;
        self.next_id += 1;

        self.handles.insert(id, LogHandle {
            context: path.to_vec().into_boxed_slice(),
            buf: Vec::new()
        });

        Ok(id)
    }

    fn dup(&mut self, _id: usize, buf: &[u8]) -> Result<usize> {
        if ! buf.is_empty() {
            return Err(Error::new(EINVAL));
        }

        Ok(0)
    }

    fn read(&mut self, id: usize, _buf: &mut [u8]) -> Result<usize> {
        let _handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        // TODO

        Ok(0)
    }

    fn write(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        let handle = self.handles.get_mut(&id).ok_or(Error::new(EBADF))?;

        let mut stdout = io::stdout();

        let mut i = 0;
        while i < buf.len() {
            let b = buf[i];

            if handle.buf.is_empty() {
                let timestamp = Local::now();
                let _ = write!(handle.buf, "{}", timestamp.format("%F %T%.f "));
                handle.buf.extend_from_slice(&handle.context);
                handle.buf.extend_from_slice(b": ");
            }

            handle.buf.push(b);

            if b == b'\n' {
                let _ = stdout.write(&handle.buf);
                let _ = stdout.flush();

                handle.buf.clear();
            }

            i += 1;
        }

        Ok(i)
    }

    fn fcntl(&mut self, id: usize, _cmd: usize, _arg: usize) -> Result<usize> {
        let _handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        Ok(0)
    }

    fn fpath(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        let scheme_path = b"log:";

        let mut i = 0;
        while i < buf.len() && i < scheme_path.len() {
            buf[i] = scheme_path[i];
            i += 1;
        }

        let mut j = 0;
        while i < buf.len() && j < handle.context.len() {
            buf[i] = handle.context[j];
            i += 1;
            j += 1;
        }

        Ok(i)
    }

    fn fsync(&mut self, id: usize) -> Result<usize> {
        let _handle = self.handles.get(&id).ok_or(Error::new(EBADF))?;

        Ok(0)
    }

    fn close(&mut self, id: usize) -> Result<usize> {
        self.handles.remove(&id).ok_or(Error::new(EBADF))?;

        Ok(0)
    }
}
