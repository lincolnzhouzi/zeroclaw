import { useEffect } from 'react'
import { useDeviceStore } from '@/stores'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { 
  Smartphone, 
  Bot, 
  Zap, 
  RefreshCw,
  Wifi,
  Bluetooth,
  Cpu
} from 'lucide-react'

export function Home() {
  const { devices, discoverDevices, loading } = useDeviceStore()
  
  useEffect(() => {
    discoverDevices()
  }, [discoverDevices])
  
  const onlineDevices = devices.filter(d => d.state.online).length
  const wifiDevices = devices.filter(d => d.protocol === 'WiFi').length
  const bleDevices = devices.filter(d => d.protocol === 'BLE').length
  
  return (
    <div className="container mx-auto p-4 space-y-6">
      <div className="text-center py-6">
        <div className="flex justify-center mb-4">
          <div className="w-20 h-20 rounded-full bg-primary/10 flex items-center justify-center">
            <Bot className="w-10 h-10 text-primary" />
          </div>
        </div>
        <h1 className="text-2xl font-bold">Mobile Claw</h1>
        <p className="text-muted-foreground mt-2">AI Agent 智能网关</p>
      </div>
      
      <div className="grid grid-cols-2 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">在线设备</p>
                <p className="text-3xl font-bold">{onlineDevices}</p>
              </div>
              <Smartphone className="w-8 h-8 text-green-500" />
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">总设备数</p>
                <p className="text-3xl font-bold">{devices.length}</p>
              </div>
              <Zap className="w-8 h-8 text-primary" />
            </div>
          </CardContent>
        </Card>
      </div>
      
      <Card>
        <CardHeader>
          <CardTitle className="text-base">网络状态</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-4">
            <div className="flex items-center gap-3">
              <Wifi className="w-5 h-5 text-blue-500" />
              <div>
                <p className="text-sm font-medium">WiFi</p>
                <p className="text-xs text-muted-foreground">{wifiDevices} 设备</p>
              </div>
            </div>
            <div className="flex items-center gap-3">
              <Bluetooth className="w-5 h-5 text-purple-500" />
              <div>
                <p className="text-sm font-medium">蓝牙</p>
                <p className="text-xs text-muted-foreground">{bleDevices} 设备</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardHeader>
          <CardTitle className="text-base">模型状态</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-3">
            <Cpu className="w-5 h-5 text-orange-500" />
            <div className="flex-1">
              <div className="flex items-center justify-between">
                <p className="text-sm font-medium">本地模型</p>
                <Badge variant="outline">未加载</Badge>
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                前往设置加载模型
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-base">快速操作</CardTitle>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => discoverDevices()}
              disabled={loading}
            >
              <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-2">
          <Button variant="outline" className="w-full justify-start">
            <Wifi className="w-4 h-4 mr-2" />
            扫描 WiFi 设备
          </Button>
          <Button variant="outline" className="w-full justify-start">
            <Bluetooth className="w-4 h-4 mr-2" />
            扫描蓝牙设备
          </Button>
          <Button variant="outline" className="w-full justify-start">
            <Bot className="w-4 h-4 mr-2" />
            开始 AI 对话
          </Button>
        </CardContent>
      </Card>
    </div>
  )
}
