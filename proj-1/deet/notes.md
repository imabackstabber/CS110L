### Proj-1
#### MileStone 2
##### how to reap a zombie process
查看了这位的[代码](https://github.com/PKUFlyingPig/CS110L/blob/main/proj-1/deet/src/inferior.rs),发现得和wait的实现一样，用``wait(None)``去捕获SIGKILL的信号（SIGNAL）

#### MileStone 5
严格的转换，&usize的类型不可以隐式转换为usize
&str[idx..]形式的切片返回的是str
对了，我还有一个问题，难道就直接insert 0xcc到代码里面么？这样不会造成对代码的破坏？

#### MileStone 6
解决了MileStone 5提出来的问题
在理解manual给出的指导的时候出现了一些困难，因为我并不能理解ptrace::step究竟做了什么
不过在阅读了一些代码之后，我理解它的作用应该是single step一次并且返回SIGTRAP信号。
具体的例子可以在这个[repo](https://github.com/oliver/ptrace-sampler/blob/master/ptrace-singlestep.C)中得到验证
```c++
while(true){
    // ...
    int sigNo = 0;
    const int pRes = ptrace(PTRACE_SINGLESTEP, pid, 0, sigNo);
    if (pRes < 0)
    {
        perror("singlestep error");
        exit(1);
    }

    waitRes = wait(&waitStat); // we know that ptrace::step will send a signal here
    sigNo = WSTOPSIG(waitStat); // converting to a specific kind of stop signal
    if (sigNo == SIGTRAP)
    {
        sigNo = 0;
    }
    else
    {
        printf("child got unexpected signal %d\n", sigNo);
        ptrace(PTRACE_CONT, pid, 0, sigNo);
        //exit(1);
        break;
    }
    // ...
}
```

#### Milestone 7
没什么好说的

#### WrapUp
代码写太丑了:
```rust
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
```
什么时候去优化一下逻辑结构吧