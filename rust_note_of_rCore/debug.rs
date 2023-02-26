use std::fmt::{self, Debug, Formatter};

//struct AContainer(pub usize);
//struct BContainer(pub usize);

/*
impl Debug for Container {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}
*/

macro_rules! impl_T {
    (for $(($t: ident, $output: literal)),+) => {
        $(  
            #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
            struct $t (pub usize);
            impl Debug for $t {
                fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                    f.write_fmt(format_args!(concat!($output,":{:#x}"), self.0))
                }
            }
        )*
    }
}

impl_T!(for (AContainer,"A"), (BContainer,"B"));


fn main() {
    let a = AContainer(0);
    let b = BContainer(1);
    println!("{:?}", a);
    println!("{:?}", b);
}
