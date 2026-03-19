export interface DeviceInfo {
  id: string
  name: string
  device_type: DeviceType
  capabilities: string[]
  endpoint: string
  port: number
  protocol: ConnectionProtocol
  last_seen: string
  state: DeviceState
}

export type DeviceType =
  | 'Light'
  | 'AirConditioner'
  | 'Television'
  | 'Camera'
  | 'SmartLock'
  | 'Curtain'
  | 'Thermostat'
  | 'Speaker'
  | 'Sensor'
  | 'Switch'
  | 'Plug'
  | 'Other'

export type ConnectionProtocol = 'WiFi' | 'BLE' | 'Zigbee' | 'Thread' | 'USB'

export interface DeviceState {
  online: boolean
  power?: boolean
  brightness?: number
  temperature?: number
  humidity?: number
  volume?: number
  channel?: number
  locked?: boolean
  position?: number
  recording?: boolean
  mode?: string
}

export interface CommandResult {
  success: boolean
  message: string
  data?: Record<string, unknown>
}
