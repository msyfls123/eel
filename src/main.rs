use std::{process};
use std::ffi::{CStr};

mod dir;

use dir::exec_fd_readdir;

// Returns: (rights_base, rights_inheriting)
pub unsafe fn fd_get_rights(fd: wasi::Fd) -> (wasi::Rights, wasi::Rights) {
    let fdstat = wasi::fd_fdstat_get(fd).expect("fd_fdstat_get failed");
    (fdstat.fs_rights_base, fdstat.fs_rights_inheriting)
}

/// why start with 3: https://stackoverflow.com/questions/36771266/what-is-the-use-of-fd-file-descriptor-in-node-js/36771339
fn open_scratch_directory(path: &str, target: &str) -> Result<wasi::Fd, String> {
    unsafe {
        for i in 3.. {
            let stat = match wasi::fd_prestat_get(i) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}, {:?}", i, e);
                    break
                },
            };
            if stat.tag != wasi::PREOPENTYPE_DIR {
                continue;
            }
            let mut dst = Vec::with_capacity(stat.u.dir.pr_name_len);
            if wasi::fd_prestat_dir_name(i, dst.as_mut_ptr(), dst.capacity()).is_err() {
                continue;
            }
            dst.set_len(stat.u.dir.pr_name_len);
            println!("{:?}: {:?}", String::from_utf8(dst.clone()), path);
            let dst_str = String::from_utf8(dst.clone()).unwrap();
            let dst_str = dst_str.trim_matches(char::from(0));
            if dst_str == path {
                let (base, inherit) = fd_get_rights(i);
                return Ok(
                    wasi::path_open(i, 0, target, wasi::OFLAGS_DIRECTORY, base, inherit, 0)
                        .expect("failed to open dir"),
                );
            }
        }

        Err(format!("failed to find scratch dir"))
    }
}

fn main() {
    // let mut args = env::args();
    // let prog = args.next().unwrap();
    // let arg = if let Some(arg) = args.next() {
    //     arg
    // } else {
    //     eprintln!("usage: {} <scratch directory>", prog);
    //     process::exit(1);
    // };
    unsafe {
        let (argc, buf_size) = wasi::args_sizes_get().unwrap();
        println!("{:?}", argc);
        let mut argv = Vec::with_capacity(argc);
        let mut argv_buf = vec![0; buf_size];
        wasi::args_get(argv.as_mut_ptr(), argv_buf.as_mut_ptr()).unwrap();
        argv.set_len(argc);
        let mut ret = Vec::with_capacity(argc);
        for ptr in argv {
            let s = CStr::from_ptr(ptr.cast());
            ret.push(s.to_str().unwrap());
        }
        println!("{:?}", ret);

        let dir_fd = match open_scratch_directory(&ret[1], &ret[2]) {
            Ok(dir_fd) => dir_fd,
            Err(err) => {
                eprintln!("{}", err);
                process::exit(1)
            }
        };
        println!("dir_fd: {}", dir_fd);

        // Check the behavior in an empty directory
        let (mut dirs, eof) = exec_fd_readdir(dir_fd, 0);
        assert!(eof, "expected to read the entire directory");
        dirs.sort_by_key(|d| d.name.clone());
        for elem in dirs {
            println!("{}", elem.name);
        }
        // wasi::fd_write(stdout, &data).unwrap();
        // let stat = wasi::fd_fdstat_get(res.unwrap()).unwrap();
        // println!("{:?}", stat);
    }
}
