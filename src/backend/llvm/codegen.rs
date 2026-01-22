use crate::backend::ports::codegen::{CodeGen, CodeGenError, Module, OptimizationLevel, BackendInputType};
use crate::backend::llvm::context::{LlvmContext, create_module_name};
use crate::backend::llvm::types::mir_type_to_llvm_type;
use crate::backend::llvm::instructions::*;
use crate::core::mir::MirFunction;
use crate::core::mir::instruction::Instruction;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::collections::HashMap;
use std::ffi::CString;

/// wrapper for LLVM module that handles disposal
pub(crate) struct LlvmModuleWrapper {
    module: LLVMModuleRef,
}

impl LlvmModuleWrapper {
    pub(crate) fn new(module: LLVMModuleRef) -> Self {
        Self { module }
    }
    
    pub fn get(&self) -> LLVMModuleRef {
        self.module
    }
}

impl Drop for LlvmModuleWrapper {
    fn drop(&mut self) {
        unsafe {
            if !self.module.is_null() {
                LLVMDisposeModule(self.module);
            }
        }
    }
}

unsafe impl Send for LlvmModuleWrapper {}
unsafe impl Sync for LlvmModuleWrapper {}

/// LLVM code generator - translates MIR to LLVM IR
pub struct LlvmCodeGen {
    context: LlvmContext,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    opt_level: OptimizationLevel,
    target_triple: String,
}

impl LlvmCodeGen {
    pub fn new() -> Self {
        let context = LlvmContext::new();
        let module_name = create_module_name("emerald_module");
        unsafe {
            let module = LLVMModuleCreateWithNameInContext(module_name.as_ptr(), context.get());
            let builder = LLVMCreateBuilderInContext(context.get());
            
            Self {
                context,
                module,
                builder,
                opt_level: OptimizationLevel::Default,
                target_triple: Self::default_target_triple(),
            }
        }
    }

    fn default_target_triple() -> String {
        // try to detect target triple, fallback to host
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            "x86_64-unknown-linux-gnu".to_string()
        }
        
        #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
        {
            "x86_64-apple-darwin".to_string()
        }
        
        #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
        {
            "x86_64-pc-windows-msvc".to_string()
        }
        
        #[cfg(not(any(
            all(target_arch = "x86_64", target_os = "linux"),
            all(target_arch = "x86_64", target_os = "macos"),
            all(target_arch = "x86_64", target_os = "windows")
        )))]
        {
            // default fallback
            "x86_64-unknown-linux-gnu".to_string()
        }
    }
}

impl Drop for LlvmCodeGen {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            // only dispose module if it hasn't been moved to Module
            if !self.module.is_null() {
                LLVMDisposeModule(self.module);
            }
        }
    }
}

impl CodeGen for LlvmCodeGen {
    fn generate_from_mir(&mut self, mir_functions: &[MirFunction]) -> Result<Module, CodeGenError> {
        // set target triple - use LLVMSetModuleDataLayout or similar if available
        // Note: LLVMSetTargetTriple might not be available in llvm-sys 211
        // For now, we'll set it via module properties if the function exists
        // If not available, the target will be set during emission

        // translate each MIR function to LLVM function
        for mir_func in mir_functions {
            self.translate_function(mir_func)?;
        }

        // create module wrapper with LLVM module stored
        let module_name = "emerald_module".to_string();
        // wrap LLVM module in a type that handles disposal
        let module_wrapper = LlvmModuleWrapper::new(self.module);
        // don't dispose module in Drop since we're transferring ownership
        // set module to null to prevent double disposal
        self.module = std::ptr::null_mut();
        Ok(Module::with_data(module_name, Box::new(module_wrapper)))
    }

    fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.opt_level = level;
    }

    fn set_target_triple(&mut self, triple: String) {
        self.target_triple = triple;
        // Note: LLVMSetTargetTriple might not be available in llvm-sys 211
        // Target triple will be set during emission
    }

    fn preferred_input(&self) -> BackendInputType {
        BackendInputType::Mir
    }
}

