use primitive_types::U256;

use crate::{
    evm::{
        utils::{flip_sign, is_negative},
        EVM,
    },
    utils::{logger::Logger, types::NextAction},
};

// 0x1b
pub fn shl(evm: &mut EVM) -> NextAction {
    let shift = evm.stack.pop().unwrap();
    let val = evm.stack.pop().unwrap();
    evm.stack.push(val << shift);

    NextAction::Continue
}

// 0x1c
pub fn shr(evm: &mut EVM) -> NextAction {
    let shift = evm.stack.pop().unwrap();
    let val = evm.stack.pop().unwrap();
    evm.stack.push(val >> shift);

    NextAction::Continue
}

// 0x1d
/// This does not work if the number is small and shift is greater or equal 8
pub fn sar(evm: &mut EVM) -> NextAction {
    let shift = evm.stack.pop().unwrap();
    let mut val = evm.stack.pop().unwrap();

    if is_negative(&val) {
        flip_sign(&mut val);
        let mut result = val >> shift;
        flip_sign(&mut result);
        evm.stack.push(result);
    } else {
        evm.stack.push(val >> shift);
    }

    NextAction::Continue
}

// 0x1a
pub fn byte(evm: &mut EVM) -> NextAction {
    let offset = evm.stack.pop().unwrap();
    let val = evm.stack.pop().unwrap();

    if offset >= U256::from(32) {
        EVM::warning("byte offset greater or equal than 32");
        evm.stack.push(U256::zero());
        return NextAction::Continue;
    }

    let byte_offset = 31 - usize::from(offset.byte(0));

    let result = val.byte(byte_offset);
    evm.stack.push(U256::from(result));

    NextAction::Continue
}

// 0x5a
/// This is not supported yet, it returns `U256::MAX`
pub fn gas(evm: &mut EVM) -> NextAction {
    evm.stack.push(U256::MAX);
    NextAction::Continue
}

// 0xfe
pub fn invalid(_evm: &mut EVM) -> NextAction {
    NextAction::Exit(1)
}