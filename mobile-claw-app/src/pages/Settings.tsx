import { useEffect } from 'react'
import { useSettingsStore } from '@/stores'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { 
  Moon, 
  Sun, 
  Globe, 
  Bell, 
  Zap, 
  Cpu,
  Download,
  Trash2
} from 'lucide-react'

export function Settings() {
  const { settings, getSettings, updateSettings, loading } = useSettingsStore()
  
  useEffect(() => {
    getSettings()
  }, [getSettings])
  
  const handleThemeChange = (theme: string) => {
    updateSettings({ theme })
  }
  
  const handleLanguageChange = (language: string) => {
    updateSettings({ language })
  }
  
  return (
    <div className="container mx-auto p-4 space-y-6">
      <h1 className="text-xl font-bold">设置</h1>
      
      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <Sun className="w-4 h-4" />
            外观
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm">主题</span>
            <div className="flex gap-2">
              <Button
                variant={settings.theme === 'light' ? 'default' : 'outline'}
                size="sm"
                onClick={() => handleThemeChange('light')}
              >
                <Sun className="w-4 h-4" />
              </Button>
              <Button
                variant={settings.theme === 'dark' ? 'default' : 'outline'}
                size="sm"
                onClick={() => handleThemeChange('dark')}
              >
                <Moon className="w-4 h-4" />
              </Button>
              <Button
                variant={settings.theme === 'system' ? 'default' : 'outline'}
                size="sm"
                onClick={() => handleThemeChange('system')}
              >
                系统
              </Button>
            </div>
          </div>
          
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Globe className="w-4 h-4 text-muted-foreground" />
              <span className="text-sm">语言</span>
            </div>
            <select
              value={settings.language}
              onChange={(e) => handleLanguageChange(e.target.value)}
              className="h-9 rounded-md border border-input bg-background px-3 text-sm"
            >
              <option value="zh-CN">简体中文</option>
              <option value="en-US">English</option>
            </select>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <Bell className="w-4 h-4" />
            通知
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <span className="text-sm">启用通知</span>
            <Switch
              checked={settings.notifications_enabled}
              onCheckedChange={(checked) => updateSettings({ notifications_enabled: checked })}
            />
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <Zap className="w-4 h-4" />
            设备发现
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm">自动发现设备</span>
            <Switch
              checked={settings.auto_discover}
              onCheckedChange={(checked) => updateSettings({ auto_discover: checked })}
            />
          </div>
          
          <div className="flex items-center justify-between">
            <span className="text-sm">发现间隔</span>
            <select
              value={settings.discovery_interval}
              onChange={(e) => updateSettings({ discovery_interval: parseInt(e.target.value) })}
              className="h-9 rounded-md border border-input bg-background px-3 text-sm"
            >
              <option value={10}>10 秒</option>
              <option value={30}>30 秒</option>
              <option value={60}>1 分钟</option>
              <option value={300}>5 分钟</option>
            </select>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <Cpu className="w-4 h-4" />
            模型设置
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">本地模型</p>
              <p className="text-xs text-muted-foreground">未加载</p>
            </div>
            <Button variant="outline" size="sm">
              <Download className="w-4 h-4 mr-1" />
              加载
            </Button>
          </div>
          
          <div className="flex items-center justify-between">
            <span className="text-sm">功耗模式</span>
            <select
              value={settings.power_mode}
              onChange={(e) => updateSettings({ power_mode: e.target.value })}
              className="h-9 rounded-md border border-input bg-background px-3 text-sm"
            >
              <option value="performance">高性能</option>
              <option value="balanced">均衡</option>
              <option value="powersaving">省电</option>
            </select>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardHeader>
          <CardTitle className="text-base">数据管理</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button variant="outline" className="w-full justify-start">
            <Download className="w-4 h-4 mr-2" />
            导出设置
          </Button>
          <Button variant="outline" className="w-full justify-start text-red-500 hover:text-red-600">
            <Trash2 className="w-4 h-4 mr-2" />
            清除所有数据
          </Button>
        </CardContent>
      </Card>
      
      <div className="text-center text-xs text-muted-foreground py-4">
        <p>Mobile Claw v0.1.0</p>
        <p className="mt-1">© 2026 ZeroClaw Team</p>
      </div>
    </div>
  )
}
