fn main() {
    let a = [0; 3];
    let b = &a;
    println!("{:?} {}", a.as_ptr(), a.len());

}
