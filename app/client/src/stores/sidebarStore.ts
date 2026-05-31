let activeKey: string | null = null;
const listeners = new Set<() => void>();

export const sidebarState = {
  getActiveKey: () => activeKey,
  setActiveKey: (key: string | null) => {
    activeKey = key;
    listeners.forEach((fn) => fn());
  },
  subscribe: (fn: () => void) => {
    listeners.add(fn);
    return () => { listeners.delete(fn); };
  },
};
