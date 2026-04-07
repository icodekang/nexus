import { useState } from 'react'
import ChatMessage from '../components/ChatMessage'
import ChatInput from '../components/ChatInput'

interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
}

export default function Chat() {
  const [messages, setMessages] = useState<Message[]>([])
  const [selectedModel, setSelectedModel] = useState('gpt-4o')

  const handleSend = async (content: string) => {
    // Add user message
    const userMsg: Message = {
      id: Date.now().toString(),
      role: 'user',
      content,
    }
    setMessages(prev => [...prev, userMsg])

    // TODO: Call API
    // Simulate assistant response
    setTimeout(() => {
      const assistantMsg: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: `Echo: ${content}`,
      }
      setMessages(prev => [...prev, assistantMsg])
    }, 1000)
  }

  return (
    <div className="flex-1 flex flex-col">
      {/* Header */}
      <header className="bg-white border-b px-6 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-lg font-semibold">NovaChat</h1>
            <p className="text-sm text-gray-500">Model: {selectedModel}</p>
          </div>
        </div>
      </header>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-6 space-y-4">
        {messages.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <p className="text-gray-500">Start a conversation...</p>
          </div>
        ) : (
          messages.map(msg => (
            <ChatMessage key={msg.id} role={msg.role} content={msg.content} />
          ))
        )}
      </div>

      {/* Input */}
      <ChatInput onSend={handleSend} />
    </div>
  )
}
