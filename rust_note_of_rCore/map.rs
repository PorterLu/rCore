fn main() {
    let a = vec![1, 2, 3];
    let b = &a;
    let c = b.into_iter().map(|a| a);
    println!("{:?}", c);
    println!("{:?}", a);
}
