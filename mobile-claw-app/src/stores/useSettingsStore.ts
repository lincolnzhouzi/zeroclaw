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

interface SettingsStore {
  settings: AppSettings
  loading: boolean
  error: string | null
  
  getSettings: () => Promise<void>
  updateSettings: (settings: Partial<AppSettings>) => Promise<void>
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

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: defaultSettings,
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
}))
