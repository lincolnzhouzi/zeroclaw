use crate::engine::mnn::ffi::*;
use crate::error::{Error, Result};
use std::ffi::c_void;

pub struct MNNTensor {
    inner: *mut MNNTensorOpaque,
    owns_data: bool,
}

unsafe impl Send for MNNTensor {}
unsafe impl Sync for MNNTensor {}

impl MNNTensor {
    pub fn new(shape: &[i32], data_type: HalideType) -> Result<Self> {
        let dims = shape.to_vec();
        let element_size = (data_type.bits as usize / 8) * (data_type.lanes as usize);
        let total_elements: usize = dims.iter().map(|&d| d as usize).product();
        let total_bytes = total_elements * element_size;

        Ok(Self {
            inner: std::ptr::null_mut(),
            owns_data: true,
        })
    }

    pub fn from_raw(ptr: *mut MNNTensorOpaque) -> Self {
        Self {
            inner: ptr,
            owns_data: false,
        }
    }

    pub fn shape(&self) -> Vec<i32> {
        if self.inner.is_null() {
            return Vec::new();
        }
        unsafe {
            let shape = MNN_getTensorShape(self.inner);
            if shape.dims.is_null() || shape.dimCount == 0 {
                return Vec::new();
            }
            std::slice::from_raw_parts(shape.dims, shape.dimCount as usize).to_vec()
        }
    }

    pub fn data_type(&self) -> HalideType {
        if self.inner.is_null() {
            return HalideType::float32();
        }
        unsafe { MNN_getTensorDataType(self.inner) }
    }

    pub fn element_count(&self) -> usize {
        self.shape().iter().map(|&d| d as usize).product()
    }

    pub fn byte_size(&self) -> usize {
        let dtype = self.data_type();
        self.element_count() * (dtype.bits as usize / 8)
    }

    pub fn write_data<T>(&self, data: &[T]) -> Result<()> {
        if self.inner.is_null() {
            return Err(Error::ModelError("Tensor is null".to_string()));
        }
        unsafe {
            MNN_writeTensorToHost(self.inner, data.as_ptr() as *const c_void);
        }
        Ok(())
    }

    pub fn read_data<T>(&self, output: &mut [T]) -> Result<()> {
        if self.inner.is_null() {
            return Err(Error::ModelError("Tensor is null".to_string()));
        }
        if output.len() < self.element_count() {
            return Err(Error::ModelError("Output buffer too small".to_string()));
        }
        unsafe {
            MNN_copyTensorToHost(self.inner, output.as_mut_ptr() as *mut c_void);
        }
        Ok(())
    }

    pub fn host_ptr(&self) -> *mut c_void {
        if self.inner.is_null() {
            return std::ptr::null_mut();
        }
        unsafe { MNN_getTensorHostMap(self.inner) }
    }

    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }
}

impl Drop for MNNTensor {
    fn drop(&mut self) {
    }
}

pub struct TensorBuilder {
    shape: Vec<i32>,
    data_type: HalideType,
    data: Option<Vec<u8>>,
}

impl TensorBuilder {
    pub fn new() -> Self {
        Self {
            shape: Vec::new(),
            data_type: HalideType::float32(),
            data: None,
        }
    }

    pub fn shape(mut self, shape: &[i32]) -> Self {
        self.shape = shape.to_vec();
        self
    }

    pub fn data_type(mut self, data_type: HalideType) -> Self {
        self.data_type = data_type;
        self
    }

    pub fn data<T>(mut self, data: &[T]) -> Self {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<T>(),
            )
        };
        self.data = Some(bytes.to_vec());
        self
    }

    pub fn build(self) -> Result<MNNTensor> {
        MNNTensor::new(&self.shape, self.data_type)
    }
}

impl Default for TensorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TensorDesc {
    pub shape: Vec<i32>,
    pub data_type: HalideType,
    pub byte_size: usize,
}

impl TensorDesc {
    pub fn from_tensor(tensor: &MNNTensor) -> Self {
        Self {
            shape: tensor.shape(),
            data_type: tensor.data_type(),
            byte_size: tensor.byte_size(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_builder() {
        let builder = TensorBuilder::new()
            .shape(&[1, 3, 224, 224])
            .data_type(HalideType::float32());

        let desc = TensorDesc {
            shape: vec![1, 3, 224, 224],
            data_type: HalideType::float32(),
            byte_size: 1 * 3 * 224 * 224 * 4,
        };
        assert_eq!(desc.shape.len(), 4);
    }

    #[test]
    fn test_tensor_desc() {
        let desc = TensorDesc {
            shape: vec![2, 3],
            data_type: HalideType::float32(),
            byte_size: 24,
        };
        assert_eq!(desc.shape, vec![2, 3]);
        assert_eq!(desc.byte_size, 24);
    }

    #[test]
    fn test_halide_type() {
        let float32 = HalideType::float32();
        assert_eq!(float32.bits, 32);
        assert_eq!(float32.code, 2);

        let int8 = HalideType::int8();
        assert_eq!(int8.bits, 8);
        assert_eq!(int8.code, 0);
    }
}
