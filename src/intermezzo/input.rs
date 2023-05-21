#[allow(dead_code)]
#[allow(clippy::ptr_arg)]
fn copy_odd(src: &Vec<i32>, dst: &mut Vec<i32>) {
    let odd = src.iter().filter(|n| *n & 0x1 != 0);
    dst.extend(odd);
}

fn main() {
    /*
        let mut v = vec![1, 2, 3, 4, 5];
        copy_odd(&v, &mut v);
        for n in &v {
            println!("{n}");
        }
    */
}
