use crate::core::mir::MirFunction;
use crate::core::hir::Hir;
use thiserror::Error;

/// represents a compiled module
/// stores backend-specific module data
pub struct Module {
    pub name: String,
    // backend-specific data stored as Any for type erasure
    pub data: Option<Box<dyn std::any::Any + Send + Sync>>,
}

impl Module {
    pub fn new(name: String) -> Self {
        Self {
            name,
            data: None,
        }
    }
    
    pub fn with_data(name: String, data: Box<dyn std::any::Any + Send + Sync>) -> Self {
        Self {
            name,
            data: Some(data),
        }
    }
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("name", &self.name)
            .field("data", &"<backend-specific>")
            .finish()
    }
}

impl Clone for Module {
    fn clone(&self) -> Self {
        // cloning module without data (backend-specific data can't be cloned generically)
        // for LLVM modules, this means the clone won't have the module reference
        Self {
            name: self.name.clone(),
            data: None,
        }
    }
}

// Module disposal is handled by the backend-specific wrapper types (e.g., LlvmModuleWrapper)

/// backend input type - some backends use HIR others use MIR
#[derive(Debug, Clone)]
pub enum BackendInput {
    Hir(Vec<Hir>),
    Mir(Vec<MirFunction>),
}

/// trait 4 code generation - supports both HIR and MIR
pub trait CodeGen {
    /// gen code from HIR (for HIR-based backends)
    fn generate_from_hir(&mut self, _hir: &[Hir]) -> Result<Module, CodeGenError> {
        Err(CodeGenError::UnsupportedFeature(
            "This backend does not support HIR input".to_string()
        ))
    }
    
    /// gen code from MIR (for MIR-based backends)
    fn generate_from_mir(&mut self, _mir: &[MirFunction]) -> Result<Module, CodeGenError> {
        Err(CodeGenError::UnsupportedFeature(
            "This backend does not support MIR input".to_string()
        ))
    }
    
    /// gen code - auto-selects HIR or MIR based on backend preference
    fn generate(&mut self, input: BackendInput) -> Result<Module, CodeGenError> {
        match input {
            BackendInput::Hir(hir) => self.generate_from_hir(&hir),
            BackendInput::Mir(mir) => self.generate_from_mir(&mir),
        }
    }
    
    /// set optimization lvl
    fn set_optimization_level(&mut self, level: OptimizationLevel);
    
    /// set target trpl
    fn set_target_triple(&mut self, triple: String);
    
    /// get preferred input type (HIR or MIR)
    fn preferred_input(&self) -> BackendInputType;
}

/// backend input type preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendInputType {
    Hir,
    Mir,
}

#[derive(Debug, Error)]
pub enum CodeGenError {
    #[error("Code generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Invalid target triple: {0}")]
    InvalidTarget(String),
    
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Default,
    Aggressive,
    Size,
    SizePerformance,
}

impl OptimizationLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "0" => Some(Self::None),
            "1" => Some(Self::Basic),
            "2" => Some(Self::Default),
            "3" => Some(Self::Aggressive),
            "s" | "size" => Some(Self::Size),
            "z" | "zsize" => Some(Self::SizePerformance),
            _ => None,
        }
    }
}
