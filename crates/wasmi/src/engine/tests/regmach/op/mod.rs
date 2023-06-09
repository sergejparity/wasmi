mod i32_add;
mod i32_and;
mod i32_mul;
mod i32_or;
mod i32_xor;

use super::{
    test_binary_consteval,
    test_binary_reg_imm,
    test_binary_reg_imm16,
    test_binary_reg_imm16_rev,
    test_binary_reg_imm_rev,
    test_binary_reg_imm_rev_with,
    test_binary_reg_imm_with,
    test_binary_reg_reg,
    test_binary_same_reg,
    Const32,
    Instruction,
    Register,
};
