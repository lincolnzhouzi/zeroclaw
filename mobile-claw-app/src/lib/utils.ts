import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatDate(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date
  return d.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

export function getDeviceIcon(type: string): string {
  const icons: Record<string, string> = {
    Light: 'lightbulb',
    AirConditioner: 'wind',
    Television: 'tv',
    Camera: 'camera',
    SmartLock: 'lock',
    Curtain: 'blinds',
    Thermostat: 'thermometer',
    Speaker: 'speaker',
    Sensor: 'radio',
    Switch: 'toggle-left',
    Plug: 'plug',
    Other: 'box',
  }
  return icons[type] || 'box'
}