impl LlvmCodeGen {
    /// translate a MIR function to LLVM function
    fn translate_function(&mut self, mir_func: &MirFunction) -> Result<(), CodeGenError> {
        unsafe {
            let context = self.context.get();
            
            // get return type
            let ret_type = mir_func.return_type.as_ref()
                .map(|t| mir_type_to_llvm_type(context, t))
                .unwrap_or_else(|| LLVMVoidType());

            // get parameter types
            let mut param_types: Vec<LLVMTypeRef> = mir_func.params.iter()
                .map(|p| mir_type_to_llvm_type(context, &p.type_))
                .collect();

            // create function type - need mutable pointer
            let func_type = if param_types.is_empty() {
                LLVMFunctionType(
                    ret_type,
                    std::ptr::null_mut(),
                    0,
                    0, // not variadic
                )
            } else {
                LLVMFunctionType(
                    ret_type,
                    param_types.as_mut_ptr(),
                    param_types.len() as u32,
                    0, // not variadic
                )
            };

            // create function
            let func_name = CString::new(mir_func.name.clone()).unwrap();
            let func = LLVMAddFunction(self.module, func_name.as_ptr(), func_type);

            // create basic blocks
            let mut bb_map = HashMap::new();
            for (idx, _bb) in mir_func.basic_blocks.iter().enumerate() {
                let bb_name = format!("bb{}", idx);
                let bb_name_cstr = CString::new(bb_name).unwrap();
                let bb = LLVMAppendBasicBlockInContext(context, func, bb_name_cstr.as_ptr());
                bb_map.insert(idx, bb);
            }

            // translate basic blocks
            let mut local_map = HashMap::new();
            
            // set up parameters
            for (idx, param) in mir_func.params.iter().enumerate() {
                let llvm_param = LLVMGetParam(func, idx as u32);
                local_map.insert(param.local.id, llvm_param);
            }

            // translate each basic block
            for (bb_idx, mir_bb) in mir_func.basic_blocks.iter().enumerate() {
                let llvm_bb = bb_map[&bb_idx];
                LLVMPositionBuilderAtEnd(self.builder, llvm_bb);

                // translate instructions
                for inst in &mir_bb.instructions {
                    self.translate_instruction(inst, &mut local_map, &bb_map, context)?;
                }
            }

            Ok(())
        }
    }

    /// translate a single MIR instruction to LLVM instruction
    fn translate_instruction(
        &mut self,
        inst: &Instruction,
        local_map: &mut HashMap<usize, LLVMValueRef>,
        bb_map: &HashMap<usize, LLVMBasicBlockRef>,
        context: LLVMContextRef,
    ) -> Result<(), CodeGenError> {
        unsafe {
            // try arithmetic first
            if let Some(_) = translate_arithmetic(self.builder, inst, local_map, context) {
                return Ok(());
            }

            // try comparison
            if let Some(_) = translate_comparison(self.builder, inst, local_map, context) {
                return Ok(());
            }

            // try memory
            if let Some(_) = translate_memory(self.builder, inst, local_map, context) {
                return Ok(());
            }

            // try control flow
            if translate_control_flow(self.builder, inst, local_map, bb_map, context) {
                return Ok(());
            }

            // handle other instructions
            match inst {
                Instruction::Call { dest, func: _func, args: _args, return_type: _return_type } => {
                    // TODO: implement function calls
                    if let Some(dest_local) = dest {
                        // placeholder - should resolve function and call it
                        let void_type = LLVMVoidType();
                        local_map.insert(dest_local.id, LLVMConstNull(void_type));
                    }
                }
                Instruction::Phi { dest, type_, incoming } => {
                    let ty = mir_type_to_llvm_type(context, type_);
                    let phi = LLVMBuildPhi(self.builder, ty, b"phi\0".as_ptr() as *const i8);
                    // add incoming values - need mutable arrays
                    if !incoming.is_empty() {
                        let mut values: Vec<LLVMValueRef> = incoming.iter()
                            .map(|(val_op, _)| operand_to_llvm_value(context, val_op, local_map))
                            .collect();
                        let mut blocks: Vec<LLVMBasicBlockRef> = incoming.iter()
                            .map(|(_, bb_idx)| bb_map[bb_idx])
                            .collect();
                        LLVMAddIncoming(
                            phi,
                            values.as_mut_ptr(),
                            blocks.as_mut_ptr(),
                            incoming.len() as u32,
                        );
                    }
                    local_map.insert(dest.id, phi);
                }
                Instruction::Copy { dest, source, type_: _type_ } => {
                    let src_val = operand_to_llvm_value(context, source, local_map);
                    local_map.insert(dest.id, src_val);
                }
                Instruction::And { dest, left, right } => {
                    let left_val = operand_to_llvm_value(context, left, local_map);
                    let right_val = operand_to_llvm_value(context, right, local_map);
                    let result = LLVMBuildAnd(self.builder, left_val, right_val, b"and\0".as_ptr() as *const i8);
                    local_map.insert(dest.id, result);
                }
                Instruction::Or { dest, left, right } => {
                    let left_val = operand_to_llvm_value(context, left, local_map);
                    let right_val = operand_to_llvm_value(context, right, local_map);
                    let result = LLVMBuildOr(self.builder, left_val, right_val, b"or\0".as_ptr() as *const i8);
                    local_map.insert(dest.id, result);
                }
                Instruction::Not { dest, operand } => {
                    let op_val = operand_to_llvm_value(context, operand, local_map);
                    let result = LLVMBuildNot(self.builder, op_val, b"not\0".as_ptr() as *const i8);
                    local_map.insert(dest.id, result);
                }
                _ => {
                    // unhandled instruction - log warning but continue
                }
            }

            Ok(())
        }
    }

    /// get LLVM module (for emitter/optimizer)
    pub fn get_module(&self) -> LLVMModuleRef {
        self.module
    }
}
