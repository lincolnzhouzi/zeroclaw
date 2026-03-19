export interface ChatMessage {
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: string
}

export interface ChatResponse {
  response: string
  conversation_id: string
}

export interface Conversation {
  id: string
  title: string
  messages: ChatMessage[]
  created_at: string
  updated_at: string
}
