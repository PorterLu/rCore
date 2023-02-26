use std::mem;

pub struct List {
    head: Link, 
}

type Link = Option<Box<Node>>;

pub struct Node {
    elem: i32,
    next: Link,
}

impl List {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&self, elem: &Node) {

        
        let next = self.head;
        //se:q
        //lf.head = Some(new_node);
    }
}

fn main() {
    
}

