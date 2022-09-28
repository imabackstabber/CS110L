### Proj-1
#### MileStone 2
##### how to reap a zombie process
查看了这位的[代码](https://github.com/PKUFlyingPig/CS110L/blob/main/proj-1/deet/src/inferior.rs),发现得和wait的实现一样，用``wait(None)``去捕获SIGKILL的信号（SIGNAL）

#### MileStone 5
严格的转换，&usize的类型不可以隐式转换为usize
&str[idx..]形式的切片返回的是str
对了，我还有一个问题，难道就直接insert 0xcc到代码里面么？这样不会造成对代码的破坏？