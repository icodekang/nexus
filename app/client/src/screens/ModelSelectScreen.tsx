import React, { useEffect } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';

import { useModelStore, Model, PROVIDERS, getProviderInfo } from '../stores/modelStore';
import { fetchModels } from '../api/client';

export default function ModelSelectScreen() {
  const navigation = useNavigation();
  
  const { 
    models, 
    selectedModel, 
    selectedProvider,
    isLoading, 
    error,
    setModels, 
    selectModel, 
    selectProvider,
    setLoading,
    setError,
    getFilteredModels,
  } = useModelStore();
  
  useEffect(() => {
    loadModels();
  }, []);
  
  const loadModels = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetchModels();
      setModels(response.data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load models');
    } finally {
      setLoading(false);
    }
  };
  
  const handleSelectModel = (model: Model) => {
    selectModel(model);
    navigation.goBack();
  };
  
  const filteredModels = getFilteredModels();
  
  // Group models by provider
  const groupedModels = filteredModels.reduce((groups, model) => {
    if (!groups[model.provider]) {
      groups[model.provider] = [];
    }
    groups[model.provider].push(model);
    return groups;
  }, {} as Record<string, Model[]>);
  
  const renderModelCard = ({ item }: { item: Model }) => {
    const provider = getProviderInfo(item.provider);
    const isSelected = selectedModel?.id === item.id;
    
    return (
      <TouchableOpacity
        style={[styles.modelCard, isSelected && styles.modelCardSelected]}
        onPress={() => handleSelectModel(item)}
      >
        <View style={styles.modelHeader}>
          <View style={styles.modelInfo}>
            <Text style={styles.modelIcon}>{provider.logo}</Text>
            <View>
              <Text style={styles.modelName}>{item.name}</Text>
              <Text style={styles.providerName}>{item.providerName}</Text>
            </View>
          </View>
          {isSelected && (
            <View style={styles.selectedBadge}>
              <Text style={styles.selectedBadgeText}>✓</Text>
            </View>
          )}
        </View>
        
        <View style={styles.modelDetails}>
          <View style={styles.capabilities}>
            {item.capabilities.includes('vision') && (
              <View style={styles.capabilityTag}>
                <Text style={styles.capabilityTagText}>vision</Text>
              </View>
            )}
            {item.capabilities.includes('function_call') && (
              <View style={styles.capabilityTag}>
                <Text style={styles.capabilityTagText}>function</Text>
              </View>
            )}
          </View>
          
          <Text style={styles.contextWindow}>
            {(item.contextWindow / 1000).toFixed(0)}K context
          </Text>
        </View>
      </TouchableOpacity>
    );
  };
  
  const renderProviderFilter = () => (
    <View style={styles.filterContainer}>
      <TouchableOpacity
        style={[styles.filterButton, !selectedProvider && styles.filterButtonActive]}
        onPress={() => selectProvider(null)}
      >
        <Text style={[styles.filterButtonText, !selectedProvider && styles.filterButtonTextActive]}>
          All
        </Text>
      </TouchableOpacity>
      {PROVIDERS.map((provider) => (
        <TouchableOpacity
          key={provider.id}
          style={[styles.filterButton, selectedProvider === provider.id && styles.filterButtonActive]}
          onPress={() => selectProvider(provider.id)}
        >
          <Text style={styles.filterIcon}>{provider.logo}</Text>
          <Text style={[styles.filterButtonText, selectedProvider === provider.id && styles.filterButtonTextActive]}>
            {provider.name}
          </Text>
        </TouchableOpacity>
      ))}
    </View>
  );
  
  if (isLoading) {
    return (
      <View style={styles.centerContainer}>
        <ActivityIndicator size="large" color="#10A37F" />
        <Text style={styles.loadingText}>Loading models...</Text>
      </View>
    );
  }
  
  if (error) {
    return (
      <View style={styles.centerContainer}>
        <Text style={styles.errorText}>{error}</Text>
        <TouchableOpacity style={styles.retryButton} onPress={loadModels}>
          <Text style={styles.retryButtonText}>Retry</Text>
        </TouchableOpacity>
      </View>
    );
  }
  
  return (
    <View style={styles.container}>
      <Text style={styles.title}>Select Model</Text>
      
      {renderProviderFilter()}
      
      <FlatList
        data={Object.entries(groupedModels)}
        keyExtractor={([provider]) => provider}
        renderItem={({ item: [provider, providerModels] }) => (
          <View style={styles.providerSection}>
            <View style={styles.providerHeader}>
              <Text style={styles.providerIcon}>{getProviderInfo(provider).logo}</Text>
              <Text style={styles.providerTitle}>{getProviderInfo(provider).name}</Text>
            </View>
            {providerModels.map((model) => (
              <View key={model.id}>
                {renderModelCard({ item: model })}
              </View>
            ))}
          </View>
        )}
        contentContainerStyle={styles.listContent}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#FFFFFF',
  },
  centerContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#FFFFFF',
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#1D1D1F',
    paddingHorizontal: 16,
    paddingTop: 16,
    paddingBottom: 12,
  },
  filterContainer: {
    flexDirection: 'row',
    paddingHorizontal: 16,
    paddingBottom: 16,
    gap: 8,
    flexWrap: 'wrap',
  },
  filterButton: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 8,
    backgroundColor: '#F5F5F7',
    borderRadius: 20,
    gap: 4,
  },
  filterButtonActive: {
    backgroundColor: '#10A37F',
  },
  filterButtonText: {
    fontSize: 14,
    color: '#86868B',
    fontWeight: '500',
  },
  filterButtonTextActive: {
    color: '#FFFFFF',
  },
  filterIcon: {
    fontSize: 12,
  },
  listContent: {
    paddingHorizontal: 16,
    paddingBottom: 100,
  },
  providerSection: {
    marginBottom: 24,
  },
  providerHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
    gap: 8,
  },
  providerIcon: {
    fontSize: 20,
  },
  providerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1D1D1F',
  },
  modelCard: {
    backgroundColor: '#F5F5F7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 8,
  },
  modelCardSelected: {
    backgroundColor: '#E8F5EF',
    borderWidth: 2,
    borderColor: '#10A37F',
  },
  modelHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  modelInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  modelIcon: {
    fontSize: 28,
  },
  modelName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1D1D1F',
  },
  providerName: {
    fontSize: 12,
    color: '#86868B',
    marginTop: 2,
  },
  selectedBadge: {
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: '#10A37F',
    alignItems: 'center',
    justifyContent: 'center',
  },
  selectedBadgeText: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: '600',
  },
  modelDetails: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginTop: 12,
  },
  capabilities: {
    flexDirection: 'row',
    gap: 6,
  },
  capabilityTag: {
    backgroundColor: '#E5E5E5',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  capabilityTagText: {
    fontSize: 10,
    color: '#86868B',
    fontWeight: '500',
  },
  contextWindow: {
    fontSize: 12,
    color: '#86868B',
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#86868B',
  },
  errorText: {
    fontSize: 14,
    color: '#DC2626',
    textAlign: 'center',
    paddingHorizontal: 32,
  },
  retryButton: {
    marginTop: 16,
    backgroundColor: '#10A37F',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  retryButtonText: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: '600',
  },
});
