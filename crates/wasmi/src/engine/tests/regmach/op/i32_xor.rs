use super::*;

const WASM_OP: WasmOp = WasmOp::I32("xor");

#[test]
fn same_reg() {
    let expected = [Instruction::ReturnImm32 {
        value: Const32::from_i32(0),
    }];
    test_binary_same_reg(WASM_OP, expected)
}

#[test]
fn reg_reg() {
    test_binary_reg_reg(WASM_OP, Instruction::i32_xor)
}

#[test]
fn reg_imm16() {
    test_binary_reg_imm16(WASM_OP, Instruction::i32_xor_imm16)
}

#[test]
fn reg_imm16_rev() {
    test_binary_reg_imm16_rev(WASM_OP, swap_ops!(Instruction::i32_xor_imm16))
}

#[test]
fn reg_imm() {
    test_binary_reg_imm32(WASM_OP, Instruction::i32_xor_imm)
}

#[test]
fn reg_imm_rev() {
    test_binary_reg_imm32_rev(WASM_OP, Instruction::i32_xor_imm)
}

#[test]
fn reg_zero() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_with(WASM_OP, 0i32, expected)
}

#[test]
fn reg_zero_rev() {
    let expected = [Instruction::ReturnReg {
        value: Register::from_u16(0),
    }];
    test_binary_reg_imm_rev_with(WASM_OP, 0i32, expected)
}

#[test]
fn consteval() {
    let lhs = 10;
    let rhs = 20;
    test_binary_consteval(
        WASM_OP,
        lhs,
        rhs,
        [Instruction::ReturnImm32 {
            value: Const32::from_i32(lhs ^ rhs),
        }],
    )
}