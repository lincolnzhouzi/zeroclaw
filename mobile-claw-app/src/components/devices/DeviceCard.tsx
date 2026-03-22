import { DeviceInfo, DeviceType } from '@/types'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { Slider } from '@/components/ui/slider'
import { Badge } from '@/components/ui/badge'
import {
  Lightbulb,
  Wind,
  Tv,
  Camera,
  Lock,
  Blinds,
  Thermometer,
  Speaker,
  Radio,
  Power,
  PowerOff,
  Wifi,
  Bluetooth
} from 'lucide-react'

interface DeviceCardProps {
  device: DeviceInfo
  onConnect: () => void
  onDisconnect: () => void
  onControl: (action: string, params?: Record<string, unknown>) => void
}

const deviceIcons: Record<DeviceType, React.ReactNode> = {
  Light: <Lightbulb className="w-6 h-6" />,
  AirConditioner: <Wind className="w-6 h-6" />,
  Television: <Tv className="w-6 h-6" />,
  Camera: <Camera className="w-6 h-6" />,
  SmartLock: <Lock className="w-6 h-6" />,
  Curtain: <Blinds className="w-6 h-6" />,
  Thermostat: <Thermometer className="w-6 h-6" />,
  Speaker: <Speaker className="w-6 h-6" />,
  Sensor: <Radio className="w-6 h-6" />,
  Switch: <Power className="w-6 h-6" />,
  Plug: <Power className="w-6 h-6" />,
  Other: <Power className="w-6 h-6" />,
}

const deviceTypeNames: Record<DeviceType, string> = {
  Light: '灯光',
  AirConditioner: '空调',
  Television: '电视',
  Camera: '摄像头',
  SmartLock: '智能锁',
  Curtain: '窗帘',
  Thermostat: '温控器',
  Speaker: '音箱',
  Sensor: '传感器',
  Switch: '开关',
  Plug: '插座',
  Other: '设备',
}

export function DeviceCard({ device, onConnect, onDisconnect, onControl }: DeviceCardProps) {
  const isOnline = device.state.online

  const handlePowerToggle = () => {
    const action = device.state.power ? 'turn_off' : 'turn_on'
    onControl(action, {})
  }

  const ProtocolIcon = device.protocol === 'BLE' ? Bluetooth : Wifi

  return (
    <Card className={`${isOnline ? 'border-green-500/50' : 'border-border'}`}>
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="flex items-center gap-2 text-base">
          <span className={isOnline ? 'text-primary' : 'text-muted-foreground'}>
            {deviceIcons[device.device_type]}
          </span>
          <span className="truncate max-w-[150px]">{device.name}</span>
        </CardTitle>
        <div className="flex items-center gap-2">
          <ProtocolIcon className="w-4 h-4 text-muted-foreground" />
          <div className={`w-2 h-2 rounded-full ${isOnline ? 'bg-green-500' : 'bg-gray-400'}`} />
        </div>
      </CardHeader>

      <CardContent>
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <Badge variant="secondary">{deviceTypeNames[device.device_type]}</Badge>
            {isOnline && (
              <Switch
                checked={device.state.power ?? false}
                onCheckedChange={handlePowerToggle}
              />
            )}
          </div>

          {isOnline && device.device_type === 'Light' && device.state.brightness !== undefined && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">亮度</span>
                <span>{device.state.brightness}%</span>
              </div>
              <Slider
                value={String(device.state.brightness)}
                max={100}
                onInput={(e) => {
                  const value = parseInt(e.currentTarget.value)
                  onControl('set_brightness', { brightness: value })
                }}
              />
            </div>
          )}

          {isOnline && device.device_type === 'AirConditioner' && device.state.temperature !== undefined && (
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">温度</span>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onControl('set_temperature', { temperature: (device.state.temperature || 24) - 1 })}
                >
                  -
                </Button>
                <span className="text-lg font-bold w-12 text-center">{device.state.temperature}°C</span>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onControl('set_temperature', { temperature: (device.state.temperature || 24) + 1 })}
                >
                  +
                </Button>
              </div>
            </div>
          )}

          {isOnline && device.device_type === 'Television' && device.state.volume !== undefined && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">音量</span>
                <span>{device.state.volume}%</span>
              </div>
              <Slider
                value={String(device.state.volume)}
                max={100}
                onInput={(e) => {
                  const value = parseInt(e.currentTarget.value)
                  onControl('set_volume', { volume: value })
                }}
              />
            </div>
          )}

          <div className="flex gap-2 pt-2">
            {isOnline ? (
              <Button variant="outline" size="sm" className="flex-1" onClick={onDisconnect}>
                <PowerOff className="w-4 h-4 mr-1" />
                断开
              </Button>
            ) : (
              <Button size="sm" className="flex-1" onClick={onConnect}>
                <Power className="w-4 h-4 mr-1" />
                连接
              </Button>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
