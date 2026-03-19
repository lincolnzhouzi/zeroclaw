import { useEffect } from 'react'
import { useDeviceStore } from '@/stores'
import { DeviceCard } from '@/components/devices/DeviceCard'
import { Button } from '@/components/ui/button'
import { RefreshCw, Plus, Search } from 'lucide-react'

export function Devices() {
  const { devices, loading, error, discoverDevices, connectDevice, disconnectDevice, executeCommand, clearError } = useDeviceStore()
  
  useEffect(() => {
    discoverDevices()
  }, [discoverDevices])
  
  return (
    <div className="container mx-auto p-4">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-xl font-bold">设备管理</h1>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={discoverDevices} disabled={loading}>
            <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          </Button>
          <Button size="sm">
            <Plus className="w-4 h-4" />
          </Button>
        </div>
      </div>
      
      <div className="relative mb-4">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <input
          type="text"
          placeholder="搜索设备..."
          className="w-full h-10 pl-10 pr-4 rounded-md border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
        />
      </div>
      
      {error && (
        <div className="bg-red-100 dark:bg-red-900/20 border border-red-400 text-red-700 dark:text-red-400 px-4 py-3 rounded mb-4 flex items-center justify-between">
          <span>{error}</span>
          <Button variant="ghost" size="sm" onClick={clearError}>×</Button>
        </div>
      )}
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {devices.map(device => (
          <DeviceCard
            key={device.id}
            device={device}
            onConnect={() => connectDevice(device.id)}
            onDisconnect={() => disconnectDevice(device.id)}
            onControl={(action, params) => executeCommand(device.id, action, params)}
          />
        ))}
      </div>
      
      {devices.length === 0 && !loading && (
        <div className="text-center py-12 text-muted-foreground">
          <p>暂无设备</p>
          <p className="text-sm mt-2">点击右上角刷新按钮扫描附近设备</p>
        </div>
      )}
      
      {loading && devices.length === 0 && (
        <div className="text-center py-12">
          <RefreshCw className="w-8 h-8 animate-spin mx-auto text-primary" />
          <p className="mt-4 text-muted-foreground">正在扫描设备...</p>
        </div>
      )}
    </div>
  )
}
