use crate::backend::ports::optimizer::{Optimizer, OptimizationError};
use crate::backend::ports::codegen::Module;
use crate::backend::ports::codegen::OptimizationLevel;
use llvm_sys::core::*;

/// LLVM optimizer - applies LLVM optimization passes
pub struct LlvmOptimizer {
    opt_level: OptimizationLevel,
}

impl LlvmOptimizer {
    pub fn new() -> Self {
        Self {
            opt_level: OptimizationLevel::Default,
        }
    }
}

impl Optimizer for LlvmOptimizer {
    fn optimize(&mut self, module: &mut Module) -> Result<(), OptimizationError> {
        unsafe {
            // get LLVM module from module data
            use crate::backend::llvm::codegen::LlvmModuleWrapper;
            let llvm_module = module.data.as_ref()
                .and_then(|d| d.downcast_ref::<LlvmModuleWrapper>())
                .map(|w| w.get())
                .ok_or_else(|| OptimizationError::OptimizationFailed(
                    "Module does not contain LLVM module".to_string()
                ))?;

            // create function pass manager
            let fpm = LLVMCreateFunctionPassManagerForModule(llvm_module);
            
            // Note: In LLVM 21, the pass manager builder API may have changed
            // For now, we'll use a simplified approach - just initialize and run
            // TODO: Add proper optimization passes when API is available
            LLVMInitializeFunctionPassManager(fpm);
            
            // run passes on all functions
            let mut func = LLVMGetFirstFunction(llvm_module);
            while !func.is_null() {
                LLVMRunFunctionPassManager(fpm, func);
                func = LLVMGetNextFunction(func);
            }
            
            LLVMFinalizeFunctionPassManager(fpm);
            LLVMDisposePassManager(fpm);

            Ok(())
        }
    }

    fn add_pass(&mut self, _pass: crate::backend::ports::optimizer::OptimizationPass) {
        // custom passes can be added here if needed
    }
}

impl Default for LlvmOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
