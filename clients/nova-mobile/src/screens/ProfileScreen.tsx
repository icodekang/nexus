import React from 'react'
import { View, Text, StyleSheet } from 'react-native'

export default function ProfileScreen() {
  return (
    <View style={styles.container}>
      <Text style={styles.title}>Profile</Text>

      <View style={styles.section}>
        <Text style={styles.label}>Credits</Text>
        <Text style={styles.value}>$10.00</Text>
      </View>

      <View style={styles.section}>
        <Text style={styles.label}>API Key</Text>
        <Text style={styles.value}>sk-nova-xxxx</Text>
      </View>
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
    padding: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 24,
  },
  section: {
    marginBottom: 24,
  },
  label: {
    fontSize: 14,
    color: '#666',
    marginBottom: 4,
  },
  value: {
    fontSize: 18,
    fontWeight: '500',
  },
})
