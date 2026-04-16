/**
 * @file main.tsx - 管理员应用入口
 * 挂载 React 应用到 DOM 的根节点
 */
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

// 将 React 应用挂载到 index.html 中的 #root 元素
ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
