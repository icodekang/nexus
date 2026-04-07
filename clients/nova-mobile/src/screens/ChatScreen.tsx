import React, { useState } from 'react'
import { View, Text, TextInput, TouchableOpacity, ScrollView, StyleSheet } from 'react-native'

interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
}

export default function ChatScreen() {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')

  const handleSend = () => {
    if (!input.trim()) return

    const userMsg: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: input.trim(),
    }
    setMessages(prev => [...prev, userMsg])
    setInput('')

    // Simulate response
    setTimeout(() => {
      const assistantMsg: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: `Echo: ${userMsg.content}`,
      }
      setMessages(prev => [...prev, assistantMsg])
    }, 1000)
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>NovaChat</Text>
        <Text style={styles.model}>GPT-4o</Text>
      </View>

      <ScrollView style={styles.messages}>
        {messages.length === 0 ? (
          <Text style={styles.empty}>Start a conversation...</Text>
        ) : (
          messages.map(msg => (
            <View
              key={msg.id}
              style={[
                styles.message,
                msg.role === 'user' ? styles.userMessage : styles.assistantMessage,
              ]}
            >
              <Text style={msg.role === 'user' ? styles.userText : styles.assistantText}>
                {msg.content}
              </Text>
            </View>
          ))
        )}
      </ScrollView>

      <View style={styles.inputContainer}>
        <TextInput
          style={styles.input}
          value={input}
          onChangeText={setInput}
          placeholder="Type a message..."
          multiline
        />
        <TouchableOpacity style={styles.sendButton} onPress={handleSend}>
          <Text style={styles.sendButtonText}>Send</Text>
        </TouchableOpacity>
      </View>
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  header: {
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e5e5',
  },
  title: {
    fontSize: 20,
    fontWeight: 'bold',
  },
  model: {
    fontSize: 14,
    color: '#666',
  },
  messages: {
    flex: 1,
    padding: 16,
  },
  empty: {
    textAlign: 'center',
    color: '#999',
    marginTop: 32,
  },
  message: {
    maxWidth: '80%',
    padding: 12,
    borderRadius: 12,
    marginBottom: 8,
  },
  userMessage: {
    alignSelf: 'flex-end',
    backgroundColor: '#007AFF',
  },
  assistantMessage: {
    alignSelf: 'flex-start',
    backgroundColor: '#f0f0f0',
  },
  userText: {
    color: '#fff',
  },
  assistantText: {
    color: '#000',
  },
  inputContainer: {
    flexDirection: 'row',
    padding: 12,
    borderTopWidth: 1,
    borderTopColor: '#e5e5e5',
  },
  input: {
    flex: 1,
    borderWidth: 1,
    borderColor: '#e5e5e5',
    borderRadius: 20,
    paddingHorizontal: 16,
    paddingVertical: 8,
    maxHeight: 100,
  },
  sendButton: {
    marginLeft: 8,
    backgroundColor: '#007AFF',
    borderRadius: 20,
    paddingHorizontal: 20,
    justifyContent: 'center',
  },
  sendButtonText: {
    color: '#fff',
    fontWeight: '600',
  },
})
