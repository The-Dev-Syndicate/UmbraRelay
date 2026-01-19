<template>
  <div>
    <!-- Theme Section -->
    <SettingsSection title="Appearance">
      <div class="theme-selector-section">
        <label class="theme-selector-label">Theme</label>
        <div class="theme-options">
          <label 
            class="theme-option"
            :class="{ active: currentTheme === 'system' }"
          >
            <input 
              type="radio" 
              value="system" 
              :checked="currentTheme === 'system'"
              @change="handleThemeChange('system')"
            />
            <span class="theme-name">System</span>
            <span class="theme-preview">
              <span class="theme-swatch" style="background: #ffffff; border-color: #000000;"></span>
              <span class="theme-swatch" style="background: #1a1a1a; border-color: #ffffff;"></span>
              <span class="system-indicator" v-if="currentTheme === 'system'">
                ({{ systemPreference === 'dark' ? 'Dark' : 'Light' }})
              </span>
            </span>
          </label>
          <label 
            class="theme-option"
            :class="{ active: currentTheme === 'light' }"
          >
            <input 
              type="radio" 
              value="light" 
              :checked="currentTheme === 'light'"
              @change="handleThemeChange('light')"
            />
            <span class="theme-name">Light</span>
            <span class="theme-preview">
              <span class="theme-swatch" style="background: #ffffff;"></span>
            </span>
          </label>
          <label 
            class="theme-option"
            :class="{ active: currentTheme === 'dark' }"
          >
            <input 
              type="radio" 
              value="dark" 
              :checked="currentTheme === 'dark'"
              @change="handleThemeChange('dark')"
            />
            <span class="theme-name">Dark</span>
            <span class="theme-preview">
              <span class="theme-swatch" style="background: #1a1a1a;"></span>
            </span>
          </label>
          <label 
            class="theme-option"
            :class="{ active: currentTheme === 'blue' }"
          >
            <input 
              type="radio" 
              value="blue" 
              :checked="currentTheme === 'blue'"
              @change="handleThemeChange('blue')"
            />
            <span class="theme-name">Blue</span>
            <span class="theme-preview">
              <span class="theme-swatch" style="background: #2196f3;"></span>
            </span>
          </label>
          <label 
            class="theme-option"
            :class="{ active: currentTheme === 'liquid-glass' }"
          >
            <input 
              type="radio" 
              value="liquid-glass" 
              :checked="currentTheme === 'liquid-glass'"
              @change="handleThemeChange('liquid-glass')"
            />
            <span class="theme-name">Liquid Glass</span>
            <span class="theme-preview">
              <span class="theme-swatch" style="background: linear-gradient(135deg, rgba(255,255,255,0.7), rgba(255,255,255,0.3)); border: 1px solid rgba(255,255,255,0.3); backdrop-filter: blur(10px);"></span>
            </span>
          </label>
        </div>
        <p class="theme-description">
          Choose how UmbraRelay looks. System follows your OS preference.
        </p>
      </div>
    </SettingsSection>
  </div>
</template>

<script setup lang="ts">
import { useTheme } from '../../composables/useTheme';
import SettingsSection from '../base/SettingsSection.vue';

const { currentTheme, systemPreference, setTheme } = useTheme();

const handleThemeChange = async (theme: 'system' | 'light' | 'dark' | 'blue' | 'liquid-glass') => {
  try {
    await setTheme(theme);
  } catch (error) {
    console.error('Failed to change theme:', error);
    alert('Failed to change theme. Please try again.');
  }
};
</script>
