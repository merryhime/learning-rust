#[derive(Debug)]
struct Test(i32, i32);

fn main() {
    println!("Hello World!");

    // Formatted printing
    // std::fmt defines e.g. format!, print!, println!, eprint!, eprintln!
    // See also: https://doc.rust-lang.org/std/fmt/

    println!("addr=0x{:x}", 0x12345);

    println!("numbered arguments {0} and {1}, 1 = {1}, 0 = {0}", "A", "B");

    println!(
        "{subject} {object} {verb}",
        subject = "mulus",
        verb = "spectat",
        object = "silvam"
    );

    println!("{:?}", Test(23, 43));
}
