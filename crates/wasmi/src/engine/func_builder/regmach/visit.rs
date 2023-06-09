#![allow(unused_variables)]

use super::{stack::Provider, FuncTranslator};
use crate::engine::{
    bytecode2::{BinInstr, BinInstrImm16, Const16, Const32, Instruction, Register, UnaryInstr},
    TranslationError,
};
use wasmi_core::{UntypedValue, ValueType};
use wasmparser::VisitOperator;

macro_rules! impl_visit_operator {
    ( @mvp $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @sign_extension $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @saturating_float_to_int $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @bulk_memory $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @reference_types $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @tail_call $($rest:tt)* ) => {
        impl_visit_operator!(@@skipped $($rest)*);
    };
    ( @@skipped $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // We skip Wasm operators that we already implement manually.
        impl_visit_operator!($($rest)*);
    };
    ( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident $($rest:tt)* ) => {
        // Wildcard match arm for all the other (yet) unsupported Wasm proposals.
        fn $visit(&mut self $($(, $arg: $argty)*)?) -> Self::Output {
            self.unsupported_operator(stringify!($op))
        }
        impl_visit_operator!($($rest)*);
    };
    () => {};
}

impl FuncTranslator<'_> {
    /// Called when translating an unsupported Wasm operator.
    ///
    /// # Note
    ///
    /// We panic instead of returning an error because unsupported Wasm
    /// errors should have been filtered out by the validation procedure
    /// already, therefore encountering an unsupported Wasm operator
    /// in the function translation procedure can be considered a bug.
    fn unsupported_operator(&self, name: &str) -> Result<(), TranslationError> {
        panic!("tried to translate an unsupported Wasm operator: {name}")
    }
}

