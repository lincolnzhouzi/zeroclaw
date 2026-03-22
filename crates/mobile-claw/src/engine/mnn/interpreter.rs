use crate::engine::mnn::ffi::*;
use crate::error::{Error, Result};
use std::ffi::{c_int, c_ulonglong, c_void, CStr, CString};
use std::path::Path;
use std::ptr;

pub struct MNNInterpreterWrapper {
    inner: *mut MNNInterpreter,
    session: Option<*mut MNNSession>,
    config: ScheduleConfig,
    backend_config: Option<BackendConfig>,
}

unsafe impl Send for MNNInterpreterWrapper {}
unsafe impl Sync for MNNInterpreterWrapper {}

impl MNNInterpreterWrapper {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        let path_cstr = CString::new(path_str)?;

        unsafe {
            let interpreter = MNN_createInterpreterFromFile(path_cstr.as_ptr());
            if interpreter.is_null() {
                return Err(Error::ModelError(format!(
                    "Failed to create interpreter from file: {:?}",
                    path.as_ref()
                )));
            }

            Ok(Self {
                inner: interpreter,
                session: None,
                config: ScheduleConfig::default(),
                backend_config: None,
            })
        }
    }

    pub fn from_buffer(buffer: &[u8]) -> Result<Self> {
        unsafe {
            let interpreter = MNN_createInterpreterFromBuffer(
                buffer.as_ptr() as *const c_void,
                buffer.len() as c_ulonglong,
            );
            if interpreter.is_null() {
                return Err(Error::ModelError(
                    "Failed to create interpreter from buffer".to_string(),
                ));
            }

            Ok(Self {
                inner: interpreter,
                session: None,
                config: ScheduleConfig::default(),
                backend_config: None,
            })
        }
    }

    pub fn set_backend(&mut self, backend_type: MNNForwardType) -> &mut Self {
        self.config.type_ = backend_type;
        self
    }

    pub fn set_threads(&mut self, threads: i32) -> &mut Self {
        self.config.numThread = threads;
        self
    }

    pub fn set_precision(&mut self, precision: Precision) -> &mut Self {
        self.backend_config
            .get_or_insert(BackendConfig::new(precision, Power::Normal, Memory::Normal))
            .precision = precision as c_int;
        self
    }

    pub fn set_power(&mut self, power: Power) -> &mut Self {
        self.backend_config
            .get_or_insert(BackendConfig::new(Precision::Normal, power, Memory::Normal))
            .power = power as c_int;
        self
    }

    pub fn set_memory(&mut self, memory: Memory) -> &mut Self {
        self.backend_config
            .get_or_insert(BackendConfig::new(Precision::Normal, Power::Normal, memory))
            .memory = memory as c_int;
        self
    }

    pub fn create_session(&mut self) -> Result<&mut Self> {
        unsafe {
            let mut config = self.config;
            if let Some(ref backend_cfg) = self.backend_config {
                config.backendConfig = backend_cfg as *const BackendConfig as *mut BackendConfig;
            }

            let session = MNN_createSession(self.inner, &config);
            if session.is_null() {
                return Err(Error::ModelError("Failed to create session".to_string()));
            }
            self.session = Some(session);
        }
        Ok(self)
    }

    pub fn run(&self) -> Result<()> {
        let session = self
            .session
            .ok_or_else(|| Error::ModelError("Session not created".to_string()))?;

        unsafe {
            let ret = MNN_runSession(self.inner, session);
            if ret != 0 {
                return Err(Error::ModelError("Session run failed".to_string()));
            }
        }
        Ok(())
    }

    pub fn get_input(&self, name: Option<&str>) -> Result<MNNTensorRef> {
        let session = self
            .session
            .ok_or_else(|| Error::ModelError("Session not created".to_string()))?;

        unsafe {
            let name_ptr = name
                .map(|n| CString::new(n).unwrap())
                .map(|c| c.as_ptr())
                .unwrap_or(ptr::null());
            let tensor = MNN_getSessionInput(self.inner, session, name_ptr);
            if tensor.is_null() {
                return Err(Error::ModelError("Failed to get input tensor".to_string()));
            }
            Ok(MNNTensorRef { inner: tensor })
        }
    }

    pub fn get_output(&self, name: Option<&str>) -> Result<MNNTensorRef> {
        let session = self
            .session
            .ok_or_else(|| Error::ModelError("Session not created".to_string()))?;

        unsafe {
            let name_ptr = name
                .map(|n| CString::new(n).unwrap())
                .map(|c| c.as_ptr())
                .unwrap_or(ptr::null());
            let tensor = MNN_getSessionOutput(self.inner, session, name_ptr);
            if tensor.is_null() {
                return Err(Error::ModelError("Failed to get output tensor".to_string()));
            }
            Ok(MNNTensorRef { inner: tensor })
        }
    }

    pub fn get_version() -> String {
        unsafe {
            let version_ptr = MNN_getVersion();
            if version_ptr.is_null() {
                return "unknown".to_string();
            }
            CStr::from_ptr(version_ptr).to_string_lossy().into_owned()
        }
    }
}

impl Drop for MNNInterpreterWrapper {
    fn drop(&mut self) {
        #[cfg(feature = "mnn")]
        unsafe {
            if let Some(session) = self.session {
                MNN_releaseSession(self.inner, session);
            }
            MNN_destroyInterpreter(self.inner);
        }
    }
}

pub struct MNNTensorRef {
    inner: *mut MNNTensor,
}

impl MNNTensorRef {
    pub fn shape(&self) -> Vec<i32> {
        unsafe {
            let shape = MNN_getTensorShape(self.inner);
            if shape.dims.is_null() || shape.dimCount == 0 {
                return Vec::new();
            }
            std::slice::from_raw_parts(shape.dims, shape.dimCount as usize).to_vec()
        }
    }

    pub fn data_type(&self) -> HalideType {
        unsafe { MNN_getTensorDataType(self.inner) }
    }

    pub fn element_size(&self) -> usize {
        let dtype = self.data_type();
        (dtype.bits as usize / 8) * (dtype.lanes as usize)
    }

    pub fn write<T>(&self, data: &[T]) {
        unsafe {
            MNN_writeTensorToHost(self.inner, data.as_ptr() as *const c_void);
        }
    }

    pub fn read<T>(&self, output: &mut [T]) {
        unsafe {
            MNN_copyTensorToHost(self.inner, output.as_mut_ptr() as *mut c_void);
        }
    }

    pub fn host_ptr(&self) -> *mut c_void {
        unsafe { MNN_getTensorHostMap(self.inner) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_config_default() {
        let config = ScheduleConfig::default();
        assert_eq!(config.numThread, 4);
        assert!(matches!(config.type_, MNNForwardType::Auto));
    }

    #[test]
    fn test_schedule_config_new() {
        let config = ScheduleConfig::new(MNNForwardType::CPU, 8);
        assert_eq!(config.numThread, 8);
        assert!(matches!(config.type_, MNNForwardType::CPU));
    }

    #[test]
    fn test_backend_config() {
        let config = BackendConfig::new(Precision::High, Power::High, Memory::High);
        assert_eq!(config.precision, Precision::High as c_int);
        assert_eq!(config.power, Power::High as c_int);
        assert_eq!(config.memory, Memory::High as c_int);
    }

    #[test]
    fn test_halide_type() {
        let float32 = HalideType::float32();
        assert_eq!(float32.bits, 32);
        assert_eq!(float32.code, 2);

        let int32 = HalideType::int32();
        assert_eq!(int32.bits, 32);
        assert_eq!(int32.code, 0);
    }
}
