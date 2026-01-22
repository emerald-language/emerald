use crate::core::mir::instruction::Instruction;
use crate::core::mir::operand::{Operand, Local, Constant};
use crate::backend::llvm::types::mir_type_to_llvm_type;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

/// helper to convert MIR operand to LLVM value
pub fn operand_to_llvm_value(
    context: LLVMContextRef,
    operand: &Operand,
    local_map: &std::collections::HashMap<usize, LLVMValueRef>,
) -> LLVMValueRef {
    match operand {
        Operand::Constant(c) => constant_to_llvm_value(context, c),
        Operand::Local(local) => {
            *local_map.get(&local.id).expect("Local not found in map")
        }
        Operand::Function(_func_ref) => {
            // function reference - will be resolved during function translation
            // for now return null, should be handled at function level
            std::ptr::null_mut()
        }
    }
}

/// convert constant to LLVM value
fn constant_to_llvm_value(context: LLVMContextRef, constant: &Constant) -> LLVMValueRef {
    unsafe {
        match constant {
            Constant::Int(i) => {
                let ty = LLVMInt32TypeInContext(context);
                LLVMConstInt(ty, *i as u64, 0)
            }
            Constant::Float(f) => {
                let ty = LLVMDoubleTypeInContext(context);
                LLVMConstReal(ty, *f)
            }
            Constant::Bool(b) => {
                let ty = LLVMInt1TypeInContext(context);
                LLVMConstInt(ty, if *b { 1 } else { 0 }, 0)
            }
            Constant::Char(c) => {
                let ty = LLVMInt32TypeInContext(context);
                LLVMConstInt(ty, *c as u64, 0)
            }
            Constant::String(s) => {
                let cstr = std::ffi::CString::new(s.as_str()).unwrap();
                LLVMConstStringInContext2(context, cstr.as_ptr(), s.len(), 0)
            }
            Constant::Null => {
                let ty = LLVMPointerType(LLVMInt8TypeInContext(context), 0);
                LLVMConstNull(ty)
            }
        }
    }
}

/// translate arithmetic instruction
pub fn translate_arithmetic(
    builder: LLVMBuilderRef,
    inst: &Instruction,
    local_map: &mut std::collections::HashMap<usize, LLVMValueRef>,
    context: LLVMContextRef,
) -> Option<LLVMValueRef> {
    unsafe {
        match inst {
            Instruction::Add { dest, left, right, type_: _ } => {
                let left_val = operand_to_llvm_value(context, left, local_map);
                let right_val = operand_to_llvm_value(context, right, local_map);
                let result = LLVMBuildAdd(builder, left_val, right_val, b"add\0".as_ptr() as *const i8);
                local_map.insert(dest.id, result);
                Some(result)
            }
            Instruction::Sub { dest, left, right, type_: _ } => {
                let left_val = operand_to_llvm_value(context, left, local_map);
                let right_val = operand_to_llvm_value(context, right, local_map);
                let result = LLVMBuildSub(builder, left_val, right_val, b"sub\0".as_ptr() as *const i8);
                local_map.insert(dest.id, result);
                Some(result)
            }
            Instruction::Mul { dest, left, right, type_: _ } => {
                let left_val = operand_to_llvm_value(context, left, local_map);
                let right_val = operand_to_llvm_value(context, right, local_map);
                let result = LLVMBuildMul(builder, left_val, right_val, b"mul\0".as_ptr() as *const i8);
                local_map.insert(dest.id, result);
                Some(result)
            }
            Instruction::Div { dest, left, right, type_: _ } => {
                let left_val = operand_to_llvm_value(context, left, local_map);
                let right_val = operand_to_llvm_value(context, right, local_map);
                // check if signed or unsigned - default to signed
                let result = LLVMBuildSDiv(builder, left_val, right_val, b"div\0".as_ptr() as *const i8);
                local_map.insert(dest.id, result);
                Some(result)
            }
            Instruction::Mod { dest, left, right, type_: _ } => {
                let left_val = operand_to_llvm_value(context, left, local_map);
                let right_val = operand_to_llvm_value(context, right, local_map);
                let result = LLVMBuildSRem(builder, left_val, right_val, b"mod\0".as_ptr() as *const i8);
                local_map.insert(dest.id, result);
                Some(result)
            }
            _ => None,
        }
    }
}

