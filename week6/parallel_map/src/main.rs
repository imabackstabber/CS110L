use crossbeam_channel;
use std::{thread, time};

fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    // TODO: implement parallel map!
    let orig_size = input_vec.len();
    for _ in 0..orig_size{
        output_vec.push(U::default());
    }
    let (sender, receiver):(crossbeam_channel::Sender<(_,_)>, crossbeam_channel::Receiver<(_,_)>) = crossbeam_channel::unbounded(); // T,U
    let (final_sender, final_receiver):(crossbeam_channel::Sender<(_,_)>, crossbeam_channel::Receiver<(_,_)>) = crossbeam_channel::unbounded(); // usize, U
    let mut threads = vec![];
    for _ in 0..num_threads{
        let receiver: crossbeam_channel::Receiver<((usize,T),F)> = receiver.clone();
        let final_sender: crossbeam_channel::Sender<(usize, U)> = final_sender.clone();
        threads.push(thread::spawn(move || {
            while let Ok(((idx, next_num),f)) = receiver.recv(){
                final_sender.send((idx, f(next_num))).expect("Tried writing [res] to channel, but there are no receivers!");
            }
            drop(final_sender); // time to close it
        }))
    }
    for idx in (0..orig_size).rev() {
        sender.send(((idx, input_vec.pop().unwrap()),f.clone())).expect("Tried writing [(num,f)] to channel, but there are no receivers!");
    }
    drop(sender);
    drop(final_sender);
    for thread in threads {
        thread.join().expect("Panic occurred in thread");
    }
    while let Ok((idx,res)) = final_receiver.recv(){
        output_vec[idx] = res;
    }
    output_vec
}

fn main() {
    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
