import { useModelStore, getBackendDisplayName } from '@/stores/useModelStore'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Cpu, Monitor, Zap } from 'lucide-react'

export function BackendSelector() {
  const { modelStatus, hardwareInfo } = useModelStore()

  const backends = [
    {
      id: 'CPU',
      name: 'CPU',
      icon: Cpu,
      available: true,
      description: '使用 CPU 进行推理',
    },
    {
      id: 'GPU',
      name: getBackendDisplayName('GPU'),
      icon: Monitor,
      available: hardwareInfo?.gpu_available || false,
      description: hardwareInfo?.gpu_type
        ? `使用 ${hardwareInfo.gpu_type} 进行加速`
        : 'GPU 不可用',
    },
    {
      id: 'NPU',
      name: getBackendDisplayName('NPU'),
      icon: Zap,
      available: hardwareInfo?.npu_available || false,
      description: hardwareInfo?.npu_type
        ? `使用 ${hardwareInfo.npu_type} 进行加速`
        : 'NPU 不可用',
    },
  ]

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-base">推理后端</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {backends.map((backend) => {
            const isActive = modelStatus.backend === backend.id
            const Icon = backend.icon

            return (
              <div
                key={backend.id}
                className={`flex items-center justify-between p-3 rounded-lg border ${isActive
                    ? 'border-primary bg-primary/5'
                    : backend.available
                      ? 'border-border'
                      : 'border-border opacity-50'
                  }`}
              >
                <div className="flex items-center gap-3">
                  <div
                    className={`p-2 rounded-md ${isActive
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-muted'
                      }`}
                  >
                    <Icon className="w-4 h-4" />
                  </div>
                  <div>
                    <div className="font-medium flex items-center gap-2">
                      {backend.name}
                      {isActive && (
                        <Badge variant="default" className="text-xs">
                          当前
                        </Badge>
                      )}
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {backend.description}
                    </p>
                  </div>
                </div>
              </div>
            )
          })}
        </div>

        {modelStatus.loaded && (
          <div className="mt-4 pt-4 border-t">
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-muted-foreground">线程数</span>
                <p className="font-medium">{modelStatus.thread_count}</p>
              </div>
              <div>
                <span className="text-muted-foreground">上下文长度</span>
                <p className="font-medium">{modelStatus.context_length} tokens</p>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
