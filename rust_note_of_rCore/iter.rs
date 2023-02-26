fn main() {
    let a = vec![1, 2, 3];
    if a.iter().any(|&v| v==4) {
        println!("Exist");
    } else {
        println!("Not Found");
    }

    println!("{}", a[0]);
}
