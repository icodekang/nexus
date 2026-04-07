import { useState } from 'react'
import ModelCard from '../components/ModelCard'
import ProviderFilter from '../components/ProviderFilter'

interface Model {
  id: string
  name: string
  provider: string
  providerName: string
  priceInput: number
  priceOutput: number
  contextWindow: number
  capabilities: string[]
}

// Mock data - in real app this comes from API
const MOCK_MODELS: Model[] = [
  {
    id: 'gpt-4o',
    name: 'GPT-4o',
    provider: 'openai',
    providerName: 'OpenAI',
    priceInput: 2.5,
    priceOutput: 10.0,
    contextWindow: 128000,
    capabilities: ['vision', 'function_call'],
  },
  {
    id: 'gpt-4o-mini',
    name: 'GPT-4o Mini',
    provider: 'openai',
    providerName: 'OpenAI',
    priceInput: 0.15,
    priceOutput: 0.6,
    contextWindow: 128000,
    capabilities: ['function_call'],
  },
  {
    id: 'claude-3-5-sonnet',
    name: 'Claude 3.5 Sonnet',
    provider: 'anthropic',
    providerName: 'Anthropic',
    priceInput: 3.0,
    priceOutput: 15.0,
    contextWindow: 200000,
    capabilities: ['vision'],
  },
  {
    id: 'gemini-1-5-pro',
    name: 'Gemini 1.5 Pro',
    provider: 'google',
    providerName: 'Google',
    priceInput: 1.25,
    priceOutput: 5.0,
    contextWindow: 2000000,
    capabilities: ['vision'],
  },
  {
    id: 'deepseek-chat',
    name: 'DeepSeek V3',
    provider: 'deepseek',
    providerName: 'DeepSeek',
    priceInput: 0.07,
    priceOutput: 0.27,
    contextWindow: 64000,
    capabilities: [],
  },
]

const PROVIDERS = ['openai', 'anthropic', 'google', 'deepseek']

export default function ModelSelect() {
  const [selectedProviders, setSelectedProviders] = useState<string[]>(PROVIDERS)
  const [sortBy, setSortBy] = useState<'price' | 'quality'>('price')

  const filteredModels = MOCK_MODELS.filter(m => selectedProviders.includes(m.provider))

  const sortedModels = [...filteredModels].sort((a, b) => {
    if (sortBy === 'price') {
      return (a.priceInput + a.priceOutput) - (b.priceInput + b.priceOutput)
    }
    return b.contextWindow - a.contextWindow
  })

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <header className="bg-white border-b px-6 py-4">
        <h1 className="text-xl font-semibold">Select Model</h1>
        <div className="mt-2 flex gap-4">
          <button
            onClick={() => setSortBy('price')}
            className={`px-3 py-1 rounded text-sm ${sortBy === 'price' ? 'bg-blue-100 text-blue-700' : 'bg-gray-100'}`}
          >
            By Price
          </button>
          <button
            onClick={() => setSortBy('quality')}
            className={`px-3 py-1 rounded text-sm ${sortBy === 'quality' ? 'bg-blue-100 text-blue-700' : 'bg-gray-100'}`}
          >
            By Quality
          </button>
        </div>
      </header>

      <div className="flex-1 overflow-y-auto p-6">
        <ProviderFilter
          providers={PROVIDERS}
          selected={selectedProviders}
          onChange={setSelectedProviders}
        />

        <div className="mt-6 grid gap-4">
          {sortedModels.map(model => (
            <ModelCard
              key={model.id}
              model={model}
              selected={false}
              onSelect={() => console.log('Selected:', model.id)}
            />
          ))}
        </div>
      </div>
    </div>
  )
}
