use std::cmp;

pub enum OpCode {
    PopTop,
    BinaryAdd,
    BinarySubtract,
    BinaryMultiply,
    BinaryTrueDivide,
    ReturnValue,
    LoadConst(u8),
    LoadName(u8),
    CallFunction(u8),
}

impl OpCode {
    fn get_value(&self) -> u8 {
        match *self {
            OpCode::PopTop => 1,
            OpCode::BinaryMultiply => 20,
            OpCode::BinaryAdd => 23,
            OpCode::BinarySubtract => 24,
            OpCode::BinaryTrueDivide => 27,
            OpCode::ReturnValue => 83,
            OpCode::LoadConst(_) => 100,
            OpCode::LoadName(_) => 101,
            OpCode::CallFunction(_) => 131
        }
    }

    pub fn to_bytes(&self) -> (u8, u8) {
        let operand = match *self {
            OpCode::LoadConst(v) |
            OpCode::LoadName(v) |
            OpCode::CallFunction(v) => v,
            _ => 0
        };
        return (self.get_value(), operand);
    }

    pub fn stack_effect(&self) -> i32 {
        match *self {
            OpCode::PopTop => -1,
            OpCode::BinaryAdd |
            OpCode::BinaryMultiply |
            OpCode::BinarySubtract |
            OpCode::BinaryTrueDivide => -1,
            OpCode::ReturnValue => -1,
            OpCode::LoadConst(_) |
            OpCode::LoadName(_) => 1,
            OpCode::CallFunction(n) => -(n as i32)
        }
    }
}

pub fn compile_code(operation_list: &[OpCode]) -> Vec<u8> {
    let code_size = operation_list.len() * 2;
    let mut result = vec![0u8; code_size];
    let mut i = 0;
    for op in operation_list {
        let (opcode, operand) = op.to_bytes();
        result[i] = opcode;
        result[i+1] = operand;
        i += 2;
    }
    result
}

pub fn calc_stack_size(operation_list: &[OpCode]) -> i32 {
    let mut max_size = 0;
    let mut current_size = 0;
    for op in operation_list {
        current_size += op.stack_effect();
        max_size = cmp::max(max_size, current_size);
    }
    max_size
}