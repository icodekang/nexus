// Nexus Admin — Minimalist Theme
// Palette: warm neutrals + soft indigo accent
// Typography: Instrument Sans (display) + DM Sans (body)

export const theme = {
  colors: {
    bg: {
      page: '#FAFAF9',        // warm white
      card: '#FFFFFF',
      sidebar: '#18181B',      // zinc-900
      sidebarHover: '#27272A', // zinc-800
      input: '#F5F5F4',        // stone-100
    },
    text: {
      primary: '#18181B',      // zinc-900
      secondary: '#71717A',    // zinc-500
      tertiary: '#A1A1AA',     // zinc-400
      inverse: '#FAFAFA',
      sidebar: '#E4E4E7',      // zinc-200
      sidebarMuted: '#71717A', // zinc-500
    },
    accent: {
      primary: '#6366F1',      // indigo-500
      primaryHover: '#4F46E5', // indigo-600
      primarySubtle: 'rgba(99, 102, 241, 0.08)',
      success: '#22C55E',
      successSubtle: 'rgba(34, 197, 94, 0.08)',
      warning: '#F59E0B',
      warningSubtle: 'rgba(245, 158, 11, 0.08)',
      error: '#EF4444',
      errorSubtle: 'rgba(239, 68, 68, 0.08)',
    },
    border: {
      subtle: '#F5F5F4',       // stone-100
      default: '#E7E5E4',      // stone-200
    },
  },
  fonts: {
    display: "'Instrument Sans', 'Inter', sans-serif",
    body: "'DM Sans', 'Inter', sans-serif",
  },
  radius: {
    sm: '6px',
    md: '10px',
    lg: '14px',
    xl: '18px',
    full: '9999px',
  },
  shadows: {
    xs: '0 1px 2px rgba(0,0,0,0.04)',
    sm: '0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04)',
    md: '0 4px 6px -1px rgba(0,0,0,0.06), 0 2px 4px -2px rgba(0,0,0,0.04)',
  },
};

export default theme;
