import { useEffect } from 'react'
import { useModelStore, formatBytes, getQuantizationDisplayName } from '@/stores/useModelStore'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Check, Download, Loader2, Cpu, HardDrive, Zap, Trash2 } from 'lucide-react'

export function ModelSelector() {
  const {
    models,
    activeModel,
    downloadProgress,
    loading,
    fetchModels,
    fetchModelStatus,
    loadModel,
    unloadModel,
    downloadModel,
    deleteModel,
    subscribeToDownloadProgress,
  } = useModelStore()

  useEffect(() => {
    fetchModels()
    fetchModelStatus()

    let unlisten: (() => void) | undefined

    subscribeToDownloadProgress().then((fn) => {
      unlisten = fn
    })

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [fetchModels, fetchModelStatus, subscribeToDownloadProgress])

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold">本地模型</h2>

      {models.length === 0 && !loading && (
        <Card>
          <CardContent className="py-8 text-center text-muted-foreground">
            <HardDrive className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>暂无可用模型</p>
            <p className="text-sm mt-2">请下载模型以开始使用</p>
          </CardContent>
        </Card>
      )}

      {models.map((model) => {
        const isActive = activeModel?.id === model.id
        const progress = downloadProgress.get(model.id)
        const isLoading = loading && !progress

        return (
          <Card key={model.id} className={isActive ? 'border-primary' : ''}>
            <CardHeader className="pb-2">
              <CardTitle className="text-base flex items-center justify-between">
                <span>{model.name}</span>
                {isActive && (
                  <Badge variant="default" className="ml-2">
                    <Check className="w-3 h-3 mr-1" />
                    已加载
                  </Badge>
                )}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-center gap-2 mb-3 flex-wrap">
                <Badge variant="outline">
                  {getQuantizationDisplayName(model.quantization)}
                </Badge>
                <Badge variant="secondary">
                  <HardDrive className="w-3 h-3 mr-1" />
                  {formatBytes(model.size_bytes)}
                </Badge>
                <Badge variant="secondary">
                  <Zap className="w-3 h-3 mr-1" />
                  {model.context_length} tokens
                </Badge>
                <Badge variant="outline">{model.model_type}</Badge>
              </div>

              {progress && progress.status !== 'completed' && (
                <div className="mb-3">
                  <Progress value={progress.percentage} className="h-2" />
                  <p className="text-xs text-muted-foreground mt-1">
                    {progress.status === 'downloading' &&
                      `下载中... ${progress.percentage}% (${formatBytes(
                        progress.downloaded_bytes
                      )} / ${formatBytes(progress.total_bytes)})`}
                    {progress.status === 'extracting' && '解压中...'}
                    {progress.status === 'error' && progress.error}
                  </p>
                </div>
              )}

              <div className="flex gap-2">
                {!model.downloaded && !progress && (
                  <Button
                    size="sm"
                    onClick={() => downloadModel(model.id)}
                    disabled={loading}
                  >
                    <Download className="w-4 h-4 mr-1" />
                    下载
                  </Button>
                )}

                {model.downloaded && !isActive && (
                  <Button
                    size="sm"
                    onClick={() => loadModel(model.id)}
                    disabled={isLoading}
                  >
                    {isLoading ? (
                      <Loader2 className="w-4 h-4 mr-1 animate-spin" />
                    ) : (
                      <Cpu className="w-4 h-4 mr-1" />
                    )}
                    加载
                  </Button>
                )}

                {isActive && (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => unloadModel()}
                    disabled={loading}
                  >
                    卸载
                  </Button>
                )}

                {model.downloaded && !isActive && (
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => deleteModel(model.id)}
                    disabled={loading}
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                )}
              </div>
            </CardContent>
          </Card>
        )
      })}
    </div>
  )
}
