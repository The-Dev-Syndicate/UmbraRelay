<template>
  <div class="config">
    <!-- Sticky Header -->
    <div class="sticky-header">
      <div class="header-left">
        <h1>Settings</h1>
      </div>
    </div>

    <!-- Tabbed View -->
    <TabbedView 
      :tabs="tabs" 
      :defaultTab="'sources'"
      @tab-change="handleTabChange"
    >
      <template #tab-sources>
        <SourcesTab ref="sourcesTabRef" />
      </template>
      <template #tab-privacy>
        <PrivacySecurityTab />
      </template>
      <template #tab-preferences>
        <PreferencesTab />
      </template>
    </TabbedView>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import TabbedView from '../base/TabbedView.vue';
import SourcesTab from './SourcesTab.vue';
import PrivacySecurityTab from './PrivacySecurityTab.vue';
import PreferencesTab from './PreferencesTab.vue';
import type { Tab } from '../base/TabbedView.vue';

const tabs: Tab[] = [
  { id: 'sources', label: 'Sources', icon: '📡' },
  { id: 'privacy', label: 'Privacy & Security', icon: '🔒' },
  { id: 'preferences', label: 'Preferences', icon: '⚙️' },
];

const sourcesTabRef = ref<InstanceType<typeof SourcesTab> | null>(null);

const handleTabChange = (_tabId: string) => {
  // Tab change handled by TabbedView component
};

// Expose method for parent component
defineExpose({
  openAddSourceModal: () => {
    if (sourcesTabRef.value && 'openAddSourceModal' in sourcesTabRef.value) {
      (sourcesTabRef.value as any).openAddSourceModal();
    }
  }
});
</script>

<style scoped>
/* Styles moved to src/styles/components/_config.scss */
</style>
