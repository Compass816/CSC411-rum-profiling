use rum::machine;
use std::env;
use rum::memory::Memory;

fn main() {
    let filename = env::args().nth(1).expect("Usage: rum progname");
    let instructions = rum::machine::boot(&filename);
    let mut memory = Memory::new(instructions);
    rum::machine::run(&mut memory);
}
