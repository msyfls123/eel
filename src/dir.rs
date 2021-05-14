/// from: https://github.com/bytecodealliance/wasmtime/blob/main/crates/test-programs/wasi-tests/src/bin/fd_readdir.rs

use std::{mem, slice, str};

const BUF_LEN: usize = 256;

pub struct DirEntry {
    dirent: wasi::Dirent,
    pub name: String,
}

// Manually reading the output from fd_readdir is tedious and repetitive,
// so encapsulate it into an iterator
struct ReadDir<'a> {
    buf: &'a [u8],
}

impl<'a> ReadDir<'a> {
    fn from_slice(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

impl<'a> Iterator for ReadDir<'a> {
    type Item = DirEntry;

    fn next(&mut self) -> Option<DirEntry> {
        unsafe {
            if self.buf.len() < mem::size_of::<wasi::Dirent>() {
                return None;
            }

            // Read the data
            let dirent_ptr = self.buf.as_ptr() as *const wasi::Dirent;
            let dirent = dirent_ptr.read_unaligned();

            if self.buf.len() < mem::size_of::<wasi::Dirent>() + dirent.d_namlen as usize {
                return None;
            }

            let name_ptr = dirent_ptr.offset(1) as *const u8;
            // NOTE Linux syscall returns a NUL-terminated name, but WASI doesn't
            let namelen = dirent.d_namlen as usize;
            let slice = slice::from_raw_parts(name_ptr, namelen);
            let name = str::from_utf8(slice).expect("invalid utf8").to_owned();

            // Update the internal state
            let delta = mem::size_of_val(&dirent) + namelen;
            self.buf = &self.buf[delta..];

            DirEntry { dirent, name }.into()
        }
    }
}

/// Return the entries plus a bool indicating EOF.
pub unsafe fn exec_fd_readdir(fd: wasi::Fd, cookie: wasi::Dircookie) -> (Vec<DirEntry>, bool) {
    let mut buf: [u8; BUF_LEN] = [0; BUF_LEN];
    let bufused =
        wasi::fd_readdir(fd, buf.as_mut_ptr(), BUF_LEN, cookie).expect("failed fd_readdir");

    println!("buf {:?}", buf);
    assert!(bufused <= BUF_LEN);

    let sl = slice::from_raw_parts(buf.as_ptr(), bufused);
    let dirs: Vec<_> = ReadDir::from_slice(sl).collect();
    let eof = bufused < BUF_LEN;
    (dirs, eof)
}
