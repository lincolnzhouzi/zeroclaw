import { create } from 'zustand'
import { DeviceInfo, CommandResult } from '@/types'
import { invoke } from '@tauri-apps/api/core'

interface DeviceStore {
  devices: DeviceInfo[]
  loading: boolean
  error: string | null
  selectedDevice: DeviceInfo | null
  
  discoverDevices: () => Promise<void>
  getAllDevices: () => Promise<void>
  selectDevice: (device: DeviceInfo | null) => void
  connectDevice: (deviceId: string) => Promise<void>
  disconnectDevice: (deviceId: string) => Promise<void>
  executeCommand: (deviceId: string, action: string, params?: Record<string, unknown>) => Promise<CommandResult>
  clearError: () => void
}

export const useDeviceStore = create<DeviceStore>((set, get) => ({
  devices: [],
  loading: false,
  error: null,
  selectedDevice: null,
  
  discoverDevices: async () => {
    set({ loading: true, error: null })
    try {
      const devices = await invoke<DeviceInfo[]>('discover_devices')
      set({ devices, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
  
  getAllDevices: async () => {
    set({ loading: true, error: null })
    try {
      const devices = await invoke<DeviceInfo[]>('get_all_devices')
      set({ devices, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
  
  selectDevice: (device) => {
    set({ selectedDevice: device })
  },
  
  connectDevice: async (deviceId: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('connect_device', { deviceId })
      const devices = get().devices.map(d => 
        d.id === deviceId ? { ...d, state: { ...d.state, online: true } } : d
      )
      set({ devices, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
  
  disconnectDevice: async (deviceId: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('disconnect_device', { deviceId })
      const devices = get().devices.map(d => 
        d.id === deviceId ? { ...d, state: { ...d.state, online: false } } : d
      )
      set({ devices, loading: false })
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
  
  executeCommand: async (deviceId: string, action: string, params?: Record<string, unknown>) => {
    set({ loading: true, error: null })
    try {
      const result = await invoke<CommandResult>('execute_device_command', { 
        deviceId, 
        action,
        parameters: params || null
      })
      set({ loading: false })
      return result
    } catch (error) {
      set({ error: String(error), loading: false })
      throw error
    }
  },
  
  clearError: () => {
    set({ error: null })
  },
}))
