<template>
  <div class="custom-view-view">
    <div class="header">
      <div class="header-left">
        <h1>{{ viewName }}</h1>
        <button @click="editView" class="edit-view-button">Edit</button>
      </div>
    </div>

    <div v-if="loading" class="loading">Loading...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <div v-else-if="filteredItems.length === 0" class="empty">
      No items found in this view.
    </div>
    <div v-else class="items-list">
      <div
        v-for="item in filteredItems"
        :key="item.id"
        class="item-card"
        :class="item.state"
        @click="selectItem(item.id)"
      >
        <div class="item-card-header">
          <span class="item-source-name">{{ item.source_name || 'Unknown Source' }}</span>
          <div class="item-badges">
            <span class="item-type-badge">{{ item.item_type.toUpperCase() }}</span>
            <span class="item-state-badge" :class="item.state">{{ item.state.toUpperCase() }}</span>
          </div>
        </div>
        <div class="item-content">
          <div class="item-content-wrapper">
            <div v-if="item.image_url" class="item-image-container">
              <img :src="item.image_url" :alt="item.title" class="item-image" />
            </div>
            <div class="item-text-content">
              <h3 class="item-title">{{ item.title }}</h3>
              <p v-if="item.summary && stripHtml(item.summary).trim() && stripHtml(item.summary).toLowerCase() !== 'comments'" class="item-summary">{{ truncate(stripHtml(item.summary), 200) }}</p>
            </div>
          </div>
        </div>
        <div class="item-footer">
          <span class="item-date">{{ formatDate(item.created_at) }}</span>
          <div v-if="item.source_group" class="item-groups">
            <span 
              v-for="group in parseGroups(item.source_group)" 
              :key="group" 
              class="group-tag"
            >
              {{ group }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useItems } from '../composables/useItems';
import { useCustomViews } from '../composables/useCustomViews';
import type { CustomView } from '../types';

const props = defineProps<{
  viewId: number;
}>();

const emit = defineEmits<{
  (e: 'select-item', id: number): void;
  (e: 'edit-view'): void;
}>();

const { items, loading, error, fetchItems, updateItemState } = useItems();
const { getCustomView } = useCustomViews();

const view = ref<CustomView | null>(null);
const viewName = computed(() => view.value?.name || 'Custom View');

// Parse JSON arrays from view
const sourceIds = computed(() => {
  if (!view.value?.source_ids) return undefined;
  try {
    return JSON.parse(view.value.source_ids) as number[];
  } catch {
    return undefined;
  }
});

const groupNames = computed(() => {
  if (!view.value?.group_names) return undefined;
  try {
    return JSON.parse(view.value.group_names) as string[];
  } catch {
    return undefined;
  }
});

const filteredItems = computed(() => {
  // Items are already filtered by backend, just return them
  return items.value;
});

const selectItem = async (id: number) => {
  // Mark item as read if it's currently unread
  const item = items.value.find(i => i.id === id);
  if (item && item.state === 'unread') {
    await updateItemState(id, 'read');
  }
  emit('select-item', id);
};

const editView = () => {
  emit('edit-view');
};

const truncate = (text: string, length: number) => {
  if (text.length <= length) return text;
  return text.substring(0, length) + '...';
};

const stripHtml = (html: string) => {
  if (!html) return '';
  const tmp = document.createElement('div');
  tmp.innerHTML = html;
  return tmp.textContent || tmp.innerText || '';
};

const formatDate = (timestamp: number) => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
};

const parseGroups = (groupString: string | null | undefined): string[] => {
  if (!groupString) return [];
  return groupString.split(',').map(g => g.trim()).filter(g => g.length > 0);
};

onMounted(async () => {
  // Load view data
  view.value = await getCustomView(props.viewId);
  
  // Fetch items with view filters
  await fetchItems(undefined, undefined, sourceIds.value, groupNames.value);
});
</script>

<style scoped>
.custom-view-view {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header h1 {
  margin: 0;
}

.edit-view-button {
  padding: 8px 16px;
  border: 1px solid #ddd;
  background: white;
  color: #333;
  cursor: pointer;
  border-radius: 4px;
  font-size: 14px;
  transition: all 0.2s;
}

.edit-view-button:hover {
  background: #f5f5f5;
  border-color: #bbb;
}

.loading, .error, .empty {
  text-align: center;
  padding: 40px;
  color: #666;
}

.error {
  color: #d32f2f;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.item-card {
  border: 1px solid #e0e0e0;
  border-radius: 12px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s ease;
  background: white;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.item-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
  border-color: #396cd8;
}

.item-card.unread {
  border-left: 4px solid #396cd8;
  background: #f8f9ff;
}

.item-card.read {
  opacity: 0.85;
}

.item-card.archived {
  opacity: 0.6;
}

.item-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #f0f0f0;
}

.item-source-name {
  font-size: 12px;
  font-weight: 600;
  color: #666;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.item-content {
  flex: 1;
}

.item-content-wrapper {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.item-image-container {
  flex-shrink: 0;
  width: 120px;
  height: 80px;
  overflow: hidden;
  border-radius: 8px;
  background: #f0f0f0;
}

.item-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.item-text-content {
  flex: 1;
  min-width: 0;
}

.item-badges {
  display: flex;
  gap: 8px;
  align-items: center;
}

.item-type-badge {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 4px 8px;
  border-radius: 4px;
  background: #f0f0f0;
  color: #666;
  letter-spacing: 0.5px;
}

.item-state-badge {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 4px 8px;
  border-radius: 4px;
  letter-spacing: 0.5px;
}

.item-state-badge.unread {
  background: #e3f2fd;
  color: #1976d2;
}

.item-state-badge.read {
  background: #e8f5e9;
  color: #388e3c;
}

.item-state-badge.archived {
  background: #f5f5f5;
  color: #757575;
}

.item-title {
  margin: 0 0 12px 0;
  font-size: 20px;
  font-weight: 600;
  line-height: 1.4;
  color: #1a1a1a;
}

.item-summary {
  margin: 0 0 12px 0;
  color: #666;
  line-height: 1.6;
  font-size: 14px;
}

.item-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: #999;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #f0f0f0;
}

.item-groups {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.group-tag {
  background: #e3f2fd;
  color: #1976d2;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
}
</style>

