#[derive(Debug)]
pub struct Student{
    pub id:usize, 
    pub age:usize, 
    pub name:String
}

fn new() -> Student {
    let a = Student{id:0, age:1, name:"abc".to_string()};
    println!("{:?}", &a as *const Student);
    a
}

fn main(){
    let main_a = Student{id:1, age:2, name:"main_a".to_string()};
    let main_b = Student{id:2, age:2, name:"main_b".to_string()};
    let main_c = Student{id:3, age:2, name:"main_c".to_string()};
    let main_d = Student{id:4, age:2, name:"main_d".to_string()};
    let main_e = Student{id:5, age:2, name:"main_e".to_string()};
    let main_f = Student{id:6, age:2, name:"main_f".to_string()};

    println!("{:?}", &main_a as *const Student);
    println!("{:?}", &main_b as *const Student);
    println!("{:?}", &main_c as *const Student);
    println!("{:?}", &main_d as *const Student);
    println!("{:?}", &main_e as *const Student);
    println!("{:?}", &main_f as *const Student);
    println!("{:?}", &new() as *const Student);
    let main_g = Student{id:7, age:3, name:"main_g".to_string()};
    println!("{:?}", &main_g as *const Student);
    println!("{:?}", &main_a as *const Student);
}
