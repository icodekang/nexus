import React, { useState, useRef, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  ScrollView,
  StyleSheet,
  KeyboardAvoidingView,
  Platform,
  ActivityIndicator,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';

import { useChatStore, Message } from '../stores/chatStore';
import { useModelStore } from '../stores/modelStore';
import { sendChat } from '../api/client';
import ChatBubble from '../components/ChatBubble';

export default function ChatScreen() {
  const navigation = useNavigation<any>();
  const scrollRef = useRef<ScrollView>(null);
  
  const [inputText, setInputText] = useState('');
  
  const { 
    conversations, 
    activeConversationId, 
    isLoading, 
    isStreaming,
    createConversation,
    addMessage,
    setLoading,
    setStreaming,
  } = useChatStore();
  
  const { selectedModel } = useModelStore();
  
  const activeConversation = conversations.find((c) => c.id === activeConversationId);

  useEffect(() => {
    // Scroll to bottom when messages change
    if (scrollRef.current) {
      setTimeout(() => {
        scrollRef.current?.scrollToEnd({ animated: true });
      }, 100);
    }
  }, [activeConversation?.messages]);

  const handleSend = async () => {
    if (!inputText.trim() || isLoading || isStreaming) return;
    
    const modelToUse = selectedModel?.id || 'gpt-4o';
    
    // Create conversation if needed
    let convId = activeConversationId;
    if (!convId) {
      const conv = createConversation(modelToUse);
      convId = conv.id;
    }
    
    const userMessage = inputText.trim();
    setInputText('');
    
    // Add user message
    addMessage(convId!, { role: 'user', content: userMessage });
    setLoading(true);
    
    try {
      // Get conversation messages for context
      const conv = conversations.find((c) => c.id === convId);
      const messages = conv?.messages.map((m) => ({
        role: m.role,
        content: m.content,
      })) || [];
      
      // Call API
      const response = await sendChat({
        model: modelToUse,
        messages,
      });
      
      // Add assistant response
      addMessage(convId!, {
        role: 'assistant',
        content: response.choices[0]?.message?.content || 'No response',
      });
    } catch (error) {
      addMessage(convId!, {
        role: 'assistant',
        content: `Error: ${error instanceof Error ? error.message : 'Unknown error'}`,
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <KeyboardAvoidingView 
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : undefined}
      keyboardVerticalOffset={Platform.OS === 'ios' ? 90 : 0}
    >
      {/* Header */}
      <View style={styles.header}>
        <TouchableOpacity 
          style={styles.headerLeft}
          onPress={() => navigation.navigate('Main', { screen: 'Models' })}
        >
          <Text style={styles.modelSelector}>
            {selectedModel?.name || 'Select Model'} ▼
          </Text>
        </TouchableOpacity>
        <View style={styles.headerRight}>
          <Text style={styles.subscriptionBadge}>
            {selectedModel?.providerName || 'Model'}
          </Text>
        </View>
      </View>

      {/* Messages */}
      <ScrollView 
        ref={scrollRef}
        style={styles.messagesContainer}
        contentContainerStyle={styles.messagesContent}
      >
        {!activeConversation || activeConversation.messages.length === 0 ? (
          <View style={styles.emptyState}>
            <Text style={styles.emptyTitle}>Start a conversation</Text>
            <Text style={styles.emptySubtitle}>
              Select a model and send a message to begin
            </Text>
          </View>
        ) : (
          activeConversation.messages.map((message) => (
            <ChatBubble
              key={message.id}
              message={message}
            />
          ))
        )}
        
        {isLoading && (
          <View style={styles.loadingContainer}>
            <ActivityIndicator size="small" color="#10A37F" />
            <Text style={styles.loadingText}>Thinking...</Text>
          </View>
        )}
      </ScrollView>

      {/* Input */}
      <View style={styles.inputContainer}>
        <TextInput
          style={styles.input}
          value={inputText}
          onChangeText={setInputText}
          placeholder="Type a message..."
          placeholderTextColor="#86868B"
          multiline
          maxLength={5000}
          onSubmitEditing={handleSend}
        />
        <TouchableOpacity
          style={[
            styles.sendButton,
            (!inputText.trim() || isLoading || isStreaming) && styles.sendButtonDisabled,
          ]}
          onPress={handleSend}
          disabled={!inputText.trim() || isLoading || isStreaming}
        >
          <Text style={styles.sendButtonText}>↑</Text>
        </TouchableOpacity>
      </View>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#FFFFFF',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#E5E5E5',
    backgroundColor: '#FFFFFF',
  },
  headerLeft: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  modelSelector: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1D1D1F',
  },
  headerRight: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  subscriptionBadge: {
    fontSize: 12,
    color: '#10A37F',
    fontWeight: '500',
  },
  messagesContainer: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  messagesContent: {
    padding: 16,
    flexGrow: 1,
  },
  emptyState: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 100,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#1D1D1F',
    marginBottom: 8,
  },
  emptySubtitle: {
    fontSize: 14,
    color: '#86868B',
    textAlign: 'center',
  },
  loadingContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 16,
    gap: 8,
  },
  loadingText: {
    fontSize: 14,
    color: '#86868B',
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    padding: 12,
    borderTopWidth: 1,
    borderTopColor: '#E5E5E5',
    backgroundColor: '#FFFFFF',
    gap: 8,
  },
  input: {
    flex: 1,
    backgroundColor: '#F5F5F7',
    borderRadius: 20,
    paddingHorizontal: 16,
    paddingVertical: 10,
    fontSize: 16,
    maxHeight: 100,
    color: '#1D1D1F',
  },
  sendButton: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: '#10A37F',
    alignItems: 'center',
    justifyContent: 'center',
  },
  sendButtonDisabled: {
    backgroundColor: '#E5E5E5',
  },
  sendButtonText: {
    fontSize: 20,
    color: '#FFFFFF',
    fontWeight: '600',
  },
});
