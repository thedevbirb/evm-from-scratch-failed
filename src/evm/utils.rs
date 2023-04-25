use crate::{
    evm::{opcodes, EVM},
    utils::{
        logger::Logger,
        types::{NextAction, Opcode, Opcodes},
    },
};
use primitive_types::U256;
use std::collections::HashMap;

use super::constants::JUMPDEST;

/// Flips the sign of a number using two's complement
pub fn flip_sign(num: &mut U256) {
    *num = !*num + 1;
}

/// Check if the given number is negative according to
/// its binary representation, looking at the MSB
pub fn is_negative(num: &U256) -> bool {
    num.bit(255)
}

fn push_n(evm: &mut EVM, n: u8) -> NextAction {
    let mut str = String::new();
    for _i in 1..=n {
        let byte = evm.execution_bytecode.get(evm.pc).expect("Missing data");
        if byte <= &u8::from(15) {
            str.push_str(&format!("0{:x}", byte));
        } else {
            str.push_str(&format!("{:x}", byte));
        }
        evm.pc += 1;
    }
    let num = U256::from_str_radix(&str, 16).unwrap();
    evm.stack.push(num);

    NextAction::Continue
}

fn generate_push_n_fn(n: u8) -> Opcode {
    if n > 32 {
        panic!("ERROR: arg must be a number between 0 and 32 included")
    }

    Box::new(move |evm: &mut EVM| push_n(evm, n))
}

fn dup_n(evm: &mut EVM, n: u8) -> NextAction {
    let mut temp_stack: Vec<U256> = Vec::with_capacity(usize::from(n));

    // pop until we find the value to duplicate
    for _i in 1..n {
        let val = evm.stack.pop().unwrap();
        temp_stack.push(val);
    }

    let val_to_dup = evm.stack.pop().unwrap();
    evm.stack.push(val_to_dup.clone());

    // fill the stack back
    for _i in 1..n {
        let val = temp_stack.pop().unwrap();
        evm.stack.push(val);
    }

    evm.stack.push(val_to_dup);

    NextAction::Continue
}

fn generate_dup_n_fn(n: u8) -> Opcode {
    if n == 0 || n > 16 {
        EVM::error("Invalid N value for DUP N: it must be between 0 and 16 excluded");
        panic!();
    }

    Box::new(move |evm: &mut EVM| dup_n(evm, n))
}

/// todo this is not good yet
fn swap_n(evm: &mut EVM, n: u8) -> NextAction {
    let mut temp_stack: Vec<U256> = Vec::with_capacity(usize::from(n));
    let first_val = evm.stack.pop().unwrap();

    // pop until we find the value to swap
    for _i in 1..n {
        let val = evm.stack.pop().unwrap();
        temp_stack.push(val);
    }

    let last_val = evm.stack.pop().unwrap();

    evm.stack.push(first_val);

    // fill the stack back
    for _i in 1..n {
        let val = temp_stack.pop().unwrap();
        evm.stack.push(val);
    }

    evm.stack.push(last_val);

    NextAction::Continue
}

fn generate_swap_n_fn(n: u8) -> Opcode {
    if n == 0 || n > 16 {
        EVM::error("Invalid N value for SWAP N: it must be between 0 and 16 excluded");
        panic!();
    }

    Box::new(move |evm: &mut EVM| swap_n(evm, n))
}

