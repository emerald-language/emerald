use crate::backend::factory::{BackendFactory, BackendType, BackendError};
use crate::backend::ports::{CodeGen, Emitter, Optimizer};
use crate::backend::llvm::codegen::LlvmCodeGen;
use crate::backend::llvm::optimizer::LlvmOptimizer;
use crate::backend::llvm::emitter::LlvmEmitter;

/// LLVM backend factory
pub struct LlvmBackendFactory;

impl BackendFactory for LlvmBackendFactory {
    fn create_codegen(&self) -> Result<Box<dyn CodeGen>, BackendError> {
        Ok(Box::new(LlvmCodeGen::new()))
    }
    
    fn create_optimizer(&self) -> Result<Box<dyn Optimizer>, BackendError> {
        Ok(Box::new(LlvmOptimizer::new()))
    }
    
    fn create_emitter(&self) -> Result<Box<dyn Emitter>, BackendError> {
        Ok(Box::new(LlvmEmitter::new()))
    }
    
    fn backend_type(&self) -> BackendType {
        BackendType::Llvm
    }
}
