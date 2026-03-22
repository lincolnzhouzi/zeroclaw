use std::ffi::{c_char, c_float, c_int, c_void};
use std::os::raw::c_ulonglong;

#[repr(C)]
pub struct MNNInterpreterOpaque {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MNNSessionOpaque {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MNNTensorOpaque {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MNNBackendOpaque {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MNNRuntimeOpaque {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MNNLlmOpaque {
    _private: [u8; 0],
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MNNForwardType {
    CPU = 0,
    Metal = 1,
    OpenCL = 2,
    OpenGL = 3,
    Vulkan = 4,
    CUDA = 6,
    NN = 7,
    Auto = 8,
}

impl Default for MNNForwardType {
    fn default() -> Self {
        Self::Auto
    }
}

impl From<crate::types::MNNBackendType> for MNNForwardType {
    fn from(backend: crate::types::MNNBackendType) -> Self {
        match backend {
            crate::types::MNNBackendType::CPU => Self::CPU,
            crate::types::MNNBackendType::GPU => Self::Auto,
            crate::types::MNNBackendType::NPU => Self::NN,
            crate::types::MNNBackendType::Auto => Self::Auto,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScheduleConfig {
    pub type_: MNNForwardType,
    pub numThread: c_int,
    pub saveTensors: *const *const c_char,
    pub saveTensorCount: c_int,
    pub path: PathConfig,
    pub backupType: MNNForwardType,
    pub backendConfig: *mut BackendConfig,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            type_: MNNForwardType::Auto,
            numThread: 4,
            saveTensors: std::ptr::null(),
            saveTensorCount: 0,
            path: PathConfig::default(),
            backupType: MNNForwardType::CPU,
            backendConfig: std::ptr::null_mut(),
        }
    }
}

impl ScheduleConfig {
    pub fn new(backend_type: MNNForwardType, threads: c_int) -> Self {
        Self {
            type_: backend_type,
            numThread: threads,
            ..Default::default()
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PathConfig {
    pub mode: c_int,
    pub inputs: *const *const c_char,
    pub inputCount: c_int,
    pub outputs: *const *const c_char,
    pub outputCount: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BackendConfig {
    pub precision: c_int,
    pub power: c_int,
    pub memory: c_int,
}

impl BackendConfig {
    pub fn new(precision: Precision, power: Power, memory: Memory) -> Self {
        Self {
            precision: precision as c_int,
            power: power as c_int,
            memory: memory as c_int,
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    Normal = 0,
    High = 1,
    Low = 2,
    LowBF16 = 3,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Power {
    Normal = 0,
    High = 1,
    Low = 2,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Memory {
    Normal = 0,
    High = 1,
    Low = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TensorShape {
    pub dims: *const c_int,
    pub dimCount: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HalideType {
    pub code: c_int,
    pub bits: c_int,
    pub lanes: c_int,
}

impl HalideType {
    pub fn float32() -> Self {
        Self {
            code: 2,
            bits: 32,
            lanes: 1,
        }
    }

    pub fn int32() -> Self {
        Self {
            code: 0,
            bits: 32,
            lanes: 1,
        }
    }

    pub fn int8() -> Self {
        Self {
            code: 0,
            bits: 8,
            lanes: 1,
        }
    }
}

extern "C" {
    pub fn MNN_createInterpreterFromFile(file: *const c_char) -> *mut MNNInterpreterOpaque;
    pub fn MNN_createInterpreterFromBuffer(
        buffer: *const c_void,
        size: c_ulonglong,
    ) -> *mut MNNInterpreterOpaque;
    pub fn MNN_destroyInterpreter(net: *mut MNNInterpreterOpaque);

    pub fn MNN_createSession(
        net: *mut MNNInterpreterOpaque,
        config: *const ScheduleConfig,
    ) -> *mut MNNSessionOpaque;
    pub fn MNN_releaseSession(net: *mut MNNInterpreterOpaque, session: *mut MNNSessionOpaque);
    pub fn MNN_runSession(net: *mut MNNInterpreterOpaque, session: *mut MNNSessionOpaque) -> c_int;
    pub fn MNN_runSessionWithCallBack(
        net: *mut MNNInterpreterOpaque,
        session: *mut MNNSessionOpaque,
        before: *const c_void,
        after: *const c_void,
    ) -> c_int;

    pub fn MNN_getSessionInput(
        net: *mut MNNInterpreterOpaque,
        session: *mut MNNSessionOpaque,
        name: *const c_char,
    ) -> *mut MNNTensorOpaque;
    pub fn MNN_getSessionOutput(
        net: *mut MNNInterpreterOpaque,
        session: *mut MNNSessionOpaque,
        name: *const c_char,
    ) -> *mut MNNTensorOpaque;
    pub fn MNN_getTensorShape(tensor: *const MNNTensorOpaque) -> TensorShape;
    pub fn MNN_getTensorDataType(tensor: *const MNNTensorOpaque) -> HalideType;
    pub fn MNN_getTensorHostMap(tensor: *const MNNTensorOpaque) -> *mut c_void;
    pub fn MNN_writeTensorToHost(tensor: *mut MNNTensorOpaque, data: *const c_void);
    pub fn MNN_copyTensorToHost(tensor: *const MNNTensorOpaque, data: *mut c_void);

    pub fn MNN_getRuntime(
        net: *mut MNNInterpreterOpaque,
        session: *mut MNNSessionOpaque,
    ) -> *mut MNNRuntimeOpaque;
    pub fn MNN_waitRuntimeFinish(runtime: *mut MNNRuntimeOpaque) -> c_int;

    pub fn MNN_getVersion() -> *const c_char;
}

extern "C" {
    pub fn Llm_createLLM(config_path: *const c_char) -> *mut MNNLlmOpaque;
    pub fn Llm_destroy(llm: *mut MNNLlmOpaque);
    pub fn Llm_load(llm: *mut MNNLlmOpaque) -> c_int;
    pub fn Llm_response(
        llm: *mut MNNLlmOpaque,
        input_ids: *const c_int,
        len: c_int,
        callback: extern "C" fn(*const c_char, c_int, *mut c_void),
        user_data: *mut c_void,
    );
    pub fn Llm_tokenizer_encode(
        llm: *mut MNNLlmOpaque,
        text: *const c_char,
        tokens: *mut c_int,
        max_len: *mut c_int,
    ) -> c_int;
    pub fn Llm_tokenizer_decode(
        llm: *mut MNNLlmOpaque,
        token: c_int,
        output: *mut c_char,
        max_len: c_int,
    ) -> c_int;
    pub fn Llm_set_config(
        llm: *mut MNNLlmOpaque,
        key: *const c_char,
        value: *const c_char,
    ) -> c_int;
    pub fn Llm_reset(llm: *mut MNNLlmOpaque);
    pub fn Llm_set_tokenizer(llm: *mut MNNLlmOpaque, tokenizer_path: *const c_char) -> c_int;
}

pub type MNNInterpreter = MNNInterpreterOpaque;
pub type MNNSession = MNNSessionOpaque;
pub type MNNTensor = MNNTensorOpaque;
pub type MNNBackend = MNNBackendOpaque;
pub type MNNRuntime = MNNRuntimeOpaque;
pub type MNNLlm = MNNLlmOpaque;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnn_forward_type_values() {
        assert!(matches!(MNNForwardType::CPU, MNNForwardType::CPU));
        assert!(matches!(MNNForwardType::Metal, MNNForwardType::Metal));
        assert!(matches!(MNNForwardType::OpenCL, MNNForwardType::OpenCL));
        assert!(matches!(MNNForwardType::OpenGL, MNNForwardType::OpenGL));
        assert!(matches!(MNNForwardType::Vulkan, MNNForwardType::Vulkan));
        assert!(matches!(MNNForwardType::CUDA, MNNForwardType::CUDA));
        assert!(matches!(MNNForwardType::NN, MNNForwardType::NN));
        assert!(matches!(MNNForwardType::Auto, MNNForwardType::Auto));
    }

    #[test]
    fn test_halide_type() {
        let float32 = HalideType::float32();
        assert_eq!(float32.code, 2);
        assert_eq!(float32.bits, 32);

        let int8 = HalideType::int8();
        assert_eq!(int8.code, 0);
        assert_eq!(int8.bits, 8);
    }

    #[test]
    fn test_schedule_config_default() {
        let config = ScheduleConfig::default();
        assert!(matches!(config.type_, MNNForwardType::Auto));
    }

    #[test]
    fn test_backend_config() {
        let config = BackendConfig::new(Precision::Normal, Power::Normal, Memory::Normal);
        assert_eq!(config.precision, Precision::Normal as c_int);
        assert_eq!(config.power, Power::Normal as c_int);
        assert_eq!(config.memory, Memory::Normal as c_int);
    }

    #[test]
    fn test_precision_variants() {
        let normal = Precision::Normal;
        let high = Precision::High;
        let low = Precision::Low;
        let lowbf16 = Precision::LowBF16;
        assert!(matches!(normal, Precision::Normal));
        assert!(matches!(high, Precision::High));
        assert!(matches!(low, Precision::Low));
        assert!(matches!(lowbf16, Precision::LowBF16));
    }

    #[test]
    fn test_power_variants() {
        let normal = Power::Normal;
        let high = Power::High;
        let low = Power::Low;
        assert!(matches!(normal, Power::Normal));
        assert!(matches!(high, Power::High));
        assert!(matches!(low, Power::Low));
    }

    #[test]
    fn test_memory_variants() {
        let normal = Memory::Normal;
        let high = Memory::High;
        let low = Memory::Low;
        assert!(matches!(normal, Memory::Normal));
        assert!(matches!(high, Memory::High));
        assert!(matches!(low, Memory::Low));
    }
}
