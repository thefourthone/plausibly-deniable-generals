extern crate messages;

use messages::*;

fn main() {
    println!("{:?}", parse_message(vec![1, 7, 9, 5, 5, 255, 1, 9, 6]));
    println!(
        "{:?}",
        parse_message(vec![0x41, 0x5, 0x48, 0x65, 0x6c, 0x6c, 0x6f])
    );
}
