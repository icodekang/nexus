import { useState } from 'react'
import Sidebar from './components/Sidebar'
import Chat from './pages/Chat'
import ModelSelect from './pages/ModelSelect'
import Settings from './pages/Settings'
import './styles/globals.css'

function App() {
  const [currentView, setCurrentView] = useState<'chat' | 'models' | 'settings'>('chat')

  return (
    <div className="flex h-screen bg-gray-100">
      <Sidebar currentView={currentView} onViewChange={setCurrentView} />
      <main className="flex-1 flex flex-col">
        {currentView === 'chat' && <Chat />}
        {currentView === 'models' && <ModelSelect />}
        {currentView === 'settings' && <Settings />}
      </main>
    </div>
  )
}

export default App