impl<'a> VisitOperator<'a> for FuncTranslator<'a> {
    type Output = Result<(), TranslationError>;

    wasmparser::for_each_operator!(impl_visit_operator);

    fn visit_unreachable(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_nop(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_block(&mut self, blockty: wasmparser::BlockType) -> Self::Output {
        todo!()
    }

    fn visit_loop(&mut self, blockty: wasmparser::BlockType) -> Self::Output {
        todo!()
    }

    fn visit_if(&mut self, blockty: wasmparser::BlockType) -> Self::Output {
        todo!()
    }

    fn visit_else(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_end(&mut self) -> Self::Output {
        self.visit_return()
    }

    fn visit_br(&mut self, relative_depth: u32) -> Self::Output {
        todo!()
    }

    fn visit_br_if(&mut self, relative_depth: u32) -> Self::Output {
        todo!()
    }

    fn visit_br_table(&mut self, targets: wasmparser::BrTable<'a>) -> Self::Output {
        todo!()
    }

    fn visit_return(&mut self) -> Self::Output {
        let instr = match self.func_type().results() {
            [] => {
                // Case: Function returns nothing therefore all return statements must return nothing.
                Instruction::Return
            }
            [ValueType::I32] => match self.alloc.stack.pop() {
                // Case: Function returns a single `i32` value which allows for special operator.
                Provider::Register(value) => Instruction::ReturnReg { value },
                Provider::Const(value) => Instruction::ReturnImm32 {
                    value: Const32::from_i32(i32::from(value)),
                },
            },
            [ValueType::I64] => match self.alloc.stack.pop() {
                // Case: Function returns a single `i64` value which allows for special operator.
                Provider::Register(value) => Instruction::ReturnReg { value },
                Provider::Const(value) => {
                    if let Ok(value) = Const32::try_from(i64::from(value)) {
                        Instruction::ReturnI64Imm32 { value }
                    } else {
                        Instruction::ReturnImm {
                            value: self.engine().alloc_const(value)?,
                        }
                    }
                }
            },
            [ValueType::F32] => {
                todo!()
            }
            [ValueType::F64] => {
                todo!()
            }
            _ => todo!(),
        };
        self.alloc.instr_encoder.push_instr(instr)?;
        Ok(())
    }

    fn visit_call(&mut self, function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_call_indirect(
        &mut self,
        type_index: u32,
        table_index: u32,
        table_byte: u8,
    ) -> Self::Output {
        todo!()
    }

    fn visit_return_call(&mut self, function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_return_call_indirect(&mut self, type_index: u32, table_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_drop(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_select(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_typed_select(&mut self, ty: wasmparser::ValType) -> Self::Output {
        todo!()
    }

    fn visit_local_get(&mut self, local_index: u32) -> Self::Output {
        self.alloc.stack.push_local(local_index)?;
        Ok(())
    }

    fn visit_local_set(&mut self, local_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_local_tee(&mut self, local_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_global_get(&mut self, global_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_global_set(&mut self, global_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f32_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f64_load(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load8_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load8_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load16_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load16_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load32_s(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_load32_u(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f32_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_f64_store(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i32_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store8(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store16(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_i64_store32(&mut self, memarg: wasmparser::MemArg) -> Self::Output {
        todo!()
    }

    fn visit_memory_size(&mut self, mem: u32, mem_byte: u8) -> Self::Output {
        todo!()
    }

    fn visit_memory_grow(&mut self, mem: u32, mem_byte: u8) -> Self::Output {
        todo!()
    }

    fn visit_i32_const(&mut self, value: i32) -> Self::Output {
        self.alloc.stack.push_const(value);
        Ok(())
    }

    fn visit_i64_const(&mut self, value: i64) -> Self::Output {
        self.alloc.stack.push_const(value);
        Ok(())
    }

    fn visit_f32_const(&mut self, value: wasmparser::Ieee32) -> Self::Output {
        todo!()
    }

    fn visit_f64_const(&mut self, value: wasmparser::Ieee64) -> Self::Output {
        todo!()
    }

    fn visit_ref_null(&mut self, ty: wasmparser::ValType) -> Self::Output {
        todo!()
    }

    fn visit_ref_is_null(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_ref_func(&mut self, function_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_i32_eqz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_lt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_lt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_gt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_gt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_le_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_le_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ge_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ge_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_eqz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_lt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_lt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_gt_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_gt_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_le_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_le_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ge_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ge_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_lt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_gt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_le(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_ge(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_eq(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_ne(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_lt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_gt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_le(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_ge(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_clz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_ctz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_popcnt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_add(&mut self) -> Self::Output {
        self.translate_binary_commutative_i32(
            Instruction::i32_add,
            Instruction::i32_add_imm,
            Instruction::i32_add_imm16,
            UntypedValue::i32_add,
            Self::no_custom_opt,
            |this, reg: Register, value: i32| {
                if value == 0 {
                    // Optimization: `add x + 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative_i32(
            Instruction::i32_mul,
            Instruction::i32_mul_imm,
            Instruction::i32_mul_imm16,
            UntypedValue::i32_mul,
            Self::no_custom_opt,
            |this, reg: Register, value: i32| {
                if value == 0 {
                    // Optimization: `add x * 0` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i32));
                    return Ok(true);
                }
                if value == 1 {
                    // Optimization: `add x * 1` is always `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_div_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_div_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rem_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rem_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_and(&mut self) -> Self::Output {
        self.translate_binary_commutative_i32(
            Instruction::i32_and,
            Instruction::i32_and_imm,
            Instruction::i32_and_imm16,
            UntypedValue::i32_and,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x & x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i32| {
                if value == -1 {
                    // Optimization: `x & -1` is same as `x`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x & 0` is same as `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i32));
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_or(&mut self) -> Self::Output {
        self.translate_binary_commutative_i32(
            Instruction::i32_or,
            Instruction::i32_or_imm,
            Instruction::i32_or_imm16,
            UntypedValue::i32_or,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x | x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i32| {
                if value == -1 {
                    // Optimization: `x | -1` is same as `-1`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_const(UntypedValue::from(-1_i32));
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x | 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_xor(&mut self) -> Self::Output {
        self.translate_binary_commutative_i32(
            Instruction::i32_xor,
            Instruction::i32_xor_imm,
            Instruction::i32_xor_imm16,
            UntypedValue::i32_xor,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x ^ x` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i32));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i32| {
                if value == 0 {
                    // Optimization: `x ^ 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_shl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_shr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_shr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rotl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_rotr(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_clz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_ctz(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_popcnt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_add(&mut self) -> Self::Output {
        self.translate_binary_commutative_i64(
            Instruction::i64_add,
            Instruction::i64_add_imm,
            Instruction::i64_add_imm16,
            UntypedValue::i64_add,
            Self::no_custom_opt,
            |this, reg: Register, value: i64| {
                if value == 0 {
                    // Optimization: `add x + 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_div_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_div_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rem_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rem_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_and(&mut self) -> Self::Output {
        self.translate_binary_commutative_i64(
            Instruction::i64_and,
            Instruction::i64_and_imm,
            Instruction::i64_and_imm16,
            UntypedValue::i64_and,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x & x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i64| {
                if value == -1 {
                    // Optimization: `x & -1` is same as `x`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x & 0` is same as `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i64));
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_or(&mut self) -> Self::Output {
        self.translate_binary_commutative_i64(
            Instruction::i64_or,
            Instruction::i64_or_imm,
            Instruction::i64_or_imm16,
            UntypedValue::i64_or,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x | x` is always just `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i64| {
                if value == -1 {
                    // Optimization: `x | -1` is same as `-1`
                    //
                    // Note: This is due to the fact that -1
                    // in twos-complements only contains 1 bits.
                    this.alloc.stack.push_const(UntypedValue::from(-1_i64));
                    return Ok(true);
                }
                if value == 0 {
                    // Optimization: `x | 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_xor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_shl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_shr_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_shr_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rotl(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_rotr(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_ceil(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_floor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_trunc(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_nearest(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_sqrt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_div(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_min(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_max(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_copysign(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_abs(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_neg(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_ceil(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_floor(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_trunc(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_nearest(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_sqrt(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_add(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_sub(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_mul(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_div(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_min(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_max(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_copysign(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_wrap_i64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_f64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend_i32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend_i32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_f64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_convert_i64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_demote_f64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_convert_i64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_promote_f32(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_reinterpret_f32(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_reinterpret_f64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f32_reinterpret_i32(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_f64_reinterpret_i64(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_extend8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_extend16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend8_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend16_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_extend32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_sat_f32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_sat_f32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_sat_f64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i32_trunc_sat_f64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_sat_f32_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_sat_f32_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_sat_f64_s(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_i64_trunc_sat_f64_u(&mut self) -> Self::Output {
        todo!()
    }

    fn visit_memory_init(&mut self, data_index: u32, mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_data_drop(&mut self, data_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_copy(&mut self, dst_mem: u32, src_mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_memory_fill(&mut self, mem: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_init(&mut self, elem_index: u32, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_elem_drop(&mut self, elem_index: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_copy(&mut self, dst_table: u32, src_table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_fill(&mut self, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_get(&mut self, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_set(&mut self, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_grow(&mut self, table: u32) -> Self::Output {
        todo!()
    }

    fn visit_table_size(&mut self, table: u32) -> Self::Output {
        todo!()
    }
}
