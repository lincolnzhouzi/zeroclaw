import { ReactNode } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { cn } from '@/lib/utils'
import { 
  Home, 
  Smartphone, 
  MessageSquare, 
  Settings,
  Bot
} from 'lucide-react'

interface LayoutProps {
  children: ReactNode
}

const navItems = [
  { path: '/', icon: Home, label: '首页' },
  { path: '/devices', icon: Smartphone, label: '设备' },
  { path: '/chat', icon: MessageSquare, label: '对话' },
  { path: '/settings', icon: Settings, label: '设置' },
]

export function Layout({ children }: LayoutProps) {
  const location = useLocation()
  
  return (
    <div className="flex flex-col h-screen bg-background">
      <header className="flex items-center justify-between px-4 py-3 border-b bg-card">
        <div className="flex items-center gap-2">
          <Bot className="w-6 h-6 text-primary" />
          <h1 className="text-lg font-semibold">Mobile Claw</h1>
        </div>
      </header>
      
      <main className="flex-1 overflow-auto">
        {children}
      </main>
      
      <nav className="flex items-center justify-around px-4 py-2 border-t bg-card safe-area-bottom">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path
          const Icon = item.icon
          
          return (
            <Link
              key={item.path}
              to={item.path}
              className={cn(
                "flex flex-col items-center gap-1 px-3 py-2 rounded-lg transition-colors",
                isActive 
                  ? "text-primary bg-primary/10" 
                  : "text-muted-foreground hover:text-foreground"
              )}
            >
              <Icon className="w-5 h-5" />
              <span className="text-xs">{item.label}</span>
            </Link>
          )
        })}
      </nav>
    </div>
  )
}
