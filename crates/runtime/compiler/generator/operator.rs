use std::fmt::Debug;

use anyhow::Result;
use llvm::types::{LLFloatType, LLIntType};
use llvm::values::{LLFloatPredicate, LLIntPredicate};
use llvm::{
    basic_block::LLBasicBlock,
    builder::LLBuilder,
    context::LLContext,
    intrinsics,
    module::LLModule,
    types::LLNumType,
    values::{LLAlloca, LLFunction, LLValue},
};
use wasmparser::Operator;

use crate::types::FuncType;

use super::{FunctionBodyGenerator, Generator};

//------------------------------------------------------------------------------
// Type Definitions
//------------------------------------------------------------------------------

/// WebAssembly has three block types for representing control flow.
pub(crate) enum Control {
    /// ```text
    ///    ┌──────────┐
    /// ┌──┤   Cond   │
    /// │  └────┬─────┘
    /// │       │
    /// │  ┌────▼─────┐
    /// │  │   Then   ├──┐
    /// │  └──────────┘  │
    /// │                │
    /// │  ┌──────────┐  │
    /// └──►   Else   │  │
    ///    └────┬─────┘  │
    ///         │        │
    ///    ┌────▼─────┐  │
    ///    │   End    ◄──┘
    ///    └──────────┘
    /// ```
    If {
        r#else: LLBasicBlock,
        end: LLBasicBlock,
    },
    /// ```text
    /// ┌─────────┐
    /// │   Loop  ◄──┐
    /// └────┬────┘  │
    ///      │       │
    ///      ├───────┘
    ///      │
    /// ┌────▼────┐
    /// │   End   │
    /// └─────────┘
    /// ```
    Loop {
        begin: LLBasicBlock,
        end: LLBasicBlock,
    },
    /// ```text
    /// ┌─────────┐
    /// │  Block  │
    /// └────┬────┘
    ///      │
    ///      │
    /// ┌────▼────┐
    /// │  End    │
    /// └─────────┘
    /// ```
    Block { end: LLBasicBlock },
}

/// Generates LLVM IR for an operation.
pub(crate) struct OperatorGenerator<'a> {
    pub(crate) operator: &'a Operator<'a>,
    pub(crate) llvm_module: &'a mut LLModule,
    pub(crate) llvm_context: &'a LLContext,
    pub(crate) llvm_locals: &'a Vec<LLAlloca>,
    pub(crate) llvm_builder: &'a mut LLBuilder,
    pub(crate) llvm_func: &'a mut LLFunction,
    pub(crate) control_stack: &'a mut Vec<Control>,
    pub(crate) value_stack: &'a mut Vec<Box<dyn LLValue>>,
    pub(crate) func_type: &'a FuncType,
}

//------------------------------------------------------------------------------
// Implementations
//------------------------------------------------------------------------------

impl<'a> Generator for OperatorGenerator<'a> {
    type Value = ();