// It'd better to have them static. See PHF crate.
pub fn get_opcodes() -> Opcodes {
    let mut opcodes: Opcodes = HashMap::new();

    opcodes.insert(0x00, Box::new(opcodes::stop));
    opcodes.insert(0x01, Box::new(opcodes::arithmetic::add));
    opcodes.insert(0x02, Box::new(opcodes::arithmetic::mul));
    opcodes.insert(0x03, Box::new(opcodes::arithmetic::sub));
    opcodes.insert(0x04, Box::new(opcodes::arithmetic::div));
    opcodes.insert(0x05, Box::new(opcodes::arithmetic::s_div));
    opcodes.insert(0x06, Box::new(opcodes::arithmetic::modulo));
    opcodes.insert(0x07, Box::new(opcodes::arithmetic::s_modulo));
    opcodes.insert(0x08, Box::new(opcodes::arithmetic::add_mod));
    opcodes.insert(0x09, Box::new(opcodes::arithmetic::mul_mod));

    opcodes.insert(0x10, Box::new(opcodes::logic::lt));
    opcodes.insert(0x11, Box::new(opcodes::logic::gt));
    opcodes.insert(0x12, Box::new(opcodes::logic::slt));
    opcodes.insert(0x13, Box::new(opcodes::logic::sgt));
    opcodes.insert(0x14, Box::new(opcodes::logic::eq));
    opcodes.insert(0x15, Box::new(opcodes::logic::is_zero));
    opcodes.insert(0x16, Box::new(opcodes::logic::and));
    opcodes.insert(0x17, Box::new(opcodes::logic::or));
    opcodes.insert(0x18, Box::new(opcodes::logic::xor));
    opcodes.insert(0x19, Box::new(opcodes::logic::not));

    opcodes.insert(0x1b, Box::new(opcodes::misc::shl));
    opcodes.insert(0x1c, Box::new(opcodes::misc::shr));
    opcodes.insert(0x1d, Box::new(opcodes::misc::sar));
    opcodes.insert(0x1a, Box::new(opcodes::misc::byte));

    opcodes.insert(0x0a, Box::new(opcodes::arithmetic::exp));
    // opcodes.insert(0x0b, Box::new(opcodes::sign_extend));
    opcodes.insert(0x50, Box::new(opcodes::stack::pop));
    opcodes.insert(0x51, Box::new(opcodes::memory::mload));
    opcodes.insert(0x52, Box::new(opcodes::memory::mstore));
    opcodes.insert(0x53, Box::new(opcodes::memory::mstore8));
    opcodes.insert(0x56, Box::new(opcodes::stack::jump));
    opcodes.insert(0x57, Box::new(opcodes::stack::jumpi));
    opcodes.insert(0x58, Box::new(opcodes::stack::pc));
    opcodes.insert(0x5a, Box::new(opcodes::misc::gas));
    opcodes.insert(0x5b, Box::new(opcodes::stack::jumpdest));

    insert_push_n_functions(&mut opcodes);
    insert_dup_n_functions(&mut opcodes);
    insert_swap_n_functions(&mut opcodes);

    opcodes.insert(0xfe, Box::new(opcodes::misc::invalid));

    opcodes
}

fn insert_push_n_functions(opcodes: &mut Opcodes) {
    for n in 1..=32 {
        opcodes.insert(0x5f + n, generate_push_n_fn(n));
    }
}

fn insert_dup_n_functions(opcodes: &mut Opcodes) {
    for n in 1..=16 {
        opcodes.insert(0x7f + n, generate_dup_n_fn(n));
    }
}

fn insert_swap_n_functions(opcodes: &mut Opcodes) {
    for n in 1..=16 {
        opcodes.insert(0x8f + n, generate_swap_n_fn(n));
    }
}

/// Reads the `execution_bytecode` and returns a vector with all
/// the indexes in which a jumpdest occurs. This vector naturally is sorted.
///
/// Computational cost: O(n), where `n` is the length of the bytecode
pub fn get_jumpdests(execution_bytecode: &Vec<u8>) -> Vec<usize> {
    let mut pc = 0;
    let mut jumpdests: Vec<usize> = Vec::new();

    while pc < execution_bytecode.len() {
        let opcode = execution_bytecode.get(pc).unwrap();

        if &0x60 <= opcode && opcode <= &0x7f {
            let offset = opcode - 0x60 + 1;
            pc += usize::from(offset);
        } else if opcode == &JUMPDEST {
            jumpdests.push(pc);
        }

        pc += 1;
    }

    jumpdests
}

pub fn is_pc_on_jumpdest(evm: &EVM) -> bool {
    if evm.jumpdests.binary_search(&evm.pc).is_ok() {
        true
    } else {
        false
    }
}