pub mod constants;
pub mod opcodes;
pub mod utils;

use self::utils::{get_jumpdests, get_opcodes};
use crate::utils::{
    logger::Logger,
    types::{EvmResult, ExecutionData, NextAction},
};
use primitive_types::U256;

#[derive(Debug)]
pub struct EVM {
    pub jumpdests: Vec<usize>,
    pub memory: Vec<u8>,
    pub msize: usize,
    pub pc: usize,
    pub stack: Vec<U256>,
}

impl<'a> Logger<'a> for EVM {
    const NAMESPACE: &'a str = "EVM";
}

impl EVM {
    pub fn new() -> EVM {
        let evm = EVM {
            stack: Vec::with_capacity(1024),
            // TODO: size may change; zero is a special value
            // which does not require copy
            memory: vec![0; 256],
            pc: 0,
            msize: 0,
            jumpdests: Vec::new(),
        };

        evm
    }

    pub fn execute(self: &mut Self, data: ExecutionData) -> EvmResult {
        let opcodes = get_opcodes();

        self.jumpdests = get_jumpdests(data.bytecode);
        self.memory = vec![0; 256];
        self.msize = 0;

        let mut success = true;

        while self.pc < data.bytecode.len() {
            let opcode_num = data.bytecode.get(self.pc).expect("Could not read bytecode");

            self.pc += 1;

            let opcode = opcodes.get(&opcode_num).expect(&format!(
                "Could not find function associated to opcode_num {:x}",
                opcode_num
            ));

            let next_action = opcode(self, &data);

            match next_action {
                NextAction::Exit(status_code) => {
                    success = status_code == 0;
                    EVM::warning(&format!("Exiting with status code {}", status_code));
                    break;
                }
                _ => {}
            }
        }

        let result = self.get_result(success);
        self.reset();

        result
    }

    fn reset(&mut self) -> () {
        self.pc = 0;
        self.stack.clear();
    }

    fn get_result(&self, success: bool) -> EvmResult {
        let mut clone = self.stack.clone();
        clone.reverse();

        EvmResult {
            stack: clone,
            success,
        }
    }
}
