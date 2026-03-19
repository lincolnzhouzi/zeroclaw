export interface ModelStatus {
  loaded: boolean
  name: string
  backend: string
  quantization: string
}

export interface ModelInfo {
  id: string
  name: string
  size_mb: number
  quantization: string
  context_length: number
  downloaded: boolean
}

export interface ModelConfig {
  model_path: string
  model_name: string
  quantization: 'FP32' | 'FP16' | 'INT8' | 'BF16'
  context_length: number
  backend_type: 'CPU' | 'GPU' | 'NPU' | 'Auto'
  thread_count: number
  power_mode: 'Performance' | 'Balanced' | 'PowerSaving'
}
