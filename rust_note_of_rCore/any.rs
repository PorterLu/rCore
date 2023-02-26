fn main() {
    let a = vec![1, 2, 3];
    if a.iter().any(|&i| i == 3) { println!("y"); }
}
