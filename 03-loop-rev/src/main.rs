fn main() {
    for i in (1..5).rev() {
        println!("{}", i);
    }
    println!("---");
    for i in (1..=5).rev() {
        println!("{}", i);
    }
}