    fn generate(&mut self) -> Result<()> {
        let block_count = self.control_stack.len();
        match self.operator {
            Operator::Unreachable => {
                self.llvm_builder.build_unreachable();
            }
            Operator::Nop => {
                // %nop = add i32 0, 0
                let llvm_zero = &self.llvm_context.i32_type().zero();
                self.llvm_builder
                    .build_int_add(llvm_zero, llvm_zero, "nop")?;
            }
            Operator::Block { .. } => {
                let llvm_begin_bb = self.llvm_func.create_and_append_basic_block(
                    &format!("block_begin.{}", block_count),
                    self.llvm_context,
                )?;

                let llvm_end_bb =
                    LLBasicBlock::new(&format!("block_end.{}", block_count), self.llvm_context)?;

                // Position the builder at the beginning of the begin block.
                self.llvm_builder.position_at_end(&llvm_begin_bb);

                self.control_stack.push(Control::Block { end: llvm_end_bb });
            }
            Operator::Loop { .. } => {
                let llvm_begin_bb = self.llvm_func.create_and_append_basic_block(
                    &format!("loop_begin.{}", block_count),
                    self.llvm_context,
                )?;

                let llvm_end_bb =
                    LLBasicBlock::new(&format!("loop_end.{}", block_count), self.llvm_context)?;

                // Position the builder at the beginning of the begin block.
                self.llvm_builder.position_at_end(&llvm_begin_bb);

                self.control_stack.push(Control::Loop {
                    begin: llvm_begin_bb,
                    end: llvm_end_bb,
                });
            }
            Operator::If { .. } => {
                let llvm_then_bb = self.llvm_func.create_and_append_basic_block(
                    &format!("if_then.{}", block_count),
                    self.llvm_context,
                )?;

                let llvm_else_bb = self.llvm_func.create_and_append_basic_block(
                    &format!("if_else.{}", block_count),
                    self.llvm_context,
                )?;

                let llvm_end_bb =
                    LLBasicBlock::new(&format!("if_end.{}", block_count), self.llvm_context)?;

                // Add conditional branching instruction.
                let stack_value = self.value_stack.pop().unwrap();
                self.llvm_builder
                    .build_cond_br(stack_value.as_ref(), &llvm_then_bb, &llvm_else_bb);

                // Position the builder at the beginning of the then block.
                self.llvm_builder.position_at_end(&llvm_then_bb);

                self.control_stack.push(Control::If {
                    r#else: llvm_else_bb,
                    end: llvm_end_bb,
                });
            }
            Operator::Else => {
                let control = self.control_stack.last_mut().unwrap();
                let (llvm_else_bb, llvm_end_bb) = match control {
                    Control::If { r#else, end, .. } => (r#else, end),
                    _ => unreachable!(),
                };

                // Just concluded if_then block should break to if_end block.
                self.llvm_builder.build_br(llvm_end_bb);
                self.llvm_builder.position_at_end(llvm_else_bb);
            }
            // Operator::Try { ty } => todo!(),
            // Operator::Catch { index } => todo!(),
            // Operator::Throw { index } => todo!(),
            // Operator::Rethrow { relative_depth } => todo!(),
            Operator::End => {
                // Position the builder at the beginning of the then block.
                if let Some(mut control) = self.control_stack.pop() {
                    match control {
                        Control::If { ref mut end, .. } => {
                            self.llvm_func.append_basic_block(end);
                            self.llvm_builder.position_at_end(end);
                        }
                        Control::Loop {
                            ref begin,
                            ref mut end,
                            ..
                        } => {
                            // Just concluded begin block should break back to itself
                            self.llvm_builder.build_br(begin);
                            self.llvm_func.append_basic_block(end);
                            self.llvm_builder.position_at_end(end);
                        }
                        Control::Block { ref mut end, .. } => {
                            self.llvm_func.append_basic_block(end);
                            self.llvm_builder.position_at_end(end);
                        }
                    }
                }
            }
            Operator::Br { relative_depth } => {
                let rev_index = self.control_stack.len() - *relative_depth as usize - 1;
                let llvm_end_bb = match &self.control_stack[rev_index] {
                    Control::If { end, .. } => end,
                    Control::Loop { end, .. } => end,
                    Control::Block { end, .. } => end,
                };

                self.llvm_builder.build_br(llvm_end_bb);
            }
            // Operator::BrIf { relative_depth } => todo!(),
            // Operator::BrTable { table } => todo!(),
            Operator::Return => {
                FunctionBodyGenerator::generate_return(
                    self.llvm_context,
                    self.llvm_builder,
                    self.value_stack,
                    &self.func_type.results,
                )?;
            }
            // Operator::Call { function_index } => todo!(),
            // Operator::CallIndirect { index, table_index } => todo!(),
            // Operator::ReturnCall { function_index } => todo!(),
            // Operator::ReturnCallIndirect { index, table_index } => todo!(),
            // Operator::Delegate { relative_depth } => todo!(),
            // Operator::CatchAll => todo!(),
            // Operator::Drop => todo!(),
            // Operator::Select => todo!(),
            // Operator::TypedSelect { ty } => todo!(),
            Operator::LocalGet { local_index } => {
                let llvm_local = self.llvm_locals[*local_index as usize].clone();
                self.value_stack.push(Box::new(llvm_local));
            }
            Operator::LocalSet { local_index } => {
                let operand = self.value_stack.pop().unwrap();
                self.llvm_builder
                    .build_store(operand.as_ref(), &self.llvm_locals[*local_index as usize]);
            }
            Operator::LocalTee { local_index } => {
                // We don't consume the value on the stack.
                let operand = self.value_stack.last().unwrap();
                self.llvm_builder
                    .build_store(operand.as_ref(), &self.llvm_locals[*local_index as usize]);
            }
            // Operator::GlobalGet { global_index } => todo!(),
            // Operator::GlobalSet { global_index } => todo!(),
            // Operator::I32Load { memarg } => todo!(),
            // Operator::I64Load { memarg } => todo!(),
            // Operator::F32Load { memarg } => todo!(),
            // Operator::F64Load { memarg } => todo!(),
            // Operator::I32Load8S { memarg } => todo!(),
            // Operator::I32Load8U { memarg } => todo!(),
            // Operator::I32Load16S { memarg } => todo!(),
            // Operator::I32Load16U { memarg } => todo!(),
            // Operator::I64Load8S { memarg } => todo!(),
            // Operator::I64Load8U { memarg } => todo!(),
            // Operator::I64Load16S { memarg } => todo!(),
            // Operator::I64Load16U { memarg } => todo!(),
            // Operator::I64Load32S { memarg } => todo!(),
            // Operator::I64Load32U { memarg } => todo!(),
            // Operator::I32Store { memarg } => todo!(),
            // Operator::I64Store { memarg } => todo!(),
            // Operator::F32Store { memarg } => todo!(),
            // Operator::F64Store { memarg } => todo!(),
            // Operator::I32Store8 { memarg } => todo!(),
            // Operator::I32Store16 { memarg } => todo!(),
            // Operator::I64Store8 { memarg } => todo!(),
            // Operator::I64Store16 { memarg } => todo!(),
            // Operator::I64Store32 { memarg } => todo!(),
            // Operator::MemorySize { mem, mem_byte } => todo!(),
            // Operator::MemoryGrow { mem, mem_byte } => todo!(),
            Operator::I32Const { value } => {
                let llvm_const = self.llvm_context.i32_type().constant(*value as u64, false);
                self.value_stack.push(Box::new(llvm_const));
            }
            Operator::I64Const { value } => {
                let llvm_const = self.llvm_context.i64_type().constant(*value as u64, false);
                self.value_stack.push(Box::new(llvm_const));
            }
            Operator::F32Const { value } => {
                let llvm_const = self.llvm_context.f32_type().constant(value.bits() as f64);
                self.value_stack.push(Box::new(llvm_const));
            }
            Operator::F64Const { value } => {
                let llvm_const = self.llvm_context.f64_type().constant(value.bits() as f64);
                self.value_stack.push(Box::new(llvm_const));
            }
            // Operator::RefNull { ty } => todo!(),
            // Operator::RefIsNull => todo!(),
            // Operator::RefFunc { function_index } => todo!(),
            Operator::I32Eqz | Operator::I64Eqz => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_const = self.llvm_context.i32_type().zero();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::EQ,
                    operand.as_ref(),
                    &llvm_const,
                    "icmp_eqz",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Eq | Operator::I64Eq => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::EQ,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "icmp_eq",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Ne | Operator::I64Ne => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::NE,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "icmp_ne",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32LtS | Operator::I64LtS => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::SLT,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "icmp_lt_s",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32LtU | Operator::I64LtU => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::ULT,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "icmp_lt_u",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32GtS | Operator::I64GtS => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::SGT,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "icmp_gt_s",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32GtU | Operator::I64GtU => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::UGT,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "icmp_gt_u",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32LeS | Operator::I64LeS => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::SLE,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "icmp_le_s",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32LeU | Operator::I64LeU => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::ULE,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "icmp_le_u",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32GeS | Operator::I64GeS => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::SGE,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "icmp_ge_s",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32GeU | Operator::I64GeU => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_int_cmp(
                    LLIntPredicate::UGE,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "icmp_ge_u",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Eq | Operator::F64Eq => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_float_cmp(
                    LLFloatPredicate::OEQ,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "fcmp_eq",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Ne | Operator::F64Ne => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_float_cmp(
                    LLFloatPredicate::UNE,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "fcmp_ne",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Lt | Operator::F64Lt => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_float_cmp(
                    LLFloatPredicate::OLT,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "fcmp_lt",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Gt | Operator::F64Gt => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_float_cmp(
                    LLFloatPredicate::OGT,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "fcmp_gt",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Le | Operator::F64Le => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_float_cmp(
                    LLFloatPredicate::OLE,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "fcmp_le",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Ge | Operator::F64Ge => {
                let operand2 = self.value_stack.pop().unwrap();
                let operand1 = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_float_cmp(
                    LLFloatPredicate::OGE,
                    operand1.as_ref(),
                    operand2.as_ref(),
                    "fcmp_ge",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Clz => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::CTLZ_I32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "clz",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Ctz => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::CTTZ_I32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "ctz",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Popcnt => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::CTPOP_I32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "popcnt",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Add | Operator::I64Add => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_add(lhs.as_ref(), rhs.as_ref(), "add")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Sub | Operator::I64Sub => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_sub(lhs.as_ref(), rhs.as_ref(), "sub")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Mul | Operator::I64Mul => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_mul(lhs.as_ref(), rhs.as_ref(), "mul")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32DivS | Operator::I64DivS => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_sdiv(lhs.as_ref(), rhs.as_ref(), "div_s")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32DivU | Operator::I64DivU => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_udiv(lhs.as_ref(), rhs.as_ref(), "div_u")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32RemS | Operator::I64RemS => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_srem(lhs.as_ref(), rhs.as_ref(), "rem_s")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32RemU | Operator::I64RemU => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_urem(lhs.as_ref(), rhs.as_ref(), "rem_u")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32And | Operator::I64And => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_and(lhs.as_ref(), rhs.as_ref(), "and")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Or | Operator::I64Or => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_or(lhs.as_ref(), rhs.as_ref(), "or")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Xor | Operator::I64Xor => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_xor(lhs.as_ref(), rhs.as_ref(), "xor")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Shl | Operator::I64Shl => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_shl(lhs.as_ref(), rhs.as_ref(), "shl")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32ShrS | Operator::I64ShrS => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_ashr(lhs.as_ref(), rhs.as_ref(), "shr_s")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32ShrU | Operator::I64ShrU => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_int_lshr(lhs.as_ref(), rhs.as_ref(), "shr_u")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Rotl | Operator::I64Rotl => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::FSHL_I32,
                    &[rhs.as_ref(), rhs.as_ref(), lhs.as_ref()],
                    self.llvm_module,
                    "rotl",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I32Rotr | Operator::I64Rotr => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::FSHR_I32,
                    &[rhs.as_ref(), rhs.as_ref(), lhs.as_ref()],
                    self.llvm_module,
                    "rotr",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I64Clz => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::CTLZ_I64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "clz",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I64Ctz => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::CTTZ_I64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "ctz",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::I64Popcnt => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::CTPOP_I64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "popcnt",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Abs => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::ABS_F32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "abs",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Neg => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::NEG_F32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "neg",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Ceil => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::CEIL_F32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "ceil",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Floor => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::FLOOR_F32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "floor",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Trunc => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::TRUNC_F32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "trunc",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Nearest => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::ROUND_EVEN_F32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "nearest",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Sqrt => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::SQRT_F32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "sqrt",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Add | Operator::F64Add => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_float_add(lhs.as_ref(), rhs.as_ref(), "add")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Sub | Operator::F64Sub => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_float_sub(lhs.as_ref(), rhs.as_ref(), "sub")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Mul | Operator::F64Mul => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_float_mul(lhs.as_ref(), rhs.as_ref(), "mul")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Div | Operator::F64Div => {
                let rhs = self.value_stack.pop().unwrap();
                let lhs = self.value_stack.pop().unwrap();
                let llvm_result =
                    self.llvm_builder
                        .build_float_div(lhs.as_ref(), rhs.as_ref(), "div")?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Min => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::MINIMUM_F32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "min",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Max => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::MAXIMUM_F32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "max",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F32Copysign => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::COPYSIGN_F32,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "copysign",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F64Abs => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::ABS_F64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "abs",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F64Neg => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::NEG_F64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "neg",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F64Ceil => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::CEIL_F64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "ceil",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F64Floor => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::FLOOR_F64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "floor",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F64Trunc => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::TRUNC_F64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "trunc",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F64Nearest => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::ROUND_EVEN_F64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "nearest",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F64Sqrt => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::SQRT_F64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "sqrt",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F64Min => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::MINIMUM_F64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "min",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F64Max => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::MAXIMUM_F64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "max",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            Operator::F64Copysign => {
                let operand = self.value_stack.pop().unwrap();
                let llvm_result = self.llvm_builder.build_call_intrinsic(
                    &intrinsics::COPYSIGN_F64,
                    &[operand.as_ref()],
                    self.llvm_module,
                    "copysign",
                )?;

                self.value_stack.push(Box::new(llvm_result));
            }
            // Operator::I32WrapI64 => todo!(),
            // Operator::I32TruncF32S => todo!(),
            // Operator::I32TruncF32U => todo!(),
            // Operator::I32TruncF64S => todo!(),
            // Operator::I32TruncF64U => todo!(),
            // Operator::I64ExtendI32S => todo!(),
            // Operator::I64ExtendI32U => todo!(),
            // Operator::I64TruncF32S => todo!(),
            // Operator::I64TruncF32U => todo!(),
            // Operator::I64TruncF64S => todo!(),
            // Operator::I64TruncF64U => todo!(),
            // Operator::F32ConvertI32S => todo!(),
            // Operator::F32ConvertI32U => todo!(),
            // Operator::F32ConvertI64S => todo!(),
            // Operator::F32ConvertI64U => todo!(),
            // Operator::F32DemoteF64 => todo!(),
            // Operator::F64ConvertI32S => todo!(),
            // Operator::F64ConvertI32U => todo!(),
            // Operator::F64ConvertI64S => todo!(),
            // Operator::F64ConvertI64U => todo!(),
            // Operator::F64PromoteF32 => todo!(),
            // Operator::I32ReinterpretF32 => todo!(),
            // Operator::I64ReinterpretF64 => todo!(),
            // Operator::F32ReinterpretI32 => todo!(),
            // Operator::F64ReinterpretI64 => todo!(),
            // Operator::I32Extend8S => todo!(),
            // Operator::I32Extend16S => todo!(),
            // Operator::I64Extend8S => todo!(),
            // Operator::I64Extend16S => todo!(),
            // Operator::I64Extend32S => todo!(),
            // Operator::I32TruncSatF32S => todo!(),
            // Operator::I32TruncSatF32U => todo!(),
            // Operator::I32TruncSatF64S => todo!(),
            // Operator::I32TruncSatF64U => todo!(),
            // Operator::I64TruncSatF32S => todo!(),
            // Operator::I64TruncSatF32U => todo!(),
            // Operator::I64TruncSatF64S => todo!(),
            // Operator::I64TruncSatF64U => todo!(),
            // Operator::MemoryInit { segment, mem } => todo!(),
            // Operator::DataDrop { segment } => todo!(),
            // Operator::MemoryCopy { src, dst } => todo!(),
            // Operator::MemoryFill { mem } => todo!(),
            // Operator::TableInit { segment, table } => todo!(),
            // Operator::ElemDrop { segment } => todo!(),
            // Operator::TableCopy {
            //     dst_table,
            //     src_table,
            // } => todo!(),
            // Operator::TableFill { table } => todo!(),
            // Operator::TableGet { table } => todo!(),
            // Operator::TableSet { table } => todo!(),
            // Operator::TableGrow { table } => todo!(),
            // Operator::TableSize { table } => todo!(),
            // Operator::MemoryAtomicNotify { memarg } => todo!(),
            // Operator::MemoryAtomicWait32 { memarg } => todo!(),
            // Operator::MemoryAtomicWait64 { memarg } => todo!(),
            // Operator::AtomicFence { flags } => todo!(),
            // Operator::I32AtomicLoad { memarg } => todo!(),
            // Operator::I64AtomicLoad { memarg } => todo!(),
            // Operator::I32AtomicLoad8U { memarg } => todo!(),
            // Operator::I32AtomicLoad16U { memarg } => todo!(),
            // Operator::I64AtomicLoad8U { memarg } => todo!(),
            // Operator::I64AtomicLoad16U { memarg } => todo!(),
            // Operator::I64AtomicLoad32U { memarg } => todo!(),
            // Operator::I32AtomicStore { memarg } => todo!(),
            // Operator::I64AtomicStore { memarg } => todo!(),
            // Operator::I32AtomicStore8 { memarg } => todo!(),
            // Operator::I32AtomicStore16 { memarg } => todo!(),
            // Operator::I64AtomicStore8 { memarg } => todo!(),
            // Operator::I64AtomicStore16 { memarg } => todo!(),
            // Operator::I64AtomicStore32 { memarg } => todo!(),
            // Operator::I32AtomicRmwAdd { memarg } => todo!(),
            // Operator::I64AtomicRmwAdd { memarg } => todo!(),
            // Operator::I32AtomicRmw8AddU { memarg } => todo!(),
            // Operator::I32AtomicRmw16AddU { memarg } => todo!(),
            // Operator::I64AtomicRmw8AddU { memarg } => todo!(),
            // Operator::I64AtomicRmw16AddU { memarg } => todo!(),
            // Operator::I64AtomicRmw32AddU { memarg } => todo!(),
            // Operator::I32AtomicRmwSub { memarg } => todo!(),
            // Operator::I64AtomicRmwSub { memarg } => todo!(),
            // Operator::I32AtomicRmw8SubU { memarg } => todo!(),
            // Operator::I32AtomicRmw16SubU { memarg } => todo!(),
            // Operator::I64AtomicRmw8SubU { memarg } => todo!(),
            // Operator::I64AtomicRmw16SubU { memarg } => todo!(),
            // Operator::I64AtomicRmw32SubU { memarg } => todo!(),
            // Operator::I32AtomicRmwAnd { memarg } => todo!(),
            // Operator::I64AtomicRmwAnd { memarg } => todo!(),
            // Operator::I32AtomicRmw8AndU { memarg } => todo!(),
            // Operator::I32AtomicRmw16AndU { memarg } => todo!(),
            // Operator::I64AtomicRmw8AndU { memarg } => todo!(),
            // Operator::I64AtomicRmw16AndU { memarg } => todo!(),
            // Operator::I64AtomicRmw32AndU { memarg } => todo!(),
            // Operator::I32AtomicRmwOr { memarg } => todo!(),
            // Operator::I64AtomicRmwOr { memarg } => todo!(),
            // Operator::I32AtomicRmw8OrU { memarg } => todo!(),
            // Operator::I32AtomicRmw16OrU { memarg } => todo!(),
            // Operator::I64AtomicRmw8OrU { memarg } => todo!(),
            // Operator::I64AtomicRmw16OrU { memarg } => todo!(),
            // Operator::I64AtomicRmw32OrU { memarg } => todo!(),
            // Operator::I32AtomicRmwXor { memarg } => todo!(),
            // Operator::I64AtomicRmwXor { memarg } => todo!(),
            // Operator::I32AtomicRmw8XorU { memarg } => todo!(),
            // Operator::I32AtomicRmw16XorU { memarg } => todo!(),
            // Operator::I64AtomicRmw8XorU { memarg } => todo!(),
            // Operator::I64AtomicRmw16XorU { memarg } => todo!(),
            // Operator::I64AtomicRmw32XorU { memarg } => todo!(),
            // Operator::I32AtomicRmwXchg { memarg } => todo!(),
            // Operator::I64AtomicRmwXchg { memarg } => todo!(),
            // Operator::I32AtomicRmw8XchgU { memarg } => todo!(),
            // Operator::I32AtomicRmw16XchgU { memarg } => todo!(),
            // Operator::I64AtomicRmw8XchgU { memarg } => todo!(),
            // Operator::I64AtomicRmw16XchgU { memarg } => todo!(),
            // Operator::I64AtomicRmw32XchgU { memarg } => todo!(),
            // Operator::I32AtomicRmwCmpxchg { memarg } => todo!(),
            // Operator::I64AtomicRmwCmpxchg { memarg } => todo!(),
            // Operator::I32AtomicRmw8CmpxchgU { memarg } => todo!(),
            // Operator::I32AtomicRmw16CmpxchgU { memarg } => todo!(),
            // Operator::I64AtomicRmw8CmpxchgU { memarg } => todo!(),
            // Operator::I64AtomicRmw16CmpxchgU { memarg } => todo!(),
            // Operator::I64AtomicRmw32CmpxchgU { memarg } => todo!(),
            // Operator::V128Load { memarg } => todo!(),
            // Operator::V128Load8x8S { memarg } => todo!(),
            // Operator::V128Load8x8U { memarg } => todo!(),
            // Operator::V128Load16x4S { memarg } => todo!(),
            // Operator::V128Load16x4U { memarg } => todo!(),
            // Operator::V128Load32x2S { memarg } => todo!(),
            // Operator::V128Load32x2U { memarg } => todo!(),
            // Operator::V128Load8Splat { memarg } => todo!(),
            // Operator::V128Load16Splat { memarg } => todo!(),
            // Operator::V128Load32Splat { memarg } => todo!(),
            // Operator::V128Load64Splat { memarg } => todo!(),
            // Operator::V128Load32Zero { memarg } => todo!(),
            // Operator::V128Load64Zero { memarg } => todo!(),
            // Operator::V128Store { memarg } => todo!(),
            // Operator::V128Load8Lane { memarg, lane } => todo!(),
            // Operator::V128Load16Lane { memarg, lane } => todo!(),
            // Operator::V128Load32Lane { memarg, lane } => todo!(),
            // Operator::V128Load64Lane { memarg, lane } => todo!(),
            // Operator::V128Store8Lane { memarg, lane } => todo!(),
            // Operator::V128Store16Lane { memarg, lane } => todo!(),
            // Operator::V128Store32Lane { memarg, lane } => todo!(),
            // Operator::V128Store64Lane { memarg, lane } => todo!(),
            // Operator::V128Const { value } => todo!(),
            // Operator::I8x16Shuffle { lanes } => todo!(),
            // Operator::I8x16ExtractLaneS { lane } => todo!(),
            // Operator::I8x16ExtractLaneU { lane } => todo!(),
            // Operator::I8x16ReplaceLane { lane } => todo!(),
            // Operator::I16x8ExtractLaneS { lane } => todo!(),
            // Operator::I16x8ExtractLaneU { lane } => todo!(),
            // Operator::I16x8ReplaceLane { lane } => todo!(),
            // Operator::I32x4ExtractLane { lane } => todo!(),
            // Operator::I32x4ReplaceLane { lane } => todo!(),
            // Operator::I64x2ExtractLane { lane } => todo!(),
            // Operator::I64x2ReplaceLane { lane } => todo!(),
            // Operator::F32x4ExtractLane { lane } => todo!(),
            // Operator::F32x4ReplaceLane { lane } => todo!(),
            // Operator::F64x2ExtractLane { lane } => todo!(),
            // Operator::F64x2ReplaceLane { lane } => todo!(),
            // Operator::I8x16Swizzle => todo!(),
            // Operator::I8x16Splat => todo!(),
            // Operator::I16x8Splat => todo!(),
            // Operator::I32x4Splat => todo!(),
            // Operator::I64x2Splat => todo!(),
            // Operator::F32x4Splat => todo!(),
            // Operator::F64x2Splat => todo!(),
            // Operator::I8x16Eq => todo!(),
            // Operator::I8x16Ne => todo!(),
            // Operator::I8x16LtS => todo!(),
            // Operator::I8x16LtU => todo!(),
            // Operator::I8x16GtS => todo!(),
            // Operator::I8x16GtU => todo!(),
            // Operator::I8x16LeS => todo!(),
            // Operator::I8x16LeU => todo!(),
            // Operator::I8x16GeS => todo!(),
            // Operator::I8x16GeU => todo!(),
            // Operator::I16x8Eq => todo!(),
            // Operator::I16x8Ne => todo!(),
            // Operator::I16x8LtS => todo!(),
            // Operator::I16x8LtU => todo!(),
            // Operator::I16x8GtS => todo!(),
            // Operator::I16x8GtU => todo!(),
            // Operator::I16x8LeS => todo!(),
            // Operator::I16x8LeU => todo!(),
            // Operator::I16x8GeS => todo!(),
            // Operator::I16x8GeU => todo!(),
            // Operator::I32x4Eq => todo!(),
            // Operator::I32x4Ne => todo!(),
            // Operator::I32x4LtS => todo!(),
            // Operator::I32x4LtU => todo!(),
            // Operator::I32x4GtS => todo!(),
            // Operator::I32x4GtU => todo!(),
            // Operator::I32x4LeS => todo!(),
            // Operator::I32x4LeU => todo!(),
            // Operator::I32x4GeS => todo!(),
            // Operator::I32x4GeU => todo!(),
            // Operator::I64x2Eq => todo!(),
            // Operator::I64x2Ne => todo!(),
            // Operator::I64x2LtS => todo!(),
            // Operator::I64x2GtS => todo!(),
            // Operator::I64x2LeS => todo!(),
            // Operator::I64x2GeS => todo!(),
            // Operator::F32x4Eq => todo!(),
            // Operator::F32x4Ne => todo!(),
            // Operator::F32x4Lt => todo!(),
            // Operator::F32x4Gt => todo!(),
            // Operator::F32x4Le => todo!(),
            // Operator::F32x4Ge => todo!(),
            // Operator::F64x2Eq => todo!(),
            // Operator::F64x2Ne => todo!(),
            // Operator::F64x2Lt => todo!(),
            // Operator::F64x2Gt => todo!(),
            // Operator::F64x2Le => todo!(),
            // Operator::F64x2Ge => todo!(),
            // Operator::V128Not => todo!(),
            // Operator::V128And => todo!(),
            // Operator::V128AndNot => todo!(),
            // Operator::V128Or => todo!(),
            // Operator::V128Xor => todo!(),
            // Operator::V128Bitselect => todo!(),
            // Operator::V128AnyTrue => todo!(),
            // Operator::I8x16Abs => todo!(),
            // Operator::I8x16Neg => todo!(),
            // Operator::I8x16Popcnt => todo!(),
            // Operator::I8x16AllTrue => todo!(),
            // Operator::I8x16Bitmask => todo!(),
            // Operator::I8x16NarrowI16x8S => todo!(),
            // Operator::I8x16NarrowI16x8U => todo!(),
            // Operator::I8x16Shl => todo!(),
            // Operator::I8x16ShrS => todo!(),
            // Operator::I8x16ShrU => todo!(),
            // Operator::I8x16Add => todo!(),
            // Operator::I8x16AddSatS => todo!(),
            // Operator::I8x16AddSatU => todo!(),
            // Operator::I8x16Sub => todo!(),
            // Operator::I8x16SubSatS => todo!(),
            // Operator::I8x16SubSatU => todo!(),
            // Operator::I8x16MinS => todo!(),
            // Operator::I8x16MinU => todo!(),
            // Operator::I8x16MaxS => todo!(),
            // Operator::I8x16MaxU => todo!(),
            // Operator::I8x16RoundingAverageU => todo!(),
            // Operator::I16x8ExtAddPairwiseI8x16S => todo!(),
            // Operator::I16x8ExtAddPairwiseI8x16U => todo!(),
            // Operator::I16x8Abs => todo!(),
            // Operator::I16x8Neg => todo!(),
            // Operator::I16x8Q15MulrSatS => todo!(),
            // Operator::I16x8AllTrue => todo!(),
            // Operator::I16x8Bitmask => todo!(),
            // Operator::I16x8NarrowI32x4S => todo!(),
            // Operator::I16x8NarrowI32x4U => todo!(),
            // Operator::I16x8ExtendLowI8x16S => todo!(),
            // Operator::I16x8ExtendHighI8x16S => todo!(),
            // Operator::I16x8ExtendLowI8x16U => todo!(),
            // Operator::I16x8ExtendHighI8x16U => todo!(),
            // Operator::I16x8Shl => todo!(),
            // Operator::I16x8ShrS => todo!(),
            // Operator::I16x8ShrU => todo!(),
            // Operator::I16x8Add => todo!(),
            // Operator::I16x8AddSatS => todo!(),
            // Operator::I16x8AddSatU => todo!(),
            // Operator::I16x8Sub => todo!(),
            // Operator::I16x8SubSatS => todo!(),
            // Operator::I16x8SubSatU => todo!(),
            // Operator::I16x8Mul => todo!(),
            // Operator::I16x8MinS => todo!(),
            // Operator::I16x8MinU => todo!(),
            // Operator::I16x8MaxS => todo!(),
            // Operator::I16x8MaxU => todo!(),
            // Operator::I16x8RoundingAverageU => todo!(),
            // Operator::I16x8ExtMulLowI8x16S => todo!(),
            // Operator::I16x8ExtMulHighI8x16S => todo!(),
            // Operator::I16x8ExtMulLowI8x16U => todo!(),
            // Operator::I16x8ExtMulHighI8x16U => todo!(),
            // Operator::I32x4ExtAddPairwiseI16x8S => todo!(),
            // Operator::I32x4ExtAddPairwiseI16x8U => todo!(),
            // Operator::I32x4Abs => todo!(),
            // Operator::I32x4Neg => todo!(),
            // Operator::I32x4AllTrue => todo!(),
            // Operator::I32x4Bitmask => todo!(),
            // Operator::I32x4ExtendLowI16x8S => todo!(),
            // Operator::I32x4ExtendHighI16x8S => todo!(),
            // Operator::I32x4ExtendLowI16x8U => todo!(),
            // Operator::I32x4ExtendHighI16x8U => todo!(),
            // Operator::I32x4Shl => todo!(),
            // Operator::I32x4ShrS => todo!(),
            // Operator::I32x4ShrU => todo!(),
            // Operator::I32x4Add => todo!(),
            // Operator::I32x4Sub => todo!(),
            // Operator::I32x4Mul => todo!(),
            // Operator::I32x4MinS => todo!(),
            // Operator::I32x4MinU => todo!(),
            // Operator::I32x4MaxS => todo!(),
            // Operator::I32x4MaxU => todo!(),
            // Operator::I32x4DotI16x8S => todo!(),
            // Operator::I32x4ExtMulLowI16x8S => todo!(),
            // Operator::I32x4ExtMulHighI16x8S => todo!(),
            // Operator::I32x4ExtMulLowI16x8U => todo!(),
            // Operator::I32x4ExtMulHighI16x8U => todo!(),
            // Operator::I64x2Abs => todo!(),
            // Operator::I64x2Neg => todo!(),
            // Operator::I64x2AllTrue => todo!(),
            // Operator::I64x2Bitmask => todo!(),
            // Operator::I64x2ExtendLowI32x4S => todo!(),
            // Operator::I64x2ExtendHighI32x4S => todo!(),
            // Operator::I64x2ExtendLowI32x4U => todo!(),
            // Operator::I64x2ExtendHighI32x4U => todo!(),
            // Operator::I64x2Shl => todo!(),
            // Operator::I64x2ShrS => todo!(),
            // Operator::I64x2ShrU => todo!(),
            // Operator::I64x2Add => todo!(),
            // Operator::I64x2Sub => todo!(),
            // Operator::I64x2Mul => todo!(),
            // Operator::I64x2ExtMulLowI32x4S => todo!(),
            // Operator::I64x2ExtMulHighI32x4S => todo!(),
            // Operator::I64x2ExtMulLowI32x4U => todo!(),
            // Operator::I64x2ExtMulHighI32x4U => todo!(),
            // Operator::F32x4Ceil => todo!(),
            // Operator::F32x4Floor => todo!(),
            // Operator::F32x4Trunc => todo!(),
            // Operator::F32x4Nearest => todo!(),
            // Operator::F32x4Abs => todo!(),
            // Operator::F32x4Neg => todo!(),
            // Operator::F32x4Sqrt => todo!(),
            // Operator::F32x4Add => todo!(),
            // Operator::F32x4Sub => todo!(),
            // Operator::F32x4Mul => todo!(),
            // Operator::F32x4Div => todo!(),
            // Operator::F32x4Min => todo!(),
            // Operator::F32x4Max => todo!(),
            // Operator::F32x4PMin => todo!(),
            // Operator::F32x4PMax => todo!(),
            // Operator::F64x2Ceil => todo!(),
            // Operator::F64x2Floor => todo!(),
            // Operator::F64x2Trunc => todo!(),
            // Operator::F64x2Nearest => todo!(),
            // Operator::F64x2Abs => todo!(),
            // Operator::F64x2Neg => todo!(),
            // Operator::F64x2Sqrt => todo!(),
            // Operator::F64x2Add => todo!(),
            // Operator::F64x2Sub => todo!(),
            // Operator::F64x2Mul => todo!(),
            // Operator::F64x2Div => todo!(),
            // Operator::F64x2Min => todo!(),
            // Operator::F64x2Max => todo!(),
            // Operator::F64x2PMin => todo!(),
            // Operator::F64x2PMax => todo!(),
            // Operator::I32x4TruncSatF32x4S => todo!(),
            // Operator::I32x4TruncSatF32x4U => todo!(),
            // Operator::F32x4ConvertI32x4S => todo!(),
            // Operator::F32x4ConvertI32x4U => todo!(),
            // Operator::I32x4TruncSatF64x2SZero => todo!(),
            // Operator::I32x4TruncSatF64x2UZero => todo!(),
            // Operator::F64x2ConvertLowI32x4S => todo!(),
            // Operator::F64x2ConvertLowI32x4U => todo!(),
            // Operator::F32x4DemoteF64x2Zero => todo!(),
            // Operator::F64x2PromoteLowF32x4 => todo!(),
            // Operator::I8x16SwizzleRelaxed => todo!(),
            // Operator::I32x4TruncSatF32x4SRelaxed => todo!(),
            // Operator::I32x4TruncSatF32x4URelaxed => todo!(),
            // Operator::I32x4TruncSatF64x2SZeroRelaxed => todo!(),
            // Operator::I32x4TruncSatF64x2UZeroRelaxed => todo!(),
            // Operator::F32x4FmaRelaxed => todo!(),
            // Operator::F32x4FmsRelaxed => todo!(),
            // Operator::F64x2FmaRelaxed => todo!(),
            // Operator::F64x2FmsRelaxed => todo!(),
            // Operator::I8x16LaneSelect => todo!(),
            // Operator::I16x8LaneSelect => todo!(),
            // Operator::I32x4LaneSelect => todo!(),
            // Operator::I64x2LaneSelect => todo!(),
            // Operator::F32x4MinRelaxed => todo!(),
            // Operator::F32x4MaxRelaxed => todo!(),
            // Operator::F64x2MinRelaxed => todo!(),
            // Operator::F64x2MaxRelaxed => todo!(),
            _ => {}
        };

        Ok(())
    }
}

impl Debug for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Control::If { .. } => f.debug_struct("If").finish(),
            Control::Loop { .. } => f.debug_struct("Loop").finish(),
            Control::Block { .. } => f.debug_struct("Block").finish(),
        }
    }
}
