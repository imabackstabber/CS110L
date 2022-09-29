use crate::debugger_command::DebuggerCommand;
use crate::inferior::Inferior;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::inferior::Status;

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints: Vec<usize>,
}

fn _parse_address(addr: &str) -> Option<usize> {
    let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
        &addr[2..]
    } else {
        &addr
    };
    usize::from_str_radix(addr_without_0x, 16).ok()
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);
        // Print debuging data
        debug_data.print();

        Debugger {
            target: target.to_string(),
            breakpoints: vec![],
            history_path,
            readline,
            inferior: None,
            debug_data,
        }
    }

    pub fn reset(&mut self){
        self.inferior = None;
        self.breakpoints = vec![];
    }

    pub fn match_res(&mut self, res: Result<Status,nix::Error>){
        match res {
            Ok(v) => {
                match v {
                    Status::Exited(_status_code) => {
                        self.reset(); // to make kill normal
                        println!("Child exited (status {})",_status_code);
                    }
                    Status::Stopped(_signal,_rip) => {
                        println!("Child stopped (signal {:?})",_signal);
                        if let Some(rip_line) = self.debug_data.get_line_from_addr(_rip){
                            println!("Stopped at {}:{}",rip_line.file, rip_line.number);
                        }
                    }
                    _ => {
                        println!("Child send unknown information");
                    }
                }
            }
            Err(e) => {
                println!("Child continue makes error:{:?}", e);
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if self.inferior.is_some(){
                        //1. first kill n reap it
                        self.inferior
                            .as_mut()
                            .unwrap()
                            .kill();
                        //2. finally clean it
                        self.reset();
                    }
                    if let Some(inferior) = Inferior::new(&self.target, &args, &self.breakpoints) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // TODO (milestone 1): make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        let cont_res = self.inferior.as_mut().unwrap().cont();
                        self.match_res(cont_res);
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Quit => {
                    if self.inferior.is_some(){
                        //1. first kill n reap it
                        self.inferior
                            .as_mut()
                            .unwrap()
                            .kill();
                        //2. finally clean it
                        self.reset();
                    }
                    return;
                }
                DebuggerCommand::Cont => {
                    // 1. check whether process is running
                    if let None = self.inferior{
                        println!("Err: no process is running yet");
                    } else{
                        // 2. resume the child
                        let my_continue_res = self.inferior.as_mut().unwrap().cont();
                        self.match_res(my_continue_res);
                    }
                }
                DebuggerCommand::Back => {
                    if let None = self.inferior{
                        println!("Err: no process is running yet");
                    } else {
                        self.inferior.as_mut().unwrap().backtrace(&self.debug_data);
                    }
                }
                DebuggerCommand::BreakPoint(args) => {
                    if args.len() > 1{
                        println!("<usage>: b/break *addr/symbol");
                    } else{
                        let addr = &args[0];
                        if addr.starts_with("*"){
                            if let Some(parse_res) = _parse_address(&addr[1..]){
                                self.add_breakpint(parse_res);
                            }
                        } else{
                            let addr = &args[0];
                            // 1. if it's function name
                            if let Some(parse_res) = self.debug_data.get_addr_for_function(None, addr){
                                self.add_breakpint(parse_res);
                            } else {
                                // 2. if it can be a line number
                                match addr.parse::<usize>().ok(){
                                    Some(addr_u) => {
                                        if let Some(parse_res) = self.debug_data.get_addr_for_line(None, addr_u){
                                            self.add_breakpint(parse_res);
                                        } else{
                                            println!("fail to parse addr {} as function or usize",&args[0]);
                                        }
                                    }
                                    None => {
                                        println!("fail to parse addr {} as function or usize",&args[0]);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn add_breakpint(&mut self, parse_res:usize) {
        println!("Set breakpoint {} at {:#x}",self.breakpoints.len(),parse_res);
        self.breakpoints.push(parse_res);
        // should we check the breakpoint already in the vector?
        if self.inferior.is_some(){
            if let Err(_) = self.inferior.as_mut().unwrap().append_breakpoint(parse_res){
                println!("Add breakpoint failed, clean INT at {:#x}",parse_res);
                self.breakpoints.pop();
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