/// translate comparison instruction
pub fn translate_comparison(
    builder: LLVMBuilderRef,
    inst: &Instruction,
    local_map: &mut std::collections::HashMap<usize, LLVMValueRef>,
    context: LLVMContextRef,
) -> Option<LLVMValueRef> {
    unsafe {
        let (left, right) = match inst {
            Instruction::Eq { left, right, .. } |
            Instruction::Ne { left, right, .. } |
            Instruction::Lt { left, right, .. } |
            Instruction::Le { left, right, .. } |
            Instruction::Gt { left, right, .. } |
            Instruction::Ge { left, right, .. } => {
                (operand_to_llvm_value(context, left, local_map),
                 operand_to_llvm_value(context, right, local_map))
            }
            _ => return None,
        };

        let result = match inst {
            Instruction::Eq { .. } => {
                LLVMBuildICmp(builder, llvm_sys::LLVMIntPredicate::LLVMIntEQ, left, right, b"eq\0".as_ptr() as *const i8)
            }
            Instruction::Ne { .. } => {
                LLVMBuildICmp(builder, llvm_sys::LLVMIntPredicate::LLVMIntNE, left, right, b"ne\0".as_ptr() as *const i8)
            }
            Instruction::Lt { .. } => {
                LLVMBuildICmp(builder, llvm_sys::LLVMIntPredicate::LLVMIntSLT, left, right, b"lt\0".as_ptr() as *const i8)
            }
            Instruction::Le { .. } => {
                LLVMBuildICmp(builder, llvm_sys::LLVMIntPredicate::LLVMIntSLE, left, right, b"le\0".as_ptr() as *const i8)
            }
            Instruction::Gt { .. } => {
                LLVMBuildICmp(builder, llvm_sys::LLVMIntPredicate::LLVMIntSGT, left, right, b"gt\0".as_ptr() as *const i8)
            }
            Instruction::Ge { .. } => {
                LLVMBuildICmp(builder, llvm_sys::LLVMIntPredicate::LLVMIntSGE, left, right, b"ge\0".as_ptr() as *const i8)
            }
            _ => return None,
        };

        if let Some(dest) = get_dest_local(inst) {
            local_map.insert(dest.id, result);
        }
        Some(result)
    }
}

/// translate memory instruction
pub fn translate_memory(
    builder: LLVMBuilderRef,
    inst: &Instruction,
    local_map: &mut std::collections::HashMap<usize, LLVMValueRef>,
    context: LLVMContextRef,
) -> Option<LLVMValueRef> {
    unsafe {
        match inst {
            Instruction::Load { dest, source, type_ } => {
                let ptr = operand_to_llvm_value(context, source, local_map);
                let ty = mir_type_to_llvm_type(context, type_);
                let result = LLVMBuildLoad2(builder, ty, ptr, b"load\0".as_ptr() as *const i8);
                local_map.insert(dest.id, result);
                Some(result)
            }
            Instruction::Store { dest, source, type_: _type_ } => {
                let ptr = operand_to_llvm_value(context, dest, local_map);
                let val = operand_to_llvm_value(context, source, local_map);
                LLVMBuildStore(builder, val, ptr);
                None
            }
            Instruction::Alloca { dest, type_ } => {
                let ty = mir_type_to_llvm_type(context, type_);
                let result = LLVMBuildAlloca(builder, ty, b"alloca\0".as_ptr() as *const i8);
                local_map.insert(dest.id, result);
                Some(result)
            }
            Instruction::Gep { dest, base, index, type_ } => {
                let base_ptr = operand_to_llvm_value(context, base, local_map);
                let idx = operand_to_llvm_value(context, index, local_map);
                let ty = mir_type_to_llvm_type(context, type_);
                let mut indices = [idx];
                let result = LLVMBuildGEP2(builder, ty, base_ptr, indices.as_mut_ptr(), indices.len() as u32, b"gep\0".as_ptr() as *const i8);
                local_map.insert(dest.id, result);
                Some(result)
            }
            _ => None,
        }
    }
}

/// translate control flow instruction
pub fn translate_control_flow(
    builder: LLVMBuilderRef,
    inst: &Instruction,
    local_map: &std::collections::HashMap<usize, LLVMValueRef>,
    bb_map: &std::collections::HashMap<usize, LLVMBasicBlockRef>,
    context: LLVMContextRef,
) -> bool {
    unsafe {
        match inst {
            Instruction::Ret { value } => {
                if let Some(val) = value {
                    let ret_val = operand_to_llvm_value(context, val, local_map);
                    LLVMBuildRet(builder, ret_val);
                } else {
                    LLVMBuildRetVoid(builder);
                }
                true // is terminator
            }
            Instruction::Jump { target } => {
                if let Some(target_bb) = bb_map.get(target) {
                    LLVMBuildBr(builder, *target_bb);
                }
                true // is terminator
            }
            Instruction::Br { condition, then_bb, else_bb } => {
                let cond = operand_to_llvm_value(context, condition, local_map);
                let then_block = bb_map.get(then_bb).copied();
                let else_block = bb_map.get(else_bb).copied();
                if let (Some(then_bb), Some(else_bb)) = (then_block, else_block) {
                    LLVMBuildCondBr(builder, cond, then_bb, else_bb);
                }
                true // is terminator
            }
            _ => false,
        }
    }
}

/// get destination local from instruction
fn get_dest_local(inst: &Instruction) -> Option<&Local> {
    match inst {
        Instruction::Add { dest, .. } |
        Instruction::Sub { dest, .. } |
        Instruction::Mul { dest, .. } |
        Instruction::Div { dest, .. } |
        Instruction::Mod { dest, .. } |
        Instruction::Eq { dest, .. } |
        Instruction::Ne { dest, .. } |
        Instruction::Lt { dest, .. } |
        Instruction::Le { dest, .. } |
        Instruction::Gt { dest, .. } |
        Instruction::Ge { dest, .. } |
        Instruction::And { dest, .. } |
        Instruction::Or { dest, .. } |
        Instruction::Not { dest, .. } |
        Instruction::Load { dest, .. } |
        Instruction::Alloca { dest, .. } |
        Instruction::Gep { dest, .. } |
        Instruction::Call { dest: Some(dest), .. } |
        Instruction::Phi { dest, .. } |
        Instruction::Copy { dest, .. } => Some(dest),
        _ => None,
    }
}
