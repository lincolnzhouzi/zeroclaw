import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface AppSettings {
  theme: string
  language: string
  auto_discover: boolean
  discovery_interval: number
  power_mode: string
  notifications_enabled: boolean
}

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

interface SettingsStore {
  settings: AppSettings
  modelStatus: ModelStatus
  availableModels: ModelInfo[]
  loading: boolean
  error: string | null
  
  getSettings: () => Promise<void>
  updateSettings: (settings: Partial<AppSettings>) => Promise<void>
  getModelStatus: () => Promise<void>
  getAvailableModels: () => Promise<void>
  loadModel: (modelId: string) => Promise<void>
  unloadModel: () => Promise<void>
  clearError: () => void
}

const defaultSettings: AppSettings = {
  theme: 'system',
  language: 'zh-CN',
  auto_discover: true,
  discovery_interval: 30,
  power_mode: 'balanced',
  notifications_enabled: true,
}

const defaultModelStatus: ModelStatus = {
  loaded: false,
  name: '',
  backend: '',
  quantization: '',
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: defaultSettings,
  modelStatus: defaultModelStatus,
  availableModels: [],
  loading: false,
  error: null,
  
  getSettings: async () => {
    set({ loading: true, error: null })
    try {
      const settings = await invoke<AppSettings>('get_settings')
      set({ settings, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
  
  updateSettings: async (newSettings: Partial<AppSettings>) => {
    set({ loading: true, error: null })
    try {
      const settings = { ...get().settings, ...newSettings }
      await invoke('update_settings', { settings })
      set({ settings, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
  
  clearError: () => {
    set({ error: null })
  },
  
  getModelStatus: async () => {
    try {
      const status = await invoke<ModelStatus>('get_model_status')
      set({ modelStatus: status })
    } catch (error) {
      console.error('Failed to get model status:', error)
    }
  },
  
  getAvailableModels: async () => {
    try {
      const models = await invoke<ModelInfo[]>('get_available_models')
      set({ availableModels: models })
    } catch (error) {
      console.error('Failed to get available models:', error)
    }
  },
  
  loadModel: async (modelId: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('load_model', {
        config: {
          model_path: `./models/${modelId}`,
          model_name: modelId,
          backend: 'Cpu',
          quantization: 'Q4',
          context_length: 2048,
        }
      })
      await get().getModelStatus()
      set({ loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
  
  unloadModel: async () => {
    set({ loading: true, error: null })
    try {
      await invoke('unload_model')
      await get().getModelStatus()
      set({ loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
}))
