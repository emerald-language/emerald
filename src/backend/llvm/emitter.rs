use crate::backend::ports::emitter::{Emitter, EmitError};
use crate::backend::ports::codegen::Module;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;
use std::ffi::CString;
use std::fs;
use std::path::Path;

/// LLVM emitter - emits various output formats
pub struct LlvmEmitter;

impl LlvmEmitter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LlvmEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Emitter for LlvmEmitter {
    fn emit_binary(&self, module: &Module, output: &Path) -> Result<(), EmitError> {
        unsafe {
            let llvm_module = self.get_llvm_module(module)?;
            
            // initialize target
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();
            
            // get target triple - use default or from module data layout
            // In LLVM 21, we need to get the triple differently
            // For now, use the default target triple
            let triple = "x86_64-unknown-linux-gnu"; // Default, can be overridden
            let triple_cstr = CString::new(triple).unwrap();
            
            // create target machine - LLVMGetTargetFromTriple takes target as out parameter
            let mut target: LLVMTargetRef = std::ptr::null_mut();
            let mut error_msg = std::ptr::null_mut();
            let target_result = LLVMGetTargetFromTriple(triple_cstr.as_ptr(), &mut target, &mut error_msg);
            if target_result != 0 || target.is_null() {
                let error = if !error_msg.is_null() {
                    std::ffi::CStr::from_ptr(error_msg).to_string_lossy().to_string()
                } else {
                    format!("Failed to get target for triple: {}", triple)
                };
                LLVMDisposeMessage(error_msg);
                return Err(EmitError::EmissionFailed(error));
            }
            
            // create target machine (use default CPU and features)
            let cpu_cstr = CString::new("").unwrap();
            let features_cstr = CString::new("").unwrap();
            let target_machine = LLVMCreateTargetMachine(
                target,
                triple_cstr.as_ptr(),
                cpu_cstr.as_ptr(),
                features_cstr.as_ptr(),
                LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                LLVMRelocMode::LLVMRelocDefault,
                LLVMCodeModel::LLVMCodeModelDefault,
            );
            
            // emit object file first
            let obj_path = output.with_extension("o");
            let obj_path_cstr = CString::new(obj_path.to_string_lossy().as_ref()).unwrap();
            let mut error_msg = std::ptr::null_mut();
            
            if LLVMTargetMachineEmitToFile(
                target_machine,
                llvm_module,
                obj_path_cstr.as_ptr(),
                LLVMCodeGenFileType::LLVMObjectFile,
                &mut error_msg,
            ) != 0 {
                let error = if !error_msg.is_null() {
                    std::ffi::CStr::from_ptr(error_msg).to_string_lossy().to_string()
                } else {
                    "Unknown error".to_string()
                };
                LLVMDisposeMessage(error_msg);
                return Err(EmitError::EmissionFailed(error));
            }
            
            // link object file to binary (simplified - in production would use proper linker)
            // for now, just copy object file as binary (this is a placeholder)
            // TODO: use proper linker (lld or system linker)
            fs::copy(&obj_path, output)?;
            
            LLVMDisposeTargetMachine(target_machine);
            
            Ok(())
        }
    }

    fn emit_assembly(&self, module: &Module, output: &Path) -> Result<(), EmitError> {
        unsafe {
            let llvm_module = self.get_llvm_module(module)?;
            
            // initialize target
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();
            
            // get target triple - use default
            let triple = "x86_64-unknown-linux-gnu";
            let triple_cstr = CString::new(triple).unwrap();
            
            // create target machine
            let mut target: LLVMTargetRef = std::ptr::null_mut();
            let mut error_msg = std::ptr::null_mut();
            let target_result = LLVMGetTargetFromTriple(triple_cstr.as_ptr(), &mut target, &mut error_msg);
            if target_result != 0 || target.is_null() {
                let error = if !error_msg.is_null() {
                    std::ffi::CStr::from_ptr(error_msg).to_string_lossy().to_string()
                } else {
                    "Failed to get target".to_string()
                };
                LLVMDisposeMessage(error_msg);
                return Err(EmitError::EmissionFailed(error));
            }
            
            let cpu_cstr = CString::new("").unwrap();
            let features_cstr = CString::new("").unwrap();
            let target_machine = LLVMCreateTargetMachine(
                target,
                triple_cstr.as_ptr(),
                cpu_cstr.as_ptr(),
                features_cstr.as_ptr(),
                LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                LLVMRelocMode::LLVMRelocDefault,
                LLVMCodeModel::LLVMCodeModelDefault,
            );
            
            let output_cstr = CString::new(output.to_string_lossy().as_ref()).unwrap();
            let mut error_msg = std::ptr::null_mut();
            
            if LLVMTargetMachineEmitToFile(
                target_machine,
                llvm_module,
                output_cstr.as_ptr(),
                LLVMCodeGenFileType::LLVMAssemblyFile,
                &mut error_msg,
            ) != 0 {
                let error = if !error_msg.is_null() {
                    std::ffi::CStr::from_ptr(error_msg).to_string_lossy().to_string()
                } else {
                    "Unknown error".to_string()
                };
                LLVMDisposeMessage(error_msg);
                LLVMDisposeTargetMachine(target_machine);
                return Err(EmitError::EmissionFailed(error));
            }
            
            LLVMDisposeTargetMachine(target_machine);
            
            Ok(())
        }
    }

