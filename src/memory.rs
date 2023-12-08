use std::collections::HashMap;
const PROGRAM_ADDRESS: u32 = 0;

#[derive(Debug)]
pub struct Memory {
    pool: Vec<u32>,
    heap: Vec<Vec<u32>>,
}

impl Memory {
    // create a new Memory, comprising a pool of reusable IDs
    // and a heap of UM words, populated with the instructions
    // as segment 0
    pub fn new(instructions: Vec<u32>) -> Memory {
        Memory { pool: vec![], heap: vec![instructions] }
    }

    // allocate and initalize (as all 0s) a memory segment.
    // returns the segment ID
    pub fn allocate(&mut self, size: u32) -> u32 {
        // can we reuse a previously unmapped segment id?
        match self.pool.pop() {
            None => {
                let x = self.heap.len() as u32;
                self.heap.push(vec![0; size as usize]);
                x
            }
            Some(address) => {
                assert!(
                    address < self.heap.len() as u32,
                    "invalid address in pool"
                );
                let segment = &mut self.heap[address as usize];
                segment.resize(size as usize, 0);
                address
            }
        }
    }

    // deallocate the memory at the given address.
    pub fn deallocate(&mut self, address: u32) {
        assert!(
            address < self.heap.len() as u32,
            "invalid address {}, cannot deallocate",
            address,
        );
        self.pool.push(address);
        let address = address as usize; // Convert address to usize
        self.heap[address].clear();
    }

    // supply contents of the memory at the given address if
    // initialized, panics otherwise.
    pub fn load(&self, seg_id: u32, address: u32) -> u32 {
        self.heap
            .get(seg_id as usize)
            .and_then(|segment| segment.get(address as usize).copied())
            .unwrap()
    }

    // get the instruction word corresponding to the given program counter
    // if it doesn't exist, then this panics
    // This may have high overhead...
    pub fn get_instruction(&self, pc: u32) -> u32 {
        // SAFETY: `heap` always has length at least 1 and PROGRAM_ADDRESS
        // is always == 0. This improves performance by about 10%.
        let segment = &self.heap[PROGRAM_ADDRESS as usize];
        segment[pc as usize]
    }

    // write a value into the given address of the given segment.
    pub fn store(&mut self, seg_id: u32, address: u32, value: u32) {
        let seg_id_usize = seg_id as usize; // Convert seg_id to usize
        let memory =
            self.heap.get_mut(seg_id_usize).expect("Memory was unallocated");
        memory[address as usize] = value;
    }

    // replace the program with the vector at the given address
    pub fn load_segment(&mut self, seg_id: u32) {
        let program = self
            .heap
            .get(seg_id as usize)
            .expect("Found no program at the given address")
            .clone();
        let dest = &mut self.heap[PROGRAM_ADDRESS as usize];
        *dest = program;
    }
}
