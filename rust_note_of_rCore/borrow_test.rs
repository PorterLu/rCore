fn test(a: &mut String) {
    let b = a;
    println!("{}", a);
    println!("{}", b);
}

fn main() {
    let mut a = "HELLO".to_string();
    test(&mut a);
}
