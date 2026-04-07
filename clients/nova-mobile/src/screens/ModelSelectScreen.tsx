import React, { useState } from 'react'
import { View, Text, TouchableOpacity, ScrollView, StyleSheet } from 'react-native'

interface Model {
  id: string
  name: string
  provider: string
  providerName: string
  priceInput: number
  priceOutput: number
  contextWindow: number
}

const MODELS: Model[] = [
  { id: 'gpt-4o', name: 'GPT-4o', provider: 'openai', providerName: 'OpenAI', priceInput: 2.5, priceOutput: 10.0, contextWindow: 128000 },
  { id: 'gpt-4o-mini', name: 'GPT-4o Mini', provider: 'openai', providerName: 'OpenAI', priceInput: 0.15, priceOutput: 0.6, contextWindow: 128000 },
  { id: 'claude-3-5-sonnet', name: 'Claude 3.5 Sonnet', provider: 'anthropic', providerName: 'Anthropic', priceInput: 3.0, priceOutput: 15.0, contextWindow: 200000 },
  { id: 'gemini-1-5-pro', name: 'Gemini 1.5 Pro', provider: 'google', providerName: 'Google', priceInput: 1.25, priceOutput: 5.0, contextWindow: 2000000 },
  { id: 'deepseek-chat', name: 'DeepSeek V3', provider: 'deepseek', providerName: 'DeepSeek', priceInput: 0.07, priceOutput: 0.27, contextWindow: 64000 },
]

const PROVIDERS = ['openai', 'anthropic', 'google', 'deepseek']

export default function ModelSelectScreen() {
  const [selectedProviders, setSelectedProviders] = useState<string[]>(PROVIDERS)

  const filteredModels = MODELS.filter(m => selectedProviders.includes(m.provider))

  const toggleProvider = (provider: string) => {
    if (selectedProviders.includes(provider)) {
      setSelectedProviders(selectedProviders.filter(p => p !== provider))
    } else {
      setSelectedProviders([...selectedProviders, provider])
    }
  }

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Select Model</Text>

      <ScrollView horizontal style={styles.filters} showsHorizontalScrollIndicator={false}>
        {PROVIDERS.map(provider => (
          <TouchableOpacity
            key={provider}
            onPress={() => toggleProvider(provider)}
            style={[
              styles.filterButton,
              selectedProviders.includes(provider) && styles.filterButtonActive,
            ]}
          >
            <Text
              style={[
                styles.filterButtonText,
                selectedProviders.includes(provider) && styles.filterButtonTextActive,
              ]}
            >
              {provider}
            </Text>
          </TouchableOpacity>
        ))}
      </ScrollView>

      <ScrollView style={styles.modelList}>
        {filteredModels.map(model => (
          <TouchableOpacity key={model.id} style={styles.modelCard}>
            <View style={styles.modelHeader}>
              <Text style={styles.modelName}>{model.name}</Text>
              <Text style={styles.providerName}>{model.providerName}</Text>
            </View>
            <View style={styles.modelDetails}>
              <Text style={styles.price}>
                ${model.priceInput.toFixed(2)} / ${model.priceOutput.toFixed(2)}
              </Text>
              <Text style={styles.context}>{(model.contextWindow / 1000).toFixed(0)}K context</Text>
            </View>
          </TouchableOpacity>
        ))}
      </ScrollView>
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    padding: 16,
  },
  filters: {
    paddingHorizontal: 16,
    marginBottom: 16,
  },
  filterButton: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    backgroundColor: '#f0f0f0',
    borderRadius: 20,
    marginRight: 8,
  },
  filterButtonActive: {
    backgroundColor: '#007AFF',
  },
  filterButtonText: {
    color: '#666',
    textTransform: 'capitalize',
  },
  filterButtonTextActive: {
    color: '#fff',
  },
  modelList: {
    flex: 1,
    paddingHorizontal: 16,
  },
  modelCard: {
    backgroundColor: '#f8f8f8',
    padding: 16,
    borderRadius: 12,
    marginBottom: 12,
  },
  modelHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  modelName: {
    fontSize: 18,
    fontWeight: '600',
  },
  providerName: {
    fontSize: 14,
    color: '#666',
  },
  modelDetails: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 8,
  },
  price: {
    fontSize: 14,
    color: '#007AFF',
  },
  context: {
    fontSize: 14,
    color: '#999',
  },
})
