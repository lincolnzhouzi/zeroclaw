import { create } from 'zustand'
import { ChatMessage, ChatResponse } from '@/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface ChatStore {
  messages: ChatMessage[]
  conversationId: string | null
  loading: boolean
  error: string | null
  streaming: boolean
  
  sendMessage: (message: string) => Promise<void>
  streamMessage: (message: string) => Promise<void>
  clearConversation: () => Promise<void>
  addMessage: (message: ChatMessage) => void
  updateLastMessage: (content: string) => void
  clearError: () => void
}

export const useChatStore = create<ChatStore>((set, get) => ({
  messages: [],
  conversationId: null,
  loading: false,
  error: null,
  streaming: false,
  
  sendMessage: async (message: string) => {
    set({ loading: true, error: null })
    
    const userMessage: ChatMessage = {
      role: 'user',
      content: message,
      timestamp: new Date().toISOString(),
    }
    set(state => ({ messages: [...state.messages, userMessage] }))
    
    try {
      const response = await invoke<ChatResponse>('send_message', {
        message,
        conversationId: get().conversationId,
      })
      
      const assistantMessage: ChatMessage = {
        role: 'assistant',
        content: response.response,
        timestamp: new Date().toISOString(),
      }
      
      set(state => ({
        messages: [...state.messages, assistantMessage],
        conversationId: response.conversation_id,
        loading: false,
      }))
    } catch (error) {
      set({ error: String(error), loading: false })
    }
  },
  
  streamMessage: async (message: string) => {
    set({ streaming: true, error: null })
    
    const userMessage: ChatMessage = {
      role: 'user',
      content: message,
      timestamp: new Date().toISOString(),
    }
    set(state => ({ messages: [...state.messages, userMessage] }))
    
    const assistantMessage: ChatMessage = {
      role: 'assistant',
      content: '',
      timestamp: new Date().toISOString(),
    }
    set(state => ({ messages: [...state.messages, assistantMessage] }))
    
    const conversationId = get().conversationId || crypto.randomUUID()
    
    try {
      await invoke('stream_message', {
        message,
        conversationId,
      })
      
      const unlistenChunk = await listen<string>('chat:chunk', (event) => {
        set(state => {
          const messages = [...state.messages]
          const lastMessage = messages[messages.length - 1]
          if (lastMessage.role === 'assistant') {
            lastMessage.content += event.payload
          }
          return { messages }
        })
      })
      
      const unlistenComplete = await listen<string>('chat:complete', () => {
        set({ streaming: false, conversationId })
        unlistenChunk()
        unlistenComplete()
      })
      
      const unlistenError = await listen<string>('chat:error', (event) => {
        set({ error: event.payload, streaming: false })
        unlistenChunk()
        unlistenError()
      })
      
    } catch (error) {
      set({ error: String(error), streaming: false })
    }
  },
  
  clearConversation: async () => {
    const conversationId = get().conversationId
    if (conversationId) {
      try {
        await invoke('clear_conversation', { conversationId })
      } catch (error) {
        console.error('Failed to clear conversation:', error)
      }
    }
    set({ messages: [], conversationId: null })
  },
  
  addMessage: (message: ChatMessage) => {
    set(state => ({ messages: [...state.messages, message] }))
  },
  
  updateLastMessage: (content: string) => {
    set(state => {
      const messages = [...state.messages]
      const lastMessage = messages[messages.length - 1]
      if (lastMessage) {
        lastMessage.content = content
      }
      return { messages }
    })
  },
  
  clearError: () => {
    set({ error: null })
  },
}))
