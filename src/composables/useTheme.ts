import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export type Theme = 'system' | 'light' | 'dark' | 'blue' | 'liquid-glass';

const currentTheme = ref<Theme>('system');
const systemPreference = ref<'light' | 'dark'>('light');
let mediaQuery: MediaQueryList | null = null;
let mediaQueryListener: ((e: MediaQueryListEvent) => void) | null = null;

/**
 * Detect the current system color scheme preference
 */
function detectSystemPreference(): 'light' | 'dark' {
  if (typeof window === 'undefined' || !window.matchMedia) {
    return 'light';
  }
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

/**
 * Watch for system preference changes
 */
function watchSystemPreference() {
  if (typeof window === 'undefined' || !window.matchMedia) {
    return;
  }

  // Clean up existing listener if any
  if (mediaQuery && mediaQueryListener) {
    mediaQuery.removeEventListener('change', mediaQueryListener);
  }

  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  systemPreference.value = mediaQuery.matches ? 'dark' : 'light';

  mediaQueryListener = (e: MediaQueryListEvent) => {
    systemPreference.value = e.matches ? 'dark' : 'light';
    // If current theme is 'system', update the applied theme
    if (currentTheme.value === 'system') {
      applyThemeClass('system');
    }
  };

  mediaQuery.addEventListener('change', mediaQueryListener);
}

/**
 * Apply theme class to root element
 */
function applyThemeClass(theme: Theme) {
  const root = document.documentElement;
  
  // Remove all theme classes
  root.classList.remove('theme-light', 'theme-dark', 'theme-blue', 'theme-liquid-glass');
  
  if (theme === 'system') {
    // Apply light or dark based on system preference
    const effectiveTheme = systemPreference.value;
    root.classList.add(`theme-${effectiveTheme}`);
  } else {
    // Apply the selected theme
    root.classList.add(`theme-${theme}`);
  }
}

/**
 * Load theme preference from database
 */
async function loadTheme(): Promise<void> {
  try {
    const savedTheme = await invoke<string | null>('get_user_preference', { key: 'theme' });
    if (savedTheme && ['system', 'light', 'dark', 'blue', 'liquid-glass'].includes(savedTheme)) {
      currentTheme.value = savedTheme as Theme;
    } else {
      // Default to 'system' if no preference is saved
      currentTheme.value = 'system';
    }
  } catch (error) {
    console.error('Failed to load theme preference:', error);
    currentTheme.value = 'system';
  }
  
  // Apply the theme
  applyThemeClass(currentTheme.value);
}

/**
 * Save theme preference to database
 */
async function saveTheme(theme: Theme): Promise<void> {
  try {
    await invoke('set_user_preference', { key: 'theme', value: theme });
    currentTheme.value = theme;
    applyThemeClass(theme);
  } catch (error) {
    console.error('Failed to save theme preference:', error);
    throw error;
  }
}

/**
 * Set theme and save to database
 */
async function setTheme(theme: Theme): Promise<void> {
  await saveTheme(theme);
  
  // If switching to system, watch for changes
  if (theme === 'system') {
    watchSystemPreference();
  }
}

export function useTheme() {
  // Initialize on mount
  onMounted(async () => {
    // Detect initial system preference
    systemPreference.value = detectSystemPreference();
    
    // Load saved theme preference
    await loadTheme();
    
    // Watch for system preference changes if theme is 'system'
    if (currentTheme.value === 'system') {
      watchSystemPreference();
    }
  });

  // Clean up on unmount
  onUnmounted(() => {
    if (mediaQuery && mediaQueryListener) {
      mediaQuery.removeEventListener('change', mediaQueryListener);
      mediaQuery = null;
      mediaQueryListener = null;
    }
  });

  // Watch for theme changes and update system preference watching
  watch(currentTheme, (newTheme) => {
    if (newTheme === 'system') {
      watchSystemPreference();
    } else {
      // Clean up system preference watcher if not using system theme
      if (mediaQuery && mediaQueryListener) {
        mediaQuery.removeEventListener('change', mediaQueryListener);
        mediaQuery = null;
        mediaQueryListener = null;
      }
    }
  });

  return {
    currentTheme,
    systemPreference,
    setTheme,
    loadTheme,
  };
}

