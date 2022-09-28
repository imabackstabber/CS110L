### Proj-1
#### MileStone 2
##### how to reap a zombie process
查看了这位的[代码](https://github.com/PKUFlyingPig/CS110L/blob/main/proj-1/deet/src/inferior.rs),发现得和wait的实现一样，用``wait(None)``去捕获SIGKILL的信号（SIGNAL）