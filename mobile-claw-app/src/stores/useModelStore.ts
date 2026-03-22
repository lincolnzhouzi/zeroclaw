import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

export interface ModelStatus {
  loaded: boolean
  name: string
  backend: string
  quantization: string
  context_length: number
  thread_count: number
}

export interface ModelInfo {
  id: string
  name: string
  size_bytes: number
  quantization: string
  context_length: number
  downloaded: boolean
  model_type: string
}

export interface HardwareInfo {
  cpu_cores: number
  total_memory: number
  gpu_available: boolean
  gpu_type: string | null
  gpu_memory: number | null
  npu_available: boolean
  npu_type: string | null
  supports_fp16: boolean
  supports_dotprod: boolean
}

export interface DownloadProgress {
  model_id: string
  status: string
  percentage: number
  downloaded_bytes: number
  total_bytes: number
  error: string | null
}

export interface LoadModelOptions {
  model_id: string
  model_path?: string
  model_name?: string
  quantization?: string
  context_length?: number
  backend_type?: string
  thread_count?: number
  power_mode?: string
}

interface ModelStore {
  models: ModelInfo[]
  activeModel: ModelInfo | null
  modelStatus: ModelStatus
  hardwareInfo: HardwareInfo | null
  downloadProgress: Map<string, DownloadProgress>
  loading: boolean
  error: string | null

  fetchModels: () => Promise<void>
  fetchModelStatus: () => Promise<void>
  fetchHardwareInfo: () => Promise<void>
  loadModel: (modelId: string, options?: Partial<LoadModelOptions>) => Promise<void>
  unloadModel: () => Promise<void>
  downloadModel: (modelId: string) => Promise<void>
  cancelDownload: (modelId: string) => Promise<void>
  deleteModel: (modelId: string) => Promise<void>
  clearError: () => void
  subscribeToDownloadProgress: () => Promise<UnlistenFn>
}

const defaultModelStatus: ModelStatus = {
  loaded: false,
  name: '',
  backend: '',
  quantization: '',
  context_length: 4096,
  thread_count: 4,
}

export const useModelStore = create<ModelStore>((set, get) => ({
  models: [],
  activeModel: null,
  modelStatus: defaultModelStatus,
  hardwareInfo: null,
  downloadProgress: new Map(),
  loading: false,
  error: null,

  fetchModels: async () => {
    set({ loading: true, error: null })
    try {
      const models = await invoke<ModelInfo[]>('get_available_models')
      set({ models, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  fetchModelStatus: async () => {
    try {
      const status = await invoke<ModelStatus>('get_model_status')
      const models = get().models
      const activeModel = models.find(m => m.name === status.name) || null
      set({ modelStatus: status, activeModel })
    } catch (error) {
      console.error('Failed to get model status:', error)
    }
  },

  fetchHardwareInfo: async () => {
    try {
      const hw = await invoke<HardwareInfo>('get_hardware_info')
      set({ hardwareInfo: hw })
    } catch (error) {
      console.error('Failed to get hardware info:', error)
    }
  },

  loadModel: async (modelId: string, options?: Partial<LoadModelOptions>) => {
    set({ loading: true, error: null })
    try {
      const models = get().models
      const model = models.find(m => m.id === modelId)

      if (!model) {
        throw new Error(`Model not found: ${modelId}`)
      }

      const request = {
        model_id: modelId,
        model_path: options?.model_path || `./models/${modelId}`,
        model_name: options?.model_name || model.name,
        quantization: options?.quantization || model.quantization,
        context_length: options?.context_length || model.context_length,
        backend_type: options?.backend_type || 'Auto',
        thread_count: options?.thread_count || 4,
        power_mode: options?.power_mode || 'Balanced',
      }

      const status = await invoke<ModelStatus>('load_model', { request })

      set({
        modelStatus: status,
        activeModel: model,
        loading: false
      })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  unloadModel: async () => {
    set({ loading: true, error: null })
    try {
      await invoke('unload_model')
      set({
        modelStatus: defaultModelStatus,
        activeModel: null,
        loading: false
      })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  downloadModel: async (modelId: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('download_model', { modelId })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  cancelDownload: async (modelId: string) => {
    try {
      await invoke('cancel_download', { modelId })
      const progress = new Map(get().downloadProgress)
      progress.delete(modelId)
      set({ downloadProgress: progress })
    } catch (error) {
      set({ error: String(error) })
    }
  },

  deleteModel: async (modelId: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('delete_model', { modelId })
      const models = get().models.filter(m => m.id !== modelId)
      set({ models, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },

  clearError: () => {
    set({ error: null })
  },

  subscribeToDownloadProgress: async () => {
    const unlisten = await listen<DownloadProgress>('download-progress', (event) => {
      const progress = event.payload
      const currentProgress = new Map(get().downloadProgress)

      if (progress.status === 'completed' || progress.status === 'error') {
        currentProgress.delete(progress.model_id)
        if (progress.status === 'completed') {
          get().fetchModels()
        }
      } else {
        currentProgress.set(progress.model_id, progress)
      }

      set({ downloadProgress: currentProgress })
    })

    return unlisten
  },
}))

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'

  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`
}

export function getBackendDisplayName(backend: string): string {
  const names: Record<string, string> = {
    'CPU': 'CPU',
    'GPU': 'GPU',
    'NPU': 'NPU',
    'Auto': '自动',
  }
  return names[backend] || backend
}

export function getQuantizationDisplayName(quant: string): string {
  const names: Record<string, string> = {
    'FP32': 'FP32 (全精度)',
    'FP16': 'FP16 (半精度)',
    'INT8': 'INT8 (量化)',
    'BF16': 'BF16',
  }
  return names[quant] || quant
}
