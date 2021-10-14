use std::io;
use rand::Rng;
use std::cmp::Ordering;

fn main() {
    let secret = rand::thread_rng().gen_range(1..101);
    // println!("Secret: {}", secret);

    loop {
        println!("Input guess:");

        let mut guess = String::new();
        io::stdin().read_line(&mut guess).expect("Fail to read line");

        let guess: u32 = match guess.trim().parse() {
            Ok(n) => n,
            Err(e) => {
                println!("Err: {}", e);
                continue;
            }
        };

        match guess.cmp(&secret) {
            Ordering::Less => println!("Too Small"),
            Ordering::Greater => println!("Too Big"),
            Ordering::Equal => {
                println!("You Win");
                break;
            }
        }
    }

}
