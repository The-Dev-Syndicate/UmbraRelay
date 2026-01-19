<template>
  <div class="tabbed-view">
    <!-- Tab Navigation -->
    <div class="tabs-nav">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-button"
        :class="{ active: activeTab === tab.id }"
        @click="handleTabClick(tab.id)"
        :aria-selected="activeTab === tab.id"
        :aria-controls="`tab-pane-${tab.id}`"
        role="tab"
      >
        <span v-if="tab.icon" class="tab-icon">{{ tab.icon }}</span>
        <span>{{ tab.label }}</span>
      </button>
    </div>

    <!-- Tab Content -->
    <div class="tab-content">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        :id="`tab-pane-${tab.id}`"
        v-show="activeTab === tab.id"
        class="tab-pane"
        role="tabpanel"
        :aria-labelledby="`tab-${tab.id}`"
      >
        <slot :name="`tab-${tab.id}`" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';

export interface Tab {
  id: string;
  label: string;
  icon?: string;
}

interface Props {
  tabs: Tab[];
  defaultTab?: string;
}

const props = withDefaults(defineProps<Props>(), {
  defaultTab: undefined,
});

const emit = defineEmits<{
  'tab-change': [tabId: string];
}>();

const activeTab = ref<string>(props.defaultTab || props.tabs[0]?.id || '');

const handleTabClick = (tabId: string) => {
  activeTab.value = tabId;
  emit('tab-change', tabId);
};

// Watch for defaultTab changes
watch(
  () => props.defaultTab,
  (newTab) => {
    if (newTab && props.tabs.some((tab) => tab.id === newTab)) {
      activeTab.value = newTab;
    }
  }
);

// Initialize active tab on mount
onMounted(() => {
  if (props.defaultTab && props.tabs.some((tab) => tab.id === props.defaultTab)) {
    activeTab.value = props.defaultTab;
  } else if (props.tabs.length > 0) {
    activeTab.value = props.tabs[0].id;
  }
});

// Expose activeTab for parent access if needed
defineExpose({
  activeTab,
  setActiveTab: (tabId: string) => {
    if (props.tabs.some((tab) => tab.id === tabId)) {
      activeTab.value = tabId;
      emit('tab-change', tabId);
    }
  },
});
</script>
