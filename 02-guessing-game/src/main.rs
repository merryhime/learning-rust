use std::io;
use rand::Rng;
use std::cmp::Ordering;

fn main() {
    println!("Input guess.");

    let secret = rand::thread_rng().gen_range(1..101);
    // println!("Secret: {}", secret);

    let mut guess = String::new();
    io::stdin().read_line(&mut guess).expect("Fail to read line");
    let guess: u32 = guess.trim().parse().expect("Expected number");

    println!("Guess: {}", guess);

    match guess.cmp(&secret) {
        Ordering::Less => println!("Too Small"),
        Ordering::Greater => println!("Too Big"),
        Ordering::Equal => println!("You Win"),
    }

}
