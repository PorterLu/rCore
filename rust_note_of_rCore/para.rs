#![feature(core_intrinsics)]
fn print_type_of<T>(_: T) {
    println!("{}", unsafe { std::intrinsics::type_name::<T>() });
}

fn main() {
    let os = Some(String::from("s"));
    match os {
      Some(ref s) => {
        print_type_of(s);
      },
      _ => ()
    };
    println!("{:?}", os);
}

