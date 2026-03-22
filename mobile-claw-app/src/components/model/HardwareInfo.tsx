import { useEffect } from 'react'
import { useModelStore, formatBytes } from '@/stores/useModelStore'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Cpu, HardDrive, Monitor, Zap, Check, X } from 'lucide-react'

export function HardwareInfo() {
  const { hardwareInfo, fetchHardwareInfo } = useModelStore()

  useEffect(() => {
    fetchHardwareInfo()
  }, [fetchHardwareInfo])

  if (!hardwareInfo) {
    return (
      <Card>
        <CardContent className="py-8 text-center text-muted-foreground">
          <Monitor className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>正在检测硬件信息...</p>
        </CardContent>
      </Card>
    )
  }

  const features = [
    {
      name: 'FP16 支持',
      available: hardwareInfo.supports_fp16,
      description: '半精度浮点运算加速',
    },
    {
      name: 'DOTPROD 支持',
      available: hardwareInfo.supports_dotprod,
      description: '点积运算加速 (ARM)',
    },
  ]

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-base">硬件信息</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-md bg-muted">
                <Cpu className="w-4 h-4" />
              </div>
              <div>
                <p className="text-xs text-muted-foreground">CPU 核心</p>
                <p className="font-medium">{hardwareInfo.cpu_cores} 核</p>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <div className="p-2 rounded-md bg-muted">
                <HardDrive className="w-4 h-4" />
              </div>
              <div>
                <p className="text-xs text-muted-foreground">内存</p>
                <p className="font-medium">{formatBytes(hardwareInfo.total_memory)}</p>
              </div>
            </div>
          </div>

          <div className="space-y-2">
            <p className="text-sm font-medium">加速器</p>

            <div className="flex items-center justify-between p-2 rounded-lg border">
              <div className="flex items-center gap-2">
                <Monitor className="w-4 h-4" />
                <span className="text-sm">GPU</span>
              </div>
              <div className="flex items-center gap-2">
                {hardwareInfo.gpu_available ? (
                  <>
                    <Badge variant="secondary">{hardwareInfo.gpu_type}</Badge>
                    <Check className="w-4 h-4 text-green-500" />
                  </>
                ) : (
                  <X className="w-4 h-4 text-muted-foreground" />
                )}
              </div>
            </div>

            <div className="flex items-center justify-between p-2 rounded-lg border">
              <div className="flex items-center gap-2">
                <Zap className="w-4 h-4" />
                <span className="text-sm">NPU</span>
              </div>
              <div className="flex items-center gap-2">
                {hardwareInfo.npu_available ? (
                  <>
                    <Badge variant="secondary">{hardwareInfo.npu_type}</Badge>
                    <Check className="w-4 h-4 text-green-500" />
                  </>
                ) : (
                  <X className="w-4 h-4 text-muted-foreground" />
                )}
              </div>
            </div>
          </div>

          <div className="space-y-2">
            <p className="text-sm font-medium">CPU 特性</p>
            <div className="flex flex-wrap gap-2">
              {features.map((feature) => (
                <Badge
                  key={feature.name}
                  variant={feature.available ? 'default' : 'outline'}
                  className="cursor-help"
                  title={feature.description}
                >
                  {feature.available ? (
                    <Check className="w-3 h-3 mr-1" />
                  ) : (
                    <X className="w-3 h-3 mr-1" />
                  )}
                  {feature.name}
                </Badge>
              ))}
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
