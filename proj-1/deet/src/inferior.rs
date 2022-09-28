use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use nix::unistd::Pid;
use std::process::Child;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::mem::size_of;

#[derive(Debug)]
pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

const INT_CODE:u8 = 0xcc as u8;

#[derive(Debug)]
pub struct Inferior {
    child: Child,
}

fn _align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}

impl Inferior {
    fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = _align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        ptrace::write(
            self.pid(),
            aligned_addr as ptrace::AddressType,
            updated_word as *mut std::ffi::c_void,
        )?;
        Ok(orig_byte as u8)
    }

    pub fn append_breakpoint(&mut self, addr:usize) -> Result<u8, nix::Error>{
        return self.write_byte(addr,INT_CODE);
    }
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>, breakpoints:&Vec<usize>) -> Option<Inferior> {
        let mut _binding = Command::new(target);
        let cmd = _binding.args(args);
        unsafe { cmd.pre_exec(child_traceme);}
        let child = cmd.spawn().ok()?;
        let mut res = Inferior{child};
        if res.wait(Some(WaitPidFlag::WSTOPPED)).is_ok(){
            for (i,brk) in breakpoints.iter().enumerate(){
                if let Err(_) = Inferior::write_byte(&mut res, *brk, INT_CODE){
                    println!("Error when injecting breakpoint {}",i);
                    return None;
                }
            }
            Some(res)
        }else {
            None
        }
    }

    pub fn cont(&mut self) -> Result<Status, nix::Error>{
        // continue
        ptrace::cont(self.pid(), None)?;
        self.wait(None)
    }

    pub fn kill(&mut self) {
        self.child.kill().unwrap();
        let _wait_res = self.wait(None).unwrap(); // SIGKILL
        println!("Killing running inferior (pid {})", self.pid());
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    pub fn backtrace(&self, debug_data:&DwarfData) {
        let regs = ptrace::getregs(self.pid()).unwrap();
        let mut rip = regs.rip as usize;
        let mut rbp = regs.rbp as usize;
        loop{
            // 1. print function/line number for instruction_ptr
            let rip_line = match debug_data.get_line_from_addr(rip){
                Some(v) => v,
                None => {
                    println!("there is no call stack yet");
                    continue;
                }
            };
            let rip_func = match debug_data.get_function_from_addr(rip){
                Some(v) => v,
                None => {
                    println!("there is no call stack yet");
                    continue;
                }
            };
            println!("{} ({:?}:{})", rip_func, rip_line.file, rip_line.number);
            // 2. compare and quit
            if rip_func == "main"{
                break
            }
            rip = ptrace::read(self.pid(), (rbp + 8) as ptrace::AddressType).unwrap() as usize;
            rbp = ptrace::read(self.pid(), rbp as ptrace::AddressType).unwrap() as usize;
        }
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }
}
