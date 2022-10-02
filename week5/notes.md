### Week5
这大概就是一个照搬过来的练习，什么也没有发生
熟悉了一下Arc 和 Mutex 的用法，仅此而已
#### 实验记录
我一开始写的版本不太正确，因为没有很好的利用好critical section
在MacBookAir上跑实验， 8 CPUS ，让程序分解个`2^6`数字结果如下
| 1 | 18.624010375s |
| 2 | 18.602548333s |
| 4 | 18.499501208s |
| 8 | 18.680017458s |
可以看到线程开多开少基本没什么变化，这就说明我们使用锁的方式不对
下面的方式是正确的，那就是把锁的生命周期用{}表示出来
一个trick就是采用一个函数封装这个过程
```rust
fn return_num_helper(remaining_nums: &Arc<Mutex<VecDeque<u32>>>) -> Option<u32>{
    let mut remaining_nums_ref = remaining_nums.lock().unwrap();
    remaining_nums_ref.pop_front()
}
```
之后的结果就自然合理了
| 1 | 18.732821041s |
| 2 | 9.749055166s |
| 4 | 5.272833541s |
| 8 | 4.596557958s |