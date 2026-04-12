import { useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from './stores/authStore';
import { I18nProvider } from './i18n';
import Layout from './components/Layout';
import LoginPage from './pages/LoginPage';
import ChatPage from './pages/ChatPage';
import ModelsPage from './pages/ModelsPage';
import KeysPage from './pages/KeysPage';
import SubscriptionPage from './pages/SubscriptionPage';
import GuidePage from './pages/GuidePage';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuthStore();
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }
  return <>{children}</>;
}

function PublicRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuthStore();
  if (isAuthenticated) {
    return <Navigate to="/chat" replace />;
  }
  return <>{children}</>;
}

function AppContent() {
  const { loadFromStorage } = useAuthStore();

  useEffect(() => {
    loadFromStorage();
  }, [loadFromStorage]);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<PublicRoute><LoginPage /></PublicRoute>} />
        <Route path="/" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
          <Route index element={<Navigate to="/chat" replace />} />
          <Route path="chat" element={<ChatPage />} />
          <Route path="models" element={<ModelsPage />} />
          <Route path="keys" element={<KeysPage />} />
          <Route path="subscription" element={<SubscriptionPage />} />
          <Route path="guide" element={<GuidePage />} />
        </Route>
        <Route path="*" element={<Navigate to="/chat" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

export default function App() {
  return (
    <I18nProvider>
      <AppContent />
    </I18nProvider>
  );
}
