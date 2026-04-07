interface ProviderFilterProps {
  providers: string[]
  selected: string[]
  onChange: (selected: string[]) => void
}

const PROVIDER_INFO: Record<string, { name: string; color: string }> = {
  openai: { name: 'OpenAI', color: 'bg-green-500' },
  anthropic: { name: 'Anthropic', color: 'bg-orange-500' },
  google: { name: 'Google', color: 'bg-blue-500' },
  deepseek: { name: 'DeepSeek', color: 'bg-red-500' },
}

export default function ProviderFilter({ providers, selected, onChange }: ProviderFilterProps) {
  const toggleProvider = (provider: string) => {
    if (selected.includes(provider)) {
      onChange(selected.filter(p => p !== provider))
    } else {
      onChange([...selected, provider])
    }
  }

  const selectAll = () => onChange([...providers])

  return (
    <div className="flex items-center gap-4">
      <span className="text-sm text-gray-600">Filter by provider:</span>
      <div className="flex flex-wrap gap-2">
        {providers.map(provider => {
          const info = PROVIDER_INFO[provider] || { name: provider, color: 'bg-gray-500' }
          const isSelected = selected.includes(provider)

          return (
            <button
              key={provider}
              onClick={() => toggleProvider(provider)}
              className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-sm transition-all ${
                isSelected
                  ? `${info.color} text-white`
                  : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
              }`}
            >
              <span
                className={`w-2 h-2 rounded-full ${isSelected ? 'bg-white' : info.color}`}
              />
              {info.name}
            </button>
          )
        })}
        {selected.length < providers.length && (
          <button
            onClick={selectAll}
            className="text-sm text-blue-600 hover:text-blue-700"
          >
            Select All
          </button>
        )}
      </div>
    </div>
  )
}
