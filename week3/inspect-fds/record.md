### week3 on wsl(Vscode,Actually)
#### Milestone3
看起来wsl上的文件描述符和linux上有出入，这就折腾了我很久。

事实上是milestone3的任务，wsl(on fxxking Vscode)上输出的和预期的有所差池，大概是这样的：

> ---- process::test::test_list_fds stdout ----
> thread 'process::test::test_list_fds' panicked at 'assertion failed: `(left == right)`
>   left: `[0, 1, 2, 19, 20, 4, 5]`,
>  right: `[0, 1, 2, 4, 5]`', src/process.rs:81:9
> note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

wtf? 看起来是多出了两个文件符号，那么就按照[CS110L官方作业手册](https://reberhardt.com/cs110l/spring-2020/assignments/week-3-exercises/)上面的来试一下：

> * Add a `sleep(30)` call at the point in the program where you want to see the file descriptor table
> * Run the buggy program in one terminal window

之后就能检测到`./multi_pipe_test`的输出，注意可能会有两个同样的cmd（这是因为`fork()`），抓父子进程的fd出来看看(使用`ls /proc/<PID>/fd`):

```bash
$ ps -a
  PID TTY          TIME CMD
    8 tty1     00:00:00 sh
    9 tty1     00:00:00 sh
   34 tty1     00:00:00 sh
   38 tty1     00:00:26 node
   49 tty1     00:00:09 node
  315 tty1     00:02:35 node
  326 tty1     00:00:06 node
  375 tty1     00:04:11 rust-analyzer
  416 tty1     00:00:00 rust-analyzer
10999 pts/0    00:00:00 multi_pipe_test
11000 pts/0    00:00:00 multi_pipe_test
...
$ ls /proc/10999/fd/
0  1  19  2  20  4  5
$ ls /proc/11000/fd/
0  1  19  2  20  3  4  5  6
```

额，看起来wsl上面的期望输出确实是0 1 2 19 20 4 5(using vscode terminal)，那么看起来rust程序本身没什么问题。至于为什么多出来了这个19,20，这里卖个关子（因为笔者实在是没想到差别出在这上面）。

但是这子进程的fd多出来的3，6是怎么回事，看看下面这个c程序：

```c
#include <stdio.h>
#include <unistd.h>
#include <sys/wait.h>

int main() {
    int fds1[2];
    int fds2[2];
    pipe(fds1);
    pipe(fds2);
    pid_t pid = fork();
    if (pid == 0) {
        printf("%d %d\n",STDIN_FILENO, STDOUT_FILENO); // 0 1
        printf("%d %d\n",fds1[0], fds2[1]); // 3 6
        dup2(fds1[0], STDIN_FILENO);
        dup2(fds2[1], STDOUT_FILENO);
        close(fds1[0]);
        close(fds1[1]);
        close(fds2[0]);
        close(fds2[1]);
        sleep(2);
        return 0;
    }
    close(fds1[0]);
    close(fds2[1]);
    waitpid(pid, NULL, 0);
    return 0;
}
```

首先需要明白[pipe](https://man7.org/linux/man-pages/man2/pipe.2.html)做了什么：

> pipefd[0] refers to the read end of the pipe.  pipefd[1] refers to the write end of the pipe.

然后是[dup2](https://man7.org/linux/man-pages/man2/dup2.2.html):

> The dup() system call allocates a new file descriptor that refers to the **same** open file description as the descriptor oldfd.
>
> The dup2() system call performs the same task as dup(), but instead of using the lowest-numbered unused file descriptor, it uses the file descriptor number specified in **newfd**.

那么看起来3是子进程用来read的，6是子进程用来write的，因为你调用了dup2，这样子进程多出来3,6也就make sense了。
这里也要注意我们printf的时机，这个时候还没有close，所以和官方作业手册上给出来的输出会有所出入，如果我们是直接把``sleep(2)``改成``sleep(100)``然后再进行``ls proc/<PID>/fd``的话，那么结果会和官方手册一致.
当然到这里我们还没有说出19和20的来历，笔者进行了多次实验后发现，这是由使用vscode自带的terminal导致的，如果我们在wsl的shell里直接操作，就不会出现这种情况了（众人：？？？）。

#### Milestone4
``/proc/<PID>/fdinfo`` is not implented for wsl. :p
我直接在代码里面返回了fake result