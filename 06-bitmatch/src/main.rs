use bitmatch::bitmatch;

#[bitmatch]
fn decode(inst: u8) -> String {
    #[bitmatch]
    match inst {
        "00oo_aabb" => format!("Op {}, {}, {}", o, a, b),
        "0100_aaii" => format!("Val {}, {}", a, i),
        "01??_????" => format!("Invalid"),
        "10ii_aabb" => format!("Ld {}, {}, {}", a, b, i),
        "11ii_aabb" => format!("St {}, {}, {}", a, b, i),
    }
}

fn main() {
    println!("{}\n", decode(0xf7));
}
