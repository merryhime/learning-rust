use std::io;

fn main() {
    println!("Input guess.");

    let mut guess = String::new();

    io::stdin().read_line(&mut guess).expect("Fail to read line");

    println!("Guess: {}", guess);
}
