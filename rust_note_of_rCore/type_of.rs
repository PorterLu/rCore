fn type_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
    // std::intrinsics::type_name::<T>()
}

fn main() {
    let mut a = 1;
    let mut b = 2;
    let mut c = &mut a;
    println!("a ownership return a == {}", a);
    c = &mut b;
    *c = 3;
    println!("{}", type_of(b)); 
}
