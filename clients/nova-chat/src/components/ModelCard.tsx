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

interface ModelCardProps {
  model: Model
  selected: boolean
  onSelect: () => void
}

const PROVIDER_COLORS: Record<string, string> = {
  openai: 'bg-green-100 text-green-700',
  anthropic: 'bg-orange-100 text-orange-700',
  google: 'bg-blue-100 text-blue-700',
  deepseek: 'bg-red-100 text-red-700',
}

const PROVIDER_LOGOS: Record<string, string> = {
  openai: '🤖',
  anthropic: '🧠',
  google: '🔵',
  deepseek: '🔴',
}

export default function ModelCard({ model, selected, onSelect }: ModelCardProps) {
  return (
    <div
      onClick={onSelect}
      className={`p-4 rounded-lg border cursor-pointer transition-all ${
        selected
          ? 'border-blue-500 bg-blue-50'
          : 'border-gray-200 bg-white hover:border-gray-300'
      }`}
    >
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3">
          <span className="text-2xl">{PROVIDER_LOGOS[model.provider] || '🤖'}</span>
          <div>
            <h3 className="font-semibold">{model.name}</h3>
            <span className={`text-xs px-2 py-1 rounded ${PROVIDER_COLORS[model.provider] || 'bg-gray-100'}`}>
              {model.providerName}
            </span>
          </div>
        </div>
        <div className="text-right">
          <p className="text-sm text-gray-500">
            ${model.priceInput.toFixed(2)} / ${model.priceOutput.toFixed(2)}
          </p>
          <p className="text-xs text-gray-400">
            {model.contextWindow.toLocaleString()} context
          </p>
        </div>
      </div>

      {model.capabilities.length > 0 && (
        <div className="mt-3 flex gap-2">
          {model.capabilities.map(cap => (
            <span key={cap} className="text-xs bg-gray-100 text-gray-600 px-2 py-1 rounded">
              {cap}
            </span>
          ))}
        </div>
      )}
    </div>
  )
}
