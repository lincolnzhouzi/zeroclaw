import { useState, useRef, useEffect } from 'react'
import { useChatStore } from '@/stores'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Send, Trash2, Bot, User, Loader2 } from 'lucide-react'

export function Chat() {
  const { messages, loading, streaming, sendMessage, streamMessage, clearConversation, clearError, error } = useChatStore()
  const [input, setInput] = useState('')
  const messagesEndRef = useRef<HTMLDivElement>(null)
  
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])
  
  const handleSend = async () => {
    if (!input.trim() || loading || streaming) return
    
    const message = input.trim()
    setInput('')
    await sendMessage(message)
  }
  
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }
  
  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-3 border-b bg-card">
        <h1 className="text-lg font-semibold">AI 对话</h1>
        <Button variant="ghost" size="sm" onClick={clearConversation}>
          <Trash2 className="w-4 h-4" />
        </Button>
      </div>
      
      {error && (
        <div className="mx-4 mt-4 bg-red-100 dark:bg-red-900/20 border border-red-400 text-red-700 dark:text-red-400 px-4 py-3 rounded flex items-center justify-between">
          <span className="text-sm">{error}</span>
          <Button variant="ghost" size="sm" onClick={clearError}>×</Button>
        </div>
      )}
      
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {messages.length === 0 && (
          <div className="text-center py-12 text-muted-foreground">
            <Bot className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>开始与 AI 对话</p>
            <p className="text-sm mt-2">您可以询问设备状态、控制智能设备等</p>
          </div>
        )}
        
        {messages.map((message, index) => (
          <div
            key={index}
            className={`flex gap-3 ${message.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            {message.role === 'assistant' && (
              <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0">
                <Bot className="w-4 h-4 text-primary" />
              </div>
            )}
            <div
              className={`max-w-[80%] rounded-lg px-4 py-2 ${
                message.role === 'user'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted'
              }`}
            >
              <p className="text-sm whitespace-pre-wrap">{message.content}</p>
            </div>
            {message.role === 'user' && (
              <div className="w-8 h-8 rounded-full bg-secondary flex items-center justify-center flex-shrink-0">
                <User className="w-4 h-4" />
              </div>
            )}
          </div>
        ))}
        
        {(loading || streaming) && messages[messages.length - 1]?.role === 'user' && (
          <div className="flex gap-3">
            <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0">
              <Bot className="w-4 h-4 text-primary" />
            </div>
            <div className="bg-muted rounded-lg px-4 py-2">
              {streaming ? (
                <p className="text-sm">{messages[messages.length - 1]?.content || '...'}</p>
              ) : (
                <Loader2 className="w-4 h-4 animate-spin" />
              )}
            </div>
          </div>
        )}
        
        <div ref={messagesEndRef} />
      </div>
      
      <div className="p-4 border-t bg-card">
        <div className="flex gap-2">
          <Input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="输入消息..."
            disabled={loading || streaming}
            className="flex-1"
          />
          <Button onClick={handleSend} disabled={!input.trim() || loading || streaming}>
            {loading || streaming ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Send className="w-4 h-4" />
            )}
          </Button>
        </div>
      </div>
    </div>
  )
}
