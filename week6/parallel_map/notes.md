### Week 6
首先，manual上说了不要从前面pop，而vec的pop方法就是pop_back
另外，根据Trait bound，我们需要在sender端传入f的clone
不然编译器会对我们不高兴(但现在不明白这个报错的含义)
```bash
`F` cannot be shared between threads safely
required because of the requirements on the impl of `Send` for `&F`
```
Q:目前的问题是怎么阻止closure拿走外部vec的所有权？ 
A:不要在closure里尝试使用vec，instead采用第二个channel发送函数结果
Q:然后是使得output vec能够排序？
A:Give it redundancy. 因为Manual里说你预先知道了input_vec的结果，所以你也知道output_vec的大小
那么我们提前填入U::default()，之后，sender发送origin num时也发送下标，and that's it！