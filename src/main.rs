use std::{env, process};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

mod dir;

use dir::exec_fd_readdir;

// Returns: (rights_base, rights_inheriting)
pub unsafe fn fd_get_rights(fd: wasi::Fd) -> (wasi::Rights, wasi::Rights) {
    let fdstat = wasi::fd_fdstat_get(fd).expect("fd_fdstat_get failed");
    (fdstat.fs_rights_base, fdstat.fs_rights_inheriting)
}

fn open_scratch_directory(path: &str) -> Result<wasi::Fd, String> {
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
            if dst == path.as_bytes() {
                let (base, inherit) = fd_get_rights(i);
                return Ok(
                    wasi::path_open(i, 0, ".", wasi::OFLAGS_DIRECTORY, base, inherit, 0)
                        .expect("failed to open dir"),
                );
            }
        }

        Err(format!("failed to find scratch dir"))
    }
}

fn main() {
    let mut args = env::args();
    let prog = args.next().unwrap();
    let arg = if let Some(arg) = args.next() {
        arg
    } else {
        eprintln!("usage: {} <scratch directory>", prog);
        process::exit(1);
    };
    let dir_fd = match open_scratch_directory(&arg) {
        Ok(dir_fd) => dir_fd,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1)
        }
    };

    unsafe {
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
