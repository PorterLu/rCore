#![feature(core_intrinsics)]
fn print_type_of<T>(_:T) {
    println!("{}", unsafe { std::intrinsics::type_name::<T>()});
}

fn main() {
    let os = Some(String::from("s"));
    if let Some(ref s) = os {
        print_type_of(s);
    } else {
        ()
    }
    println!("{:?}", os);
}
