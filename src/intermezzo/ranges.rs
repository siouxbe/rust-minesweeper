fn even(i: &i32) -> bool {
    0 == i % 2
}

fn square(i: i32) -> i32 {
    i * i
}

fn main() {
    for i in (0..6).filter(even).map(square) {
        println!("{i}");
    }
}
