pub enum DebuggerCommand {
    Quit,
    Cont,
    Back,
    BreakPoint(Vec<String>),
    Run(Vec<String>),
}

impl DebuggerCommand {
    pub fn from_tokens(tokens: &Vec<&str>) -> Option<DebuggerCommand> {
        match tokens[0] {
            "q" | "quit" => Some(DebuggerCommand::Quit),
            "c" | "cont" | "continue" => Some(DebuggerCommand::Cont),
            "bt" | "back" | "backtrace" => Some(DebuggerCommand::Back),
            "b" | "break" => {
                // do more parsing
                let args = tokens[1..].to_vec();
                Some(DebuggerCommand::BreakPoint(
                    args.iter().map(|s| s.to_string()).collect(),
                ))
            }
            "r" | "run" => {
                let args = tokens[1..].to_vec();
                Some(DebuggerCommand::Run(
                    args.iter().map(|s| s.to_string()).collect(),
                ))
            }
            // Default case:
            _ => None,
        }
    }
}
