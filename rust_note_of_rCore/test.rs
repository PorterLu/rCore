fn test(nums: Vec<i32>) -> Vec<i32> {
    nums.clone()
}

fn main() {
    let mut a = vec![1, 2, 3];
    let b: &mut Vec<i32> = &mut a;
    
    for i in b {
        println!("{}", i);
    }

    println!("{}", b[0]);
}
