#![allow(unused_variables)]

use super::{bail_unreachable, stack::Provider, FuncTranslator};
use crate::engine::{
    bytecode2::{BinInstr, BinInstrImm16, Const16, Const32, Instruction, Register, UnaryInstr},
    TranslationError,
};
use wasmi_core::{TrapCode, UntypedValue, ValueType, F32, F64};
use wasmparser::VisitOperator;

/// Used to swap operands of a `rev` variant [`Instruction`] constructor.
macro_rules! swap_ops {
    ($fn_name:path) => {
        |result: Register, lhs: Const16, rhs: Register| -> Instruction {
            $fn_name(result, rhs, lhs)
        }
    };
}

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
        bail_unreachable!(self);
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
            [ValueType::F32] => match self.alloc.stack.pop() {
                // Case: Function returns a single `f32` value which allows for special operator.
                Provider::Register(value) => Instruction::ReturnReg { value },
                Provider::Const(value) => Instruction::ReturnImm32 {
                    value: Const32::from_f32(F32::from(value)),
                },
            },
            [ValueType::F64] => match self.alloc.stack.pop() {
                // Case: Function returns a single `f64` value which allows for special operator.
                Provider::Register(value) => Instruction::ReturnReg { value },
                Provider::Const(value) => Instruction::ReturnImm {
                    value: self.engine().alloc_const(value)?,
                },
            },
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
        self.alloc.stack.push_const(F32::from_bits(value.bits()));
        Ok(())
    }

    fn visit_f64_const(&mut self, value: wasmparser::Ieee64) -> Self::Output {
        self.alloc.stack.push_const(F64::from_bits(value.bits()));
        Ok(())
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
        bail_unreachable!(self);
        // Push a zero on the value stack so we can translate `i32.eqz` as `i32.eq(x, 0)`.
        self.alloc.stack.push_const(0_i32);
        self.visit_i32_eq()
    }

    fn visit_i32_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32>(
            Instruction::i32_eq,
            Instruction::i32_eq_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_eq_imm16,
            UntypedValue::i32_eq,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x == x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i32_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i32>(
            Instruction::i32_ne,
            Instruction::i32_ne_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_ne_imm16,
            UntypedValue::i32_ne,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x != x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i32_lt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_lt_s,
            Instruction::i32_lt_s_imm,
            Instruction::i32_gt_s_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_lt_s_imm16,
            swap_ops!(Instruction::i32_gt_s_imm16),
            UntypedValue::i32_lt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == i32::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, rhs: Register| {
                if lhs == i32::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_lt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_lt_u,
            Instruction::i32_lt_u_imm,
            Instruction::i32_gt_u_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_lt_u_imm16,
            swap_ops!(Instruction::i32_gt_u_imm16),
            UntypedValue::i32_lt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: u32| {
                if rhs == u32::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, rhs: Register| {
                if lhs == u32::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_gt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_gt_s,
            Instruction::i32_gt_s_imm,
            Instruction::i32_lt_s_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_gt_s_imm16,
            swap_ops!(Instruction::i32_lt_s_imm16),
            UntypedValue::i32_gt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == i32::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, rhs: Register| {
                if lhs == i32::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_gt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_gt_u,
            Instruction::i32_gt_u_imm,
            Instruction::i32_lt_u_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_gt_u_imm16,
            swap_ops!(Instruction::i32_lt_u_imm16),
            UntypedValue::i32_gt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: u32| {
                if rhs == u32::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, rhs: Register| {
                if lhs == u32::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_le_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_le_s,
            Instruction::i32_le_s_imm,
            Instruction::i32_ge_s_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_le_s_imm16,
            swap_ops!(Instruction::i32_ge_s_imm16),
            UntypedValue::i32_le_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == i32::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, rhs: Register| {
                if lhs == i32::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_le_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_le_u,
            Instruction::i32_le_u_imm,
            Instruction::i32_ge_u_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_le_u_imm16,
            swap_ops!(Instruction::i32_ge_u_imm16),
            UntypedValue::i32_le_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: u32| {
                if rhs == u32::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, rhs: Register| {
                if lhs == u32::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_ge_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_ge_s,
            Instruction::i32_ge_s_imm,
            Instruction::i32_le_s_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_ge_s_imm16,
            swap_ops!(Instruction::i32_le_s_imm16),
            UntypedValue::i32_ge_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == i32::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i32, rhs: Register| {
                if lhs == i32::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_ge_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i32_ge_u,
            Instruction::i32_ge_u_imm,
            Instruction::i32_le_u_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_ge_u_imm16,
            swap_ops!(Instruction::i32_le_u_imm16),
            UntypedValue::i32_ge_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: u32| {
                if rhs == u32::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u32, rhs: Register| {
                if lhs == u32::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_eqz(&mut self) -> Self::Output {
        bail_unreachable!(self);
        // Push a zero on the value stack so we can translate `i64.eqz` as `i64.eq(x, 0)`.
        self.alloc.stack.push_const(0_i64);
        self.visit_i64_eq()
    }

    fn visit_i64_eq(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64>(
            Instruction::i64_eq,
            Instruction::i64_eq_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_eq_imm16,
            UntypedValue::i64_eq,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x == x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i64_ne(&mut self) -> Self::Output {
        self.translate_binary_commutative::<i64>(
            Instruction::i64_ne,
            Instruction::i64_ne_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_ne_imm16,
            UntypedValue::i64_ne,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x != x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i64_lt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_lt_s,
            Instruction::i64_lt_s_imm,
            Instruction::i64_gt_s_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_lt_s_imm16,
            swap_ops!(Instruction::i64_gt_s_imm16),
            UntypedValue::i64_lt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == i64::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, rhs: Register| {
                if lhs == i64::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_lt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_lt_u,
            Instruction::i64_lt_u_imm,
            Instruction::i64_gt_u_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_lt_u_imm16,
            swap_ops!(Instruction::i64_gt_u_imm16),
            UntypedValue::i64_lt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: u64| {
                if rhs == u64::MIN {
                    // Optimization: `x < MIN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, rhs: Register| {
                if lhs == u64::MAX {
                    // Optimization: `MAX < x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_gt_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_gt_s,
            Instruction::i64_gt_s_imm,
            Instruction::i64_lt_s_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_gt_s_imm16,
            swap_ops!(Instruction::i64_lt_s_imm16),
            UntypedValue::i64_gt_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == i64::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, rhs: Register| {
                if lhs == i64::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_gt_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_gt_u,
            Instruction::i64_gt_u_imm,
            Instruction::i64_lt_u_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_gt_u_imm16,
            swap_ops!(Instruction::i64_lt_u_imm16),
            UntypedValue::i64_gt_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: u64| {
                if rhs == u64::MAX {
                    // Optimization: `x > MAX` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, rhs: Register| {
                if lhs == u64::MIN {
                    // Optimization: `MIN > x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_le_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_le_s,
            Instruction::i64_le_s_imm,
            Instruction::i64_ge_s_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_le_s_imm16,
            swap_ops!(Instruction::i64_ge_s_imm16),
            UntypedValue::i64_le_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == i64::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, rhs: Register| {
                if lhs == i64::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_le_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_le_u,
            Instruction::i64_le_u_imm,
            Instruction::i64_ge_u_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_le_u_imm16,
            swap_ops!(Instruction::i64_ge_u_imm16),
            UntypedValue::i64_le_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: u64| {
                if rhs == u64::MAX {
                    // Optimization: `x <= MAX` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, rhs: Register| {
                if lhs == u64::MIN {
                    // Optimization: `MIN <= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_ge_s(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_ge_s,
            Instruction::i64_ge_s_imm,
            Instruction::i64_le_s_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_ge_s_imm16,
            swap_ops!(Instruction::i64_le_s_imm16),
            UntypedValue::i64_ge_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == i64::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: i64, rhs: Register| {
                if lhs == i64::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_ge_u(&mut self) -> Self::Output {
        self.translate_binary(
            Instruction::i64_ge_u,
            Instruction::i64_ge_u_imm,
            Instruction::i64_le_u_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_ge_u_imm16,
            swap_ops!(Instruction::i64_le_u_imm16),
            UntypedValue::i64_ge_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: u64| {
                if rhs == u64::MIN {
                    // Optimization: `x >= MIN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: u64, rhs: Register| {
                if lhs == u64::MAX {
                    // Optimization: `MAX >= x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_eq(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f32>(
            Instruction::f32_eq,
            Instruction::f32_eq_imm,
            Self::make_instr_imm_param_32,
            UntypedValue::f32_eq,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x == x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg_in: Register, imm_in: f32| {
                if imm_in.is_nan() {
                    // Optimization: `NaN == x` or `x == NaN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_ne(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f32>(
            Instruction::f32_ne,
            Instruction::f32_ne_imm,
            Self::make_instr_imm_param_32,
            UntypedValue::f32_ne,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x != x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg_in: Register, imm_in: f32| {
                if imm_in.is_nan() {
                    // Optimization: `NaN == x` or `x == NaN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
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
        self.translate_fbinary_commutative::<f64>(
            Instruction::f64_eq,
            Instruction::f64_eq_imm,
            Self::make_instr_imm_param_64,
            UntypedValue::f64_eq,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x == x` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg_in: Register, imm_in: f64| {
                if imm_in.is_nan() {
                    // Optimization: `NaN == x` or `x == NaN` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_ne(&mut self) -> Self::Output {
        self.translate_fbinary_commutative::<f64>(
            Instruction::f64_ne,
            Instruction::f64_ne_imm,
            Self::make_instr_imm_param_64,
            UntypedValue::f64_ne,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x != x` is always `false`
                    this.alloc.stack.push_const(false);
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg_in: Register, imm_in: f64| {
                if imm_in.is_nan() {
                    // Optimization: `NaN == x` or `x == NaN` is always `true`
                    this.alloc.stack.push_const(true);
                    return Ok(true);
                }
                Ok(false)
            },
        )
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
        self.translate_unary(Instruction::i32_clz, UntypedValue::i32_clz)
    }

    fn visit_i32_ctz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_ctz, UntypedValue::i32_ctz)
    }

    fn visit_i32_popcnt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i32_popcnt, UntypedValue::i32_popcnt)
    }

    fn visit_i32_add(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_add,
            Instruction::i32_add_imm,
            Self::make_instr_imm_param_32,
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
        self.translate_binary(
            Instruction::i32_sub,
            Instruction::i32_sub_imm,
            Instruction::i32_sub_imm_rev,
            Self::make_instr_imm_param_32,
            Instruction::i32_sub_imm16,
            Instruction::i32_sub_imm16_rev,
            UntypedValue::i32_sub,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `sub x - x` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i32));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == 0 {
                    // Optimization: `sub x - 0` is same as `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i32_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_mul,
            Instruction::i32_mul_imm,
            Self::make_instr_imm_param_32,
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
        self.translate_divrem(
            Instruction::i32_div_s,
            Instruction::i32_div_s_imm,
            Instruction::i32_div_s_imm_rev,
            Self::make_instr_imm_param_32,
            Instruction::i32_div_s_imm16,
            Instruction::i32_div_s_imm16_rev,
            UntypedValue::i32_div_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x / x` is always `1`
                    this.alloc.stack.push_const(UntypedValue::from(1_i32));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_div_u(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i32_div_u,
            Instruction::i32_div_u_imm,
            Instruction::i32_div_u_imm_rev,
            Self::make_instr_imm_param_32,
            Instruction::i32_div_u_imm16,
            Instruction::i32_div_u_imm16_rev,
            UntypedValue::i32_div_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x / x` is always `1`
                    this.alloc.stack.push_const(UntypedValue::from(1_i32));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_rem_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i32_rem_s,
            Instruction::i32_rem_s_imm,
            Instruction::i32_rem_s_imm_rev,
            Self::make_instr_imm_param_32,
            Instruction::i32_rem_s_imm16,
            Instruction::i32_rem_s_imm16_rev,
            UntypedValue::i32_rem_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x % x` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i32));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == 1 || rhs == -1 {
                    // Optimization: `x % 1` or `x % -1` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i32));
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_rem_u(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i32_rem_u,
            Instruction::i32_rem_u_imm,
            Instruction::i32_rem_u_imm_rev,
            Self::make_instr_imm_param_32,
            Instruction::i32_rem_u_imm16,
            Instruction::i32_rem_u_imm16_rev,
            UntypedValue::i32_rem_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x % x` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i32));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i32| {
                if rhs == 1 {
                    // Optimization: `x % 1` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i32));
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_and(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i32_and,
            Instruction::i32_and_imm,
            Self::make_instr_imm_param_32,
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
        self.translate_binary_commutative(
            Instruction::i32_or,
            Instruction::i32_or_imm,
            Self::make_instr_imm_param_32,
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
        self.translate_binary_commutative(
            Instruction::i32_xor,
            Instruction::i32_xor_imm,
            Self::make_instr_imm_param_32,
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
        self.translate_shift(
            Instruction::i32_shl,
            Instruction::i32_shl_imm,
            Self::make_instr_imm_param_32::<i32>,
            Instruction::i32_shl_imm_rev,
            Instruction::i32_shl_imm16_rev,
            UntypedValue::i32_shl,
            Self::no_custom_opt,
        )
    }

    fn visit_i32_shr_s(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_shr_s,
            Instruction::i32_shr_s_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_shr_s_imm_rev,
            Instruction::i32_shr_s_imm16_rev,
            UntypedValue::i32_shr_s,
            |this, lhs: i32, rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_shr_u(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_shr_u,
            Instruction::i32_shr_u_imm,
            Self::make_instr_imm_param_32::<i32>,
            Instruction::i32_shr_u_imm_rev,
            Instruction::i32_shr_u_imm16_rev,
            UntypedValue::i32_shr_u,
            Self::no_custom_opt,
        )
    }

    fn visit_i32_rotl(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_rotl,
            Instruction::i32_rotl_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_rotl_imm_rev,
            Instruction::i32_rotl_imm16_rev,
            UntypedValue::i32_rotl,
            |this, lhs: i32, rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1.rotate_left(x)` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i32_rotr(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i32_rotr,
            Instruction::i32_rotr_imm,
            Self::make_instr_imm_param_32,
            Instruction::i32_rotr_imm_rev,
            Instruction::i32_rotr_imm16_rev,
            UntypedValue::i32_rotr,
            |this, lhs: i32, rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1.rotate_right(x)` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_clz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_clz, UntypedValue::i64_clz)
    }

    fn visit_i64_ctz(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_ctz, UntypedValue::i64_ctz)
    }

    fn visit_i64_popcnt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::i64_popcnt, UntypedValue::i64_popcnt)
    }

    fn visit_i64_add(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_add,
            Instruction::i64_add_imm,
            Self::make_instr_imm_param_64,
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
        self.translate_binary(
            Instruction::i64_sub,
            Instruction::i64_sub_imm,
            Instruction::i64_sub_imm_rev,
            Self::make_instr_imm_param_64,
            Instruction::i64_sub_imm16,
            Instruction::i64_sub_imm16_rev,
            UntypedValue::i64_sub,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `sub x - x` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i64));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == 0 {
                    // Optimization: `sub x - 0` is same as `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            Self::no_custom_opt,
        )
    }

    fn visit_i64_mul(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_mul,
            Instruction::i64_mul_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_mul_imm16,
            UntypedValue::i64_mul,
            Self::no_custom_opt,
            |this, reg: Register, value: i64| {
                if value == 0 {
                    // Optimization: `add x * 0` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i64));
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

    fn visit_i64_div_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i64_div_s,
            Instruction::i64_div_s_imm,
            Instruction::i64_div_s_imm_rev,
            Self::make_instr_imm_param_64,
            Instruction::i64_div_s_imm16,
            Instruction::i64_div_s_imm16_rev,
            UntypedValue::i64_div_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x / x` is always `1`
                    this.alloc.stack.push_const(UntypedValue::from(1_i64));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_div_u(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i64_div_u,
            Instruction::i64_div_u_imm,
            Instruction::i64_div_u_imm_rev,
            Self::make_instr_imm_param_64,
            Instruction::i64_div_u_imm16,
            Instruction::i64_div_u_imm16_rev,
            UntypedValue::i64_div_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x / x` is always `1`
                    this.alloc.stack.push_const(UntypedValue::from(1_i64));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == 1 {
                    // Optimization: `x / 1` is always `x`
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_rem_s(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i64_rem_s,
            Instruction::i64_rem_s_imm,
            Instruction::i64_rem_s_imm_rev,
            Self::make_instr_imm_param_64,
            Instruction::i64_rem_s_imm16,
            Instruction::i64_rem_s_imm16_rev,
            UntypedValue::i64_rem_s,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x % x` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i64));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == 1 || rhs == -1 {
                    // Optimization: `x % 1` or `x % -1` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i64));
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_rem_u(&mut self) -> Self::Output {
        self.translate_divrem(
            Instruction::i64_rem_u,
            Instruction::i64_rem_u_imm,
            Instruction::i64_rem_u_imm_rev,
            Self::make_instr_imm_param_64,
            Instruction::i64_rem_u_imm16,
            Instruction::i64_rem_u_imm16_rev,
            UntypedValue::i64_rem_u,
            |this, lhs: Register, rhs: Register| {
                if lhs == rhs {
                    // Optimization: `x % x` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i64));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, lhs: Register, rhs: i64| {
                if rhs == 1 {
                    // Optimization: `x % 1` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i64));
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_and(&mut self) -> Self::Output {
        self.translate_binary_commutative(
            Instruction::i64_and,
            Instruction::i64_and_imm,
            Self::make_instr_imm_param_64,
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
        self.translate_binary_commutative(
            Instruction::i64_or,
            Instruction::i64_or_imm,
            Self::make_instr_imm_param_64,
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
        self.translate_binary_commutative(
            Instruction::i64_xor,
            Instruction::i64_xor_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_xor_imm16,
            UntypedValue::i64_xor,
            |this, lhs, rhs| {
                if lhs == rhs {
                    // Optimization: `x ^ x` is always `0`
                    this.alloc.stack.push_const(UntypedValue::from(0_i64));
                    return Ok(true);
                }
                Ok(false)
            },
            |this, reg: Register, value: i64| {
                if value == 0 {
                    // Optimization: `x ^ 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_shl(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_shl,
            Instruction::i64_shl_imm,
            Self::make_instr_imm_param_64::<i64>,
            Instruction::i64_shl_imm_rev,
            Instruction::i64_shl_imm16_rev,
            UntypedValue::i64_shl,
            Self::no_custom_opt,
        )
    }

    fn visit_i64_shr_s(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_shr_s,
            Instruction::i64_shr_s_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_shr_s_imm_rev,
            Instruction::i64_shr_s_imm16_rev,
            UntypedValue::i64_shr_s,
            |this, lhs: i64, rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_shr_u(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_shr_u,
            Instruction::i64_shr_u_imm,
            Self::make_instr_imm_param_64::<i64>,
            Instruction::i64_shr_u_imm_rev,
            Instruction::i64_shr_u_imm16_rev,
            UntypedValue::i64_shr_u,
            Self::no_custom_opt,
        )
    }

    fn visit_i64_rotl(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_rotl,
            Instruction::i64_rotl_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_rotl_imm_rev,
            Instruction::i64_rotl_imm16_rev,
            UntypedValue::i64_rotl,
            |this, lhs: i64, rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_i64_rotr(&mut self) -> Self::Output {
        self.translate_shift(
            Instruction::i64_rotr,
            Instruction::i64_rotr_imm,
            Self::make_instr_imm_param_64,
            Instruction::i64_rotr_imm_rev,
            Instruction::i64_rotr_imm16_rev,
            UntypedValue::i64_rotr,
            |this, lhs: i64, rhs: Register| {
                if lhs == -1 {
                    // Optimization: `-1 >> x` is always `-1` for every valid `x`
                    this.alloc.stack.push_const(lhs);
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_abs(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_abs, UntypedValue::f32_abs)
    }

    fn visit_f32_neg(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_neg, UntypedValue::f32_neg)
    }

    fn visit_f32_ceil(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_ceil, UntypedValue::f32_ceil)
    }

    fn visit_f32_floor(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_floor, UntypedValue::f32_floor)
    }

    fn visit_f32_trunc(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_trunc, UntypedValue::f32_trunc)
    }

    fn visit_f32_nearest(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_nearest, UntypedValue::f32_nearest)
    }

    fn visit_f32_sqrt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f32_sqrt, UntypedValue::f32_sqrt)
    }

    fn visit_f32_add(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_add,
            Instruction::f32_add_imm,
            Self::make_instr_imm_param_32,
            UntypedValue::f32_add,
            Self::no_custom_opt,
            |this, reg: Register, value: f32| {
                if value == 0.0 || value == -0.0 {
                    // Optimization: `add x + 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_sub(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f32_sub,
            Instruction::f32_sub_imm,
            Instruction::f32_sub_imm_rev,
            Self::make_instr_imm_param_32,
            UntypedValue::f32_sub,
            Self::no_custom_opt,
            |this, lhs: Register, rhs: f32| {
                if rhs == 0.0 && rhs.is_sign_positive() {
                    // Optimization: `x - 0` is same as `x`
                    //
                    // Note due to behavior dictated by the Wasm specification
                    // we cannot apply this optimization for negative zeros.
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            // Unfortuantely we cannot optimize for the case that `lhs == 0.0`
            // since the Wasm specification mandates different behavior in
            // dependence of `rhs` which we do not know at this point.
            Self::no_custom_opt,
        )
    }

    fn visit_f32_mul(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_mul,
            Instruction::f32_mul_imm,
            Self::make_instr_imm_param_32::<f32>,
            UntypedValue::f32_mul,
            Self::no_custom_opt,
            // Unfortunately we cannot apply `x * 0` or `0 * x` optimizations
            // since Wasm mandates different behaviors if `x` is infinite or
            // NaN in these cases.
            Self::no_custom_opt,
        )
    }

    fn visit_f32_div(&mut self) -> Self::Output {
        self.translate_fbinary::<f32>(
            Instruction::f32_div,
            Instruction::f32_div_imm,
            Instruction::f32_div_imm_rev,
            Self::make_instr_imm_param_32,
            UntypedValue::f32_div,
            Self::no_custom_opt,
            Self::no_custom_opt,
            Self::no_custom_opt,
        )
    }

    fn visit_f32_min(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_min,
            Instruction::f32_min_imm,
            Self::make_instr_imm_param_32,
            UntypedValue::f32_min,
            Self::no_custom_opt,
            |this, reg: Register, value: f32| {
                if value.is_infinite() && value.is_sign_positive() {
                    // Optimization: `min(x, +inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_max(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f32_max,
            Instruction::f32_max_imm,
            Self::make_instr_imm_param_32,
            UntypedValue::f32_max,
            Self::no_custom_opt,
            |this, reg: Register, value: f32| {
                if value.is_infinite() && value.is_sign_negative() {
                    // Optimization: `max(x, -inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f32_copysign(&mut self) -> Self::Output {
        self.translate_fcopysign(
            Instruction::f32_copysign,
            Instruction::f32_copysign_imm,
            Instruction::f32_copysign_imm_rev,
            Self::make_instr_imm_param_32::<f32>,
            UntypedValue::f32_copysign,
        )
    }

    fn visit_f64_abs(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_abs, UntypedValue::f64_abs)
    }

    fn visit_f64_neg(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_neg, UntypedValue::f64_neg)
    }

    fn visit_f64_ceil(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_ceil, UntypedValue::f64_ceil)
    }

    fn visit_f64_floor(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_floor, UntypedValue::f64_floor)
    }

    fn visit_f64_trunc(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_trunc, UntypedValue::f64_trunc)
    }

    fn visit_f64_nearest(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_nearest, UntypedValue::f64_nearest)
    }

    fn visit_f64_sqrt(&mut self) -> Self::Output {
        self.translate_unary(Instruction::f64_sqrt, UntypedValue::f64_sqrt)
    }

    fn visit_f64_add(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_add,
            Instruction::f64_add_imm,
            Self::make_instr_imm_param_64,
            UntypedValue::f64_add,
            Self::no_custom_opt,
            |this, reg: Register, value: f64| {
                if value == 0.0 || value == -0.0 {
                    // Optimization: `add x + 0` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_sub(&mut self) -> Self::Output {
        self.translate_fbinary(
            Instruction::f64_sub,
            Instruction::f64_sub_imm,
            Instruction::f64_sub_imm_rev,
            Self::make_instr_imm_param_64,
            UntypedValue::f64_sub,
            Self::no_custom_opt,
            |this, lhs: Register, rhs: f64| {
                if rhs == 0.0 && rhs.is_sign_positive() {
                    // Optimization: `x - 0` is same as `x`
                    //
                    // Note due to behavior dictated by the Wasm specification
                    // we cannot apply this optimization for negative zeros.
                    this.alloc.stack.push_register(lhs)?;
                    return Ok(true);
                }
                Ok(false)
            },
            // Unfortuantely we cannot optimize for the case that `lhs == 0.0`
            // since the Wasm specification mandates different behavior in
            // dependence of `rhs` which we do not know at this point.
            Self::no_custom_opt,
        )
    }

    fn visit_f64_mul(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_mul,
            Instruction::f64_mul_imm,
            Self::make_instr_imm_param_64::<f64>,
            UntypedValue::f64_mul,
            Self::no_custom_opt,
            // Unfortunately we cannot apply `x * 0` or `0 * x` optimizations
            // since Wasm mandates different behaviors if `x` is infinite or
            // NaN in these cases.
            Self::no_custom_opt,
        )
    }

    fn visit_f64_div(&mut self) -> Self::Output {
        self.translate_fbinary::<f64>(
            Instruction::f64_div,
            Instruction::f64_div_imm,
            Instruction::f64_div_imm_rev,
            Self::make_instr_imm_param_64,
            UntypedValue::f64_div,
            Self::no_custom_opt,
            Self::no_custom_opt,
            Self::no_custom_opt,
        )
    }

    fn visit_f64_min(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_min,
            Instruction::f64_min_imm,
            Self::make_instr_imm_param_64,
            UntypedValue::f64_min,
            Self::no_custom_opt,
            |this, reg: Register, value: f64| {
                if value.is_infinite() && value.is_sign_positive() {
                    // Optimization: `min(x, +inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_max(&mut self) -> Self::Output {
        self.translate_fbinary_commutative(
            Instruction::f64_max,
            Instruction::f64_max_imm,
            Self::make_instr_imm_param_64,
            UntypedValue::f64_max,
            Self::no_custom_opt,
            |this, reg: Register, value: f64| {
                if value.is_infinite() && value.is_sign_negative() {
                    // Optimization: `min(x, +inf)` is same as `x`
                    this.alloc.stack.push_register(reg)?;
                    return Ok(true);
                }
                Ok(false)
            },
        )
    }

    fn visit_f64_copysign(&mut self) -> Self::Output {
        self.translate_fcopysign(
            Instruction::f64_copysign,
            Instruction::f64_copysign_imm,
            Instruction::f64_copysign_imm_rev,
            Self::make_instr_imm_param_64::<f64>,
            UntypedValue::f64_copysign,
        )
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
