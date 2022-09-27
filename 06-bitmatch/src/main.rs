use bitmatch::bitmatch;

#[bitmatch]
pub fn unpack_instructions(bytes: std::vec::Vec<u8>) -> Option<[u32; 4]> {
    if let &[a, b, cd, e, fg, h, ij, k, l] = &bytes[..9] {
        let c = (cd & 0b11000000) >> 6;
        let d = cd & 0b00111111;
        let f = (fg & 0b11110000) >> 4;
        let g = fg & 0b00001111;
        let i = (ij & 0b11111100) >> 2;
        let j = ij & 0b00000011;
        let ins1: u32 = bitpack!("aaaaaaaabbbbbbbbcc");
        let ins2: u32 = bitpack!("ddddddeeeeeeeeffff");
        let ins3: u32 = bitpack!("gggghhhhhhhhiiiiii");
        let ins4: u32 = bitpack!("jjkkkkkkkkllllllll");
        Some([ins1, ins2, ins3, ins4])
    } else {
        None
    }
}

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
    println!(
        "{:?}\n",
        unpack_instructions(vec![0, 1, 2, 3, 4, 5, 6, 7, 8])
    );
}
