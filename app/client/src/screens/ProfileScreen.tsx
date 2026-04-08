import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Alert,
  ActivityIndicator,
  Modal,
  TextInput,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';

import { useUserStore } from '../stores/userStore';
import { fetchSubscription, fetchApiKeys, createApiKey, deleteApiKey, ApiKeyResponse } from '../api/client';

export default function ProfileScreen() {
  const navigation = useNavigation();
  const { user, logout } = useUserStore();
  
  const [subscription, setSubscription] = useState<any>(null);
  const [apiKeys, setApiKeys] = useState<ApiKeyResponse[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [showCreateKeyModal, setShowCreateKeyModal] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setIsLoading(true);
    try {
      const [subData, keysData] = await Promise.all([
        fetchSubscription(),
        fetchApiKeys(),
      ]);
      setSubscription(subData);
      setApiKeys(keysData.data || []);
    } catch (error) {
      console.error('Failed to load profile data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateApiKey = async () => {
    setShowCreateKeyModal(true);
  };

  const handleCreateKeyConfirm = async () => {
    setShowCreateKeyModal(false);
    try {
      await createApiKey({ name: newKeyName || undefined });
      setNewKeyName('');
      await loadData();
      Alert.alert('Success', 'API key created successfully. Make sure to copy it now - you won\'t see it again!');
    } catch (error) {
      Alert.alert('Error', 'Failed to create API key');
    }
  };

  const handleDeleteApiKey = (keyId: string) => {
    Alert.alert(
      'Delete API Key',
      'Are you sure you want to delete this API key? This action cannot be undone.',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Delete',
          style: 'destructive',
          onPress: async () => {
            try {
              await deleteApiKey(keyId);
              await loadData();
            } catch (error) {
              Alert.alert('Error', 'Failed to delete API key');
            }
          },
        },
      ]
    );
  };

  const handleLogout = () => {
    Alert.alert(
      'Logout',
      'Are you sure you want to logout?',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Logout',
          style: 'destructive',
          onPress: async () => {
            await logout();
          },
        },
      ]
    );
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return 'N/A';
    return new Date(dateString).toLocaleDateString();
  };

  return (
    <ScrollView style={styles.container}>
      {/* User Info */}
      <View style={styles.section}>
        <View style={styles.avatar}>
          <Text style={styles.avatarText}>
            {user?.email?.[0]?.toUpperCase() || 'U'}
          </Text>
        </View>
        <Text style={styles.email}>{user?.email || 'Not logged in'}</Text>
        {user?.phone && <Text style={styles.phone}>{user.phone}</Text>}
      </View>

      {/* Subscription */}
      <View style={styles.section}>
        <View style={styles.sectionHeader}>
          <Text style={styles.sectionTitle}>💰 My Subscription</Text>
        </View>
        <View style={styles.card}>
          {isLoading ? (
            <ActivityIndicator size="small" color="#10A37F" />
          ) : subscription ? (
            <>
              <View style={styles.subscriptionRow}>
                <Text style={styles.subscriptionLabel}>Plan</Text>
                <Text style={styles.subscriptionValue}>
                  {subscription.subscription_plan?.toUpperCase() || 'NONE'}
                </Text>
              </View>
              <View style={styles.subscriptionRow}>
                <Text style={styles.subscriptionLabel}>Status</Text>
                <View style={[styles.statusBadge, subscription.is_active ? styles.statusActive : styles.statusInactive]}>
                  <Text style={[styles.statusText, subscription.is_active ? styles.statusTextActive : styles.statusTextInactive]}>
                    {subscription.is_active ? 'Active' : 'Inactive'}
                  </Text>
                </View>
              </View>
              {subscription.subscription_end && (
                <View style={styles.subscriptionRow}>
                  <Text style={styles.subscriptionLabel}>Expires</Text>
                  <Text style={styles.subscriptionValue}>
                    {formatDate(subscription.subscription_end)}
                  </Text>
                </View>
              )}
              <TouchableOpacity style={styles.renewButton}>
                <Text style={styles.renewButtonText}>Renew / Upgrade</Text>
              </TouchableOpacity>
            </>
          ) : (
            <Text style={styles.noSubscription}>No active subscription</Text>
          )}
        </View>
      </View>

      {/* API Keys */}
      <View style={styles.section}>
        <View style={styles.sectionHeader}>
          <Text style={styles.sectionTitle}>🔑 API Keys</Text>
          <TouchableOpacity onPress={handleCreateApiKey}>
            <Text style={styles.addButton}>+ Create</Text>
          </TouchableOpacity>
        </View>
        <View style={styles.card}>
          {apiKeys.length === 0 ? (
            <Text style={styles.noKeys}>No API keys yet</Text>
          ) : (
            apiKeys.map((key) => (
              <View key={key.id} style={styles.apiKeyRow}>
                <View>
                  <Text style={styles.apiKeyName}>{key.name || 'Unnamed Key'}</Text>
                  <Text style={styles.apiKeyPrefix}>{key.key_prefix}...</Text>
                </View>
                <TouchableOpacity 
                  style={styles.deleteButton}
                  onPress={() => handleDeleteApiKey(key.id)}
                >
                  <Text style={styles.deleteButtonText}>Delete</Text>
                </TouchableOpacity>
              </View>
            ))
          )}
        </View>
      </View>

      {/* Settings */}
      <View style={styles.section}>
        <View style={styles.sectionHeader}>
          <Text style={styles.sectionTitle}>⚙️ Settings</Text>
        </View>
        <TouchableOpacity 
          style={styles.menuItem}
          onPress={() => (navigation as any).navigate('Settings')}
        >
          <Text style={styles.menuItemText}>App Settings</Text>
          <Text style={styles.menuItemArrow}>→</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.menuItem}>
          <Text style={styles.menuItemText}>Privacy Policy</Text>
          <Text style={styles.menuItemArrow}>→</Text>
        </TouchableOpacity>
        <TouchableOpacity style={styles.menuItem}>
          <Text style={styles.menuItemText}>Terms of Service</Text>
          <Text style={styles.menuItemArrow}>→</Text>
        </TouchableOpacity>
      </View>

      {/* Logout */}
      <TouchableOpacity style={styles.logoutButton} onPress={handleLogout}>
        <Text style={styles.logoutButtonText}>Logout</Text>
      </TouchableOpacity>

      <View style={styles.footer}>
        <Text style={styles.footerText}>Nexus v0.1.0</Text>
      </View>

      {/* Create API Key Modal */}
      <Modal
        visible={showCreateKeyModal}
        transparent
        animationType="fade"
        onRequestClose={() => setShowCreateKeyModal(false)}
      >
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>Create API Key</Text>
            <TextInput
              style={styles.modalInput}
              placeholder="Key name (optional)"
              placeholderTextColor="#86868B"
              value={newKeyName}
              onChangeText={setNewKeyName}
            />
            <View style={styles.modalButtons}>
              <TouchableOpacity
                style={styles.modalCancelButton}
                onPress={() => {
                  setShowCreateKeyModal(false);
                  setNewKeyName('');
                }}
              >
                <Text style={styles.modalCancelText}>Cancel</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={styles.modalConfirmButton}
                onPress={handleCreateKeyConfirm}
              >
                <Text style={styles.modalConfirmText}>Create</Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  section: {
    marginTop: 24,
    paddingHorizontal: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1D1D1F',
  },
  avatar: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#10A37F',
    alignItems: 'center',
    justifyContent: 'center',
    alignSelf: 'center',
    marginBottom: 12,
  },
  avatarText: {
    fontSize: 32,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  email: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1D1D1F',
    textAlign: 'center',
  },
  phone: {
    fontSize: 14,
    color: '#86868B',
    textAlign: 'center',
    marginTop: 4,
  },
  card: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
  },
  subscriptionRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#F5F5F7',
  },
  subscriptionLabel: {
    fontSize: 14,
    color: '#86868B',
  },
  subscriptionValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1D1D1F',
  },
  statusBadge: {
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 12,
  },
  statusActive: {
    backgroundColor: '#DCFCE7',
  },
  statusInactive: {
    backgroundColor: '#FEE2E2',
  },
  statusText: {
    fontSize: 12,
    fontWeight: '600',
  },
  statusTextActive: {
    color: '#16A34A',
  },
  statusTextInactive: {
    color: '#DC2626',
  },
  noSubscription: {
    fontSize: 14,
    color: '#86868B',
    textAlign: 'center',
    paddingVertical: 12,
  },
  renewButton: {
    backgroundColor: '#10A37F',
    borderRadius: 8,
    paddingVertical: 12,
    alignItems: 'center',
    marginTop: 16,
  },
  renewButtonText: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: '600',
  },
  addButton: {
    fontSize: 14,
    color: '#10A37F',
    fontWeight: '600',
  },
  noKeys: {
    fontSize: 14,
    color: '#86868B',
    textAlign: 'center',
    paddingVertical: 12,
  },
  apiKeyRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#F5F5F7',
  },
  apiKeyName: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1D1D1F',
  },
  apiKeyPrefix: {
    fontSize: 12,
    color: '#86868B',
    marginTop: 2,
    fontFamily: 'monospace',
  },
  deleteButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
  },
  deleteButtonText: {
    fontSize: 14,
    color: '#DC2626',
    fontWeight: '500',
  },
  menuItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: '#FFFFFF',
    paddingHorizontal: 16,
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#F5F5F7',
  },
  menuItemText: {
    fontSize: 16,
    color: '#1D1D1F',
  },
  menuItemArrow: {
    fontSize: 16,
    color: '#86868B',
  },
  logoutButton: {
    marginHorizontal: 16,
    marginTop: 32,
    backgroundColor: '#FEE2E2',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
  },
  logoutButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#DC2626',
  },
  footer: {
    alignItems: 'center',
    paddingVertical: 32,
  },
  footerText: {
    fontSize: 12,
    color: '#86868B',
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modalContent: {
    backgroundColor: '#FFFFFF',
    borderRadius: 16,
    padding: 24,
    width: '100%',
    maxWidth: 340,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1D1D1F',
    marginBottom: 16,
  },
  modalInput: {
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 12,
    fontSize: 16,
    color: '#1D1D1F',
    marginBottom: 20,
  },
  modalButtons: {
    flexDirection: 'row',
    gap: 12,
  },
  modalCancelButton: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    backgroundColor: '#F5F5F7',
    alignItems: 'center',
  },
  modalCancelText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#86868B',
  },
  modalConfirmButton: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 8,
    backgroundColor: '#10A37F',
    alignItems: 'center',
  },
  modalConfirmText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
});
