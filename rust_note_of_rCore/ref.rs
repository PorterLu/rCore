fn type_of<T>(_: T) -> &'static str{
    std::any::type_name::<T>()
}

fn main() {
    let a = 1;
    let ref b = a;
    println!("{}", type_of(*b));
}
