/**
 * @file App.tsx - 客户端应用根组件
 * 配置路由体系：公开路由（登录）和受保护路由（带布局）
 */
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

/**
 * ProtectedRoute - 受保护路由
 * @description 未认证用户重定向到登录页
 */
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

  // 从 localStorage 恢复认证状态
  useEffect(() => {
    loadFromStorage();
  }, [loadFromStorage]);

  return (
    <BrowserRouter>
      <Routes>
        {/* 公开路由：已登录用户访问登录页会重定向到聊天 */}
        <Route path="/login" element={<PublicRoute><LoginPage /></PublicRoute>} />
        {/* 受保护路由：聊天、模型、密钥、订阅、指南页面 */}
        <Route path="/" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
          <Route index element={<Navigate to="/chat" replace />} />
          <Route path="chat" element={<ChatPage />} />
          <Route path="models" element={<ModelsPage />} />
          <Route path="keys" element={<KeysPage />} />
          <Route path="subscription" element={<SubscriptionPage />} />
          <Route path="guide" element={<GuidePage />} />
        </Route>
        {/* 未匹配路由重定向到聊天页 */}
        <Route path="*" element={<Navigate to="/chat" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

/**
 * App - 根组件
 * @description 提供 I18nProvider 包裹 AppContent
 */
export default function App() {
  return (
    <I18nProvider>
      <AppContent />
    </I18nProvider>
  );
}
