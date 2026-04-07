interface SidebarProps {
  currentView: string
  onViewChange: (view: 'chat' | 'models' | 'settings') => void
}

export default function Sidebar({ currentView, onViewChange }: SidebarProps) {
  const navItems = [
    { id: 'chat', label: 'Chat', icon: '💬' },
    { id: 'models', label: 'Models', icon: '🤖' },
    { id: 'settings', label: 'Settings', icon: '⚙️' },
  ]

  return (
    <aside className="w-64 bg-gray-900 text-white flex flex-col">
      <div className="p-4 border-b border-gray-800">
        <h1 className="text-xl font-bold">NovaChat</h1>
      </div>

      <nav className="flex-1 p-4">
        {navItems.map(item => (
          <button
            key={item.id}
            onClick={() => onViewChange(item.id as 'chat' | 'models' | 'settings')}
            className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg mb-2 transition-colors ${
              currentView === item.id
                ? 'bg-blue-600 text-white'
                : 'text-gray-400 hover:bg-gray-800 hover:text-white'
            }`}
          >
            <span>{item.icon}</span>
            <span>{item.label}</span>
          </button>
        ))}
      </nav>

      <div className="p-4 border-t border-gray-800">
        <div className="text-sm text-gray-400">
          <p>Credits: $10.00</p>
        </div>
      </div>
    </aside>
  )
}
