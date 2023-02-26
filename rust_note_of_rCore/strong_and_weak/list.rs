// linked_list.rs
use std::fmt::Debug;
use std::rc::Rc; 

#[derive(Debug)]
struct LinkedList<T: Debug> { 
    head: Option<Rc<Node<T>>> 
} 

#[derive(Debug)]
struct Node<T: Debug> { 
    data: T,
    next: Option<Rc<Node<T>>>, 
} 

impl<T> LinkedList<T> 
    where T: Debug
{ 
    fn new() -> Self { 
        LinkedList { head: None } 
    } 

    fn append(&self, data: T) -> Self { 
        LinkedList { 
            head: Some(Rc::new(Node { 
                data: data, 
                next: self.head.clone() 
            })) 
        } 
    } 
} 

fn main() { 
    let list_of_nums = LinkedList::new().append(1).append(2); 
    println!("nums: {:?}", list_of_nums); 

    let list_of_strs = LinkedList::new().append("foo").append("bar"); 
    println!("strs: {:?}", list_of_strs); 
}

