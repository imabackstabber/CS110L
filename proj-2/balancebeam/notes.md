### proj-2
在完成这个proj之前，需要了解异步编程模型的基础概念
线程是一种经典的异步编程模型，但是其有很多缺点，诸如难以避免死锁
future模型，抑或是被js称作promise，是另一种异步编程模型
我现在暂时把它作为event-driven来看的
伴随这个概念出现的还有executor，它用于在runtime中监控polling的返回情况
（顺带一说，poll是用于查看future是否完成的接口）
我的理解是executor充当了一个隐形的loop，正因如此，我们才可以异步执行io
不然我们一直polling的话也就没有异步这一说法了嘛！
#### Milestone 1
直接对于每一个新的connection使用一个新的thread
这里我们只需要确保regression test依然通过就可以了
这里由于tcplistener.incoming()只能让我们serially process connection
不需要care tokio会把线程开得巨多，manual上面说开出来的都是轻量级的
#### Milestone 2
实现1的时候就顺手实现了Milestone 2，注意到把`request.rs`和`response.rs`里面的函数
改写成async之后，也要把main里面的调用加上await，不然future不会工作（compiler yelled）
#### Milestone 3
涉及到加锁，需要明确的是这里可以使用读写锁，没有什么问题，但我并没有这样实现，我是直接用了Mutex（
需要注意的是我们仍然传一个Arc(shared_state_ref)，只不过我们需要获取的锁仅仅加在一个成员变量上
我们仍然可以通过获取这个锁，调用lock的lock()方法对成员变量进行改写
'这是因为我们从未改变锁本身以及其monitor的对象'，所以也不存在破坏了shared_state_ref的读状态一说法
#### Milestone 4
很明显，我们另外开一个线程，heartbeat
还需要明白主动检测的工作模式，是要先发一个请求给path，然后再读回response，根据response的内容来判断服务器是否down