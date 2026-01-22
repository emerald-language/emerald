use crate::core::types::ty::Type;
use crate::core::types::primitive::PrimitiveType;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMTypeKind;

/// convert MIR type to LLVM type
pub fn mir_type_to_llvm_type(context: LLVMContextRef, ty: &Type) -> LLVMTypeRef {
    unsafe {
        match ty {
            Type::Primitive(p) => primitive_to_llvm_type(context, p),
            Type::Pointer(ptr) => {
                let pointee = mir_type_to_llvm_type(context, &ptr.pointee);
                LLVMPointerType(pointee, 0) // addr space 0
            }
            Type::Array(arr) => {
                let element = mir_type_to_llvm_type(context, &arr.element);
                LLVMArrayType2(element, arr.size as u64)
            }
            Type::Struct(s) => {
                // create struct type - for now use opaque struct
                // TODO: properly handle struct fields
                let name = format!("struct.{}", s.name);
                let name_cstr = std::ffi::CString::new(name).unwrap();
                LLVMStructCreateNamed(context, name_cstr.as_ptr())
            }
            Type::Function(func) => {
                let ret_type = mir_type_to_llvm_type(context, &func.return_type);
                
                let mut param_types: Vec<LLVMTypeRef> = func.params.iter()
                    .map(|p| mir_type_to_llvm_type(context, p))
                    .collect();
                
                if param_types.is_empty() {
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
                }
            }
            Type::String => {
                // string is (ptr, len) - for now just use i8*
                LLVMPointerType(LLVMInt8TypeInContext(context), 0)
            }
            Type::TraitObject(_) => {
                // trait object is (data_ptr, vtable_ptr) - use i8* for now
                LLVMPointerType(LLVMInt8TypeInContext(context), 0)
            }
            Type::Generic(_) => {
                // generic types should be monomorphized before reaching backend
                // use i8* as fallback
                LLVMPointerType(LLVMInt8TypeInContext(context), 0)
            }
        }
    }
}

/// convert primitive type to LLVM type
fn primitive_to_llvm_type(context: LLVMContextRef, p: &PrimitiveType) -> LLVMTypeRef {
    unsafe {
        match p {
            PrimitiveType::Void => LLVMVoidType(),
            PrimitiveType::Byte => LLVMInt8TypeInContext(context),
            PrimitiveType::Int => LLVMInt32TypeInContext(context),
            PrimitiveType::Long => LLVMInt64TypeInContext(context),
            PrimitiveType::Size => {
                // size_t is platform-dependent, use u64 for 64-bit
                LLVMInt64TypeInContext(context)
            }
            PrimitiveType::Float => LLVMDoubleTypeInContext(context),
            PrimitiveType::Bool => LLVMInt1TypeInContext(context),
            PrimitiveType::Char => LLVMInt32TypeInContext(context), // char32_t
        }
    }
}

/// get LLVM type kind (for debugging)
pub fn get_type_kind(ty: LLVMTypeRef) -> LLVMTypeKind {
    unsafe { LLVMGetTypeKind(ty) }
}
