use linked_list::LinkedList;
pub mod linked_list;

fn main() {
    let mut list: LinkedList<u32> = LinkedList::new();
    assert!(list.is_empty());
    assert_eq!(list.get_size(), 0);
    for i in 1..12 {
        list.push_front(i);
    }
    println!("{}", list);
    println!("list size: {}", list.get_size());
    println!("top element: {}", list.pop_front().unwrap());
    println!("{}", list);
    println!("size: {}", list.get_size());
    println!("{}", list.to_string()); // ToString impl for anything impl Display

    let mut list2: LinkedList<u32> = LinkedList::new();
    assert_eq!(false, list == list2);
    for i in 1..11{
        list2.push_front(i);
    }
    assert_eq!(true, list == list2);

    let mut list3 = list.clone();
    assert_eq!(true, list == list3);
    list3.push_front(11);
    assert_eq!(false, list == list3);


    // If you implement iterator trait:
    //for val in &list {
    //    println!("{}", val);
    //}
}