    fn emit_llvm_ir(&self, module: &Module, output: &Path) -> Result<(), EmitError> {
        unsafe {
            let llvm_module = self.get_llvm_module(module)?;
            
            // print LLVM IR to string
            let ir_cstr = LLVMPrintModuleToString(llvm_module);
            if ir_cstr.is_null() {
                return Err(EmitError::EmissionFailed("Failed to generate LLVM IR".to_string()));
            }
            
            let ir_string = std::ffi::CStr::from_ptr(ir_cstr).to_string_lossy();
            fs::write(output, ir_string.as_ref())?;
            
            LLVMDisposeMessage(ir_cstr);
            
            Ok(())
        }
    }

    fn emit_object(&self, module: &Module, output: &Path) -> Result<(), EmitError> {
        unsafe {
            let llvm_module = self.get_llvm_module(module)?;
            
            // initialize target
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();
            
            // get target triple - use default
            let triple = "x86_64-unknown-linux-gnu";
            let triple_cstr = CString::new(triple).unwrap();
            
            // create target machine
            let mut target: LLVMTargetRef = std::ptr::null_mut();
            let mut error_msg = std::ptr::null_mut();
            let target_result = LLVMGetTargetFromTriple(triple_cstr.as_ptr(), &mut target, &mut error_msg);
            if target_result != 0 || target.is_null() {
                let error = if !error_msg.is_null() {
                    std::ffi::CStr::from_ptr(error_msg).to_string_lossy().to_string()
                } else {
                    "Failed to get target".to_string()
                };
                LLVMDisposeMessage(error_msg);
                return Err(EmitError::EmissionFailed(error));
            }
            
            let cpu_cstr = CString::new("").unwrap();
            let features_cstr = CString::new("").unwrap();
            let target_machine = LLVMCreateTargetMachine(
                target,
                triple_cstr.as_ptr(),
                cpu_cstr.as_ptr(),
                features_cstr.as_ptr(),
                LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                LLVMRelocMode::LLVMRelocDefault,
                LLVMCodeModel::LLVMCodeModelDefault,
            );
            
            let output_cstr = CString::new(output.to_string_lossy().as_ref()).unwrap();
            let mut error_msg = std::ptr::null_mut();
            
            if LLVMTargetMachineEmitToFile(
                target_machine,
                llvm_module,
                output_cstr.as_ptr(),
                LLVMCodeGenFileType::LLVMObjectFile,
                &mut error_msg,
            ) != 0 {
                let error = if !error_msg.is_null() {
                    std::ffi::CStr::from_ptr(error_msg).to_string_lossy().to_string()
                } else {
                    "Unknown error".to_string()
                };
                LLVMDisposeMessage(error_msg);
                LLVMDisposeTargetMachine(target_machine);
                return Err(EmitError::EmissionFailed(error));
            }
            
            LLVMDisposeTargetMachine(target_machine);
            
            Ok(())
        }
    }
}

impl LlvmEmitter {
    /// get LLVM module from Module struct
    fn get_llvm_module(&self, module: &Module) -> Result<LLVMModuleRef, EmitError> {
        // get LLVM module from module data
        use crate::backend::llvm::codegen::LlvmModuleWrapper;
        module.data.as_ref()
            .and_then(|d| d.downcast_ref::<LlvmModuleWrapper>())
            .map(|w| w.get())
            .ok_or_else(|| EmitError::EmissionFailed(
                "Module does not contain LLVM module".to_string()
            ))
    }
}
