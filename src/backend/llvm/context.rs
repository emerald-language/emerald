use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use std::ffi::CString;
use std::sync::Once;

static LLVM_INIT: Once = Once::new();

/// initialize LLVM (thread-safe, idempotent)
pub fn initialize_llvm() {
    LLVM_INIT.call_once(|| {
        unsafe {
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();
            LLVM_InitializeNativeAsmParser();
        }
    });
}

/// LLVM context wrapper
pub struct LlvmContext {
    context: LLVMContextRef,
}

impl LlvmContext {
    pub fn new() -> Self {
        initialize_llvm();
        unsafe {
            let context = LLVMContextCreate();
            Self { context }
        }
    }

    pub fn get(&self) -> LLVMContextRef {
        self.context
    }
}

impl Default for LlvmContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LlvmContext {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.context);
        }
    }
}

/// create a module name as CString
pub fn create_module_name(name: &str) -> CString {
    CString::new(name).expect("Module name contains null byte")
}
