use crate::open_file::OpenFile;
// #[allow(unused)] // TODO: delete this line for Milestone 3
use std::fs;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Process {
    pub pid: usize,
    pub ppid: usize,
    pub command: String,
}

impl Process {
    // #[allow(unused)] // TODO: delete this line for Milestone 1
    pub fn new(pid: usize, ppid: usize, command: String) -> Process {
        Process { pid, ppid, command }
    }

    /// This function returns a list of file descriptor numbers for this Process, if that
    /// information is available (it will return None if the information is unavailable). The
    /// information will commonly be unavailable if the process has exited. (Zombie processes
    /// still have a pid, but their resources have already been freed, including the file
    /// descriptor table.)
    // #[allow(unused)] // TODO: delete this line for Milestone 3
    pub fn list_fds(&self) -> Option<Vec<usize>> {
        // TODO: implement for Milestone 3
        // unimplemented!();
        let proc_path = format!("/proc/{}/fd", self.pid);
        let possible_dirres = fs::read_dir(proc_path).ok();
        let dirres = match possible_dirres {
            Some(_dir) => _dir,
            None => {
                return None
            }
        };
        let mut names =
            dirres.filter_map(|entry| {
                entry.ok().and_then(|e|
                    e.path().file_name()
                    .and_then(|n| n.to_str().map(|s| String::from(s)))
                )
            }).filter_map(|entry| entry.parse::<usize>().ok()).collect::<Vec<usize>>();
        names.sort();
        Some(names)
    }

    /// This function returns a list of (fdnumber, OpenFile) tuples, if file descriptor
    /// information is available (it returns None otherwise). The information is commonly
    /// unavailable if the process has already exited.
    // #[allow(unused)] // TODO: delete this line for Milestone 4
    pub fn list_open_files(&self) -> Option<Vec<(usize, OpenFile)>> {
        let mut open_files = vec![];
        for fd in self.list_fds()? {
            open_files.push((fd, OpenFile::from_fd(self.pid, fd)?));
        }
        Some(open_files)
    }
}

impl fmt::Display for Process{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "========== \"{}\" (pid {}, ppid {}) ==========\n",&self.command,self.pid,self.ppid)?;
        // write!(f, "{:?}",self.list_fds())
        match self.list_open_files() {
            None => println!(
                "Warning: could not inspect file descriptors for this process! \
                    It might have exited just as we were about to look at its fd table, \
                    or it might have exited a while ago and is waiting for the parent \
                    to reap it."
            ),
            Some(open_files) => {
                for (fd, file) in open_files {
                    println!(
                        "{:<4} {:<15} cursor: {:<4} {}",
                        fd,
                        format!("({})", file.access_mode),
                        file.cursor,
                        file.colorized_name(),
                    );
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ps_utils;
    use std::process::{Child, Command};

    fn start_c_program(program: &str) -> Child {
        Command::new(program)
            .spawn()
            .expect(&format!("Could not find {}. Have you run make?", program))
    }

    #[test]
    fn test_list_fds() {
        let mut test_subprocess = start_c_program("./multi_pipe_test");
        let process = ps_utils::get_target("multi_pipe_test").unwrap().unwrap();
        assert_eq!(
            process
                .list_fds()
                .expect("Expected list_fds to find file descriptors, but it returned None"),
            vec![0, 1, 2, 4, 5]
        );
        let _ = test_subprocess.kill();
    }

    #[test]
    fn test_list_fds_zombie() {
        let mut test_subprocess = start_c_program("./nothing");
        let process = ps_utils::get_target("nothing").unwrap().unwrap();
        assert!(
            process.list_fds().is_none(),
            "Expected list_fds to return None for a zombie process"
        );
        let _ = test_subprocess.kill();
    }
}
