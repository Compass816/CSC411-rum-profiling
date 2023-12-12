use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::process;

//use crate::memory;
use crate::memory::Memory;


pub fn run(memory: &mut Memory) {
    // Takes an in-memory executable image
    // as specified by the UM spec, and executes it
    // It is a c.r.e. if an instruction word has
    // an invalid opcode (14 or 15).
    // let mut segmap = memory::Memory::new(program);
    // next, start calling decode() on each instruction
    // and dispatch it!
    let mut r = Registers::new();
    let mut pc = 0_u32;
    let mut inst_counter = 0_u64;
    loop {
        let instr_word = memory.get_instruction(pc);
        let instr = match Instruction::decode(instr_word) {
            Some(instr) => instr,
            None => panic!("illegal instruction"),
        };
        let op = instr.opcode;
        inst_counter += 1;
        pc += 1;

        let ra_val = r[instr.ra];
        let rb_val = r[instr.rb];
        let rc_val = r[instr.rc];

        match op {
            Opcode::CMov => {
                if rc_val != 0 {
                    r[instr.ra] = rb_val
                }
            }
            Opcode::Load => {
                r[instr.ra] = memory.load(rb_val, rc_val);
            }
            Opcode::Store => {
                memory.store(ra_val, rb_val, rc_val);
            }
            Opcode::Add => {
                r[instr.ra] = rb_val.wrapping_add(rc_val);
            }
            Opcode::Mul => {
                r[instr.ra] = rb_val.wrapping_mul(rc_val);
            }
            Opcode::Div => {
                r[instr.ra] = rb_val / rc_val;
            }
            Opcode::Nand => {
                r[instr.ra] = !(rb_val & rc_val);
            }
            Opcode::Halt => {
                eprintln!("{} instructions executed", inst_counter);
                process::exit(0);
            }
            Opcode::MapSegment => {
                r[instr.rb] = memory.allocate(rc_val);
            }
            Opcode::UnmapSegment => {
                memory.deallocate(rc_val);
            }
            Opcode::Output => {
                let value = rc_val as u8;
                stdout().write_all(&[value]).unwrap();
                stdout().flush().unwrap();
            }
            Opcode::Input => match stdin().bytes().next() {
                Some(value) => {
                    r[instr.rc] = value.unwrap() as u32;
                }
                None => r[instr.rc] = !0,
            },
            Opcode::LoadProgram => {
                if rb_val != 0 {
                    memory.load_segment(rb_val);
                }
                pc = rc_val;
                
            }
            Opcode::LoadValue => {
                r[instr.ra] = instr.value;
            }
        }
    }
}

#[derive(Debug, PartialEq)]
#[repr(u32)]
enum Opcode {
    CMov,
    Load,
    Store,
    Add,
    Mul,
    Div,
    Nand,
    Halt,
    MapSegment,
    UnmapSegment,
    Output,
    Input,
    LoadProgram,
    LoadValue,
}

pub fn boot(filename: &str) -> Vec<u32> {
    // Load a UM binary meeting the specification, and load it as a
    // sequence of 32-bit words in memory.
    let mut f = File::open(filename)
        .unwrap_or_else(|_| panic!("File not found: {}", filename));
    let mut contents = Vec::new();

    match f.read_to_end(&mut contents) {
        Ok(_bytes) => {
            let program: Vec<u32> = contents
                .chunks_exact(4)
                .map(|x| u32::from_be_bytes(x.try_into().unwrap()))
                .collect();
            program
        }
        Err(e) => {
            panic!("Encountered error while reading from {}: {}", filename, e)
        }
    }
}

// functions for instruction parsing.

#[derive(Debug)]
struct Instruction {
    opcode: Opcode,
    ra: u32,
    rb: u32,
    rc: u32,
    value: u32,
}

impl Instruction {
    fn decode(instruction: u32) -> Option<Instruction> {
        let opcode = match(instruction >> 28) & 0b1111 {
            0 => Opcode::CMov,
            1 => Opcode::Load,
            2 => Opcode::Store,
            3 => Opcode::Add,
            4 => Opcode::Mul,
            5 => Opcode::Div,
            6 => Opcode::Nand,
            7 => Opcode::Halt,
            8 => Opcode::MapSegment,
            9 => Opcode::UnmapSegment,
            10 => Opcode::Output,
            11 => Opcode::Input,
            12 => Opcode::LoadProgram,
            13 => Opcode::LoadValue,
            _ => return None,
        };

        let mut inst = Instruction { opcode, ra: 0, rb: 0, rc: 0, value: 0 };
        match inst.opcode {
            Opcode::LoadValue => {
                inst.ra = (instruction >> 25) & 0x7;
                inst.value = (instruction << 7) >> 7;
            }
            _ => {
                inst.ra = (instruction >> 6) & 0x7;
                inst.rb = (instruction >> 3) & 0x7;
                inst.rc = instruction & 0x7;
            }
        }
        Some(inst)
    }
}

// A wrapper for encapsulating register logic. Makes it easier to experiment
// with indexing (e.g., unchecked indexing).
#[derive(Debug)]
struct Registers([u32; 8]);

impl Registers {
    pub fn new() -> Registers {
        Registers([0; 8])
    }
}

impl std::ops::Index<u32> for Registers {
    type Output = u32;

    fn index(&self, i: u32) -> &u32 {
        &self.0[i as usize]
        // Actually does not seem to improve midmark.
        // This would probably be unsound anyway without verifying that
        // every index is correct. In practice, Instruction.{ra,rb,rc} is
        // always in the range [0, 8), so it's OK for this program.
        // unsafe { self.0.get_unchecked(i as usize) }
    }
}

impl std::ops::IndexMut<u32> for Registers {
    fn index_mut(&mut self, i: u32) -> &mut u32 {
        &mut self.0[i as usize]
        // Actually does not seem to improve midmark.
        // unsafe { self.0.get_unchecked_mut(i as usize) }
    }
}
