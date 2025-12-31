<template>
  <div class="item-detail">
    <div v-if="loading" class="loading">Loading...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <div v-else-if="item" class="item-content">
      <div class="header">
        <button @click="$emit('back')" class="back-button">← Back</button>
        <div class="actions">
          <button
            v-if="item.state === 'unread'"
            @click="updateState('read')"
            class="action-button"
          >
            Mark Read
          </button>
          <button
            v-else
            @click="updateState('unread')"
            class="action-button"
          >
            Mark Unread
          </button>
          <button
            v-if="item.state !== 'archived'"
            @click="updateState('archived')"
            class="action-button"
          >
            Archive
          </button>
          <button
            @click="openExternal"
            class="action-button primary"
          >
            Open Link
          </button>
        </div>
      </div>

      <div class="item-meta">
        <span class="item-type">{{ item.item_type }}</span>
        <span class="item-state">{{ item.state }}</span>
        <span class="item-date">{{ formatDate(item.created_at) }}</span>
      </div>

      <h1 class="item-title">{{ item.title }}</h1>

      <div v-if="hasValidSummary" class="item-summary">{{ cleanedSummary }}</div>
      <div v-else-if="item.summary" class="item-summary-raw">
        <p><em>Summary content filtered (may contain only links or minimal text)</em></p>
        <details>
          <summary style="cursor: pointer; color: #666; font-size: 14px;">Show raw content</summary>
          <pre style="margin-top: 8px; padding: 12px; background: #f5f5f5; border-radius: 4px; font-size: 12px; overflow-x: auto;">{{ item.summary }}</pre>
        </details>
      </div>
      <div v-else class="item-no-content">
        <p><strong>No summary available in RSS feed</strong></p>
        <p style="margin-top: 8px; font-size: 14px; color: #666;">
          Many RSS feeds only provide titles and links. Click "Open Link" above to view the full article content on the original website.
        </p>
      </div>

      <div class="item-url">
        <a :href="item.url" target="_blank" rel="noopener noreferrer">
          {{ item.url }}
        </a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { Item } from '../types';

defineEmits<{
  (e: 'back'): void;
}>();

const props = defineProps<{
  itemId: number;
}>();

const item = ref<Item | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

const fetchItem = async () => {
  loading.value = true;
  error.value = null;
  try {
    item.value = await invoke<Item>('get_item', { id: props.itemId });
  } catch (e) {
    error.value = e as string;
    console.error('Failed to fetch item:', e);
  } finally {
    loading.value = false;
  }
};

const updateState = async (state: 'unread' | 'read' | 'archived') => {
  if (!item.value) return;
  
  try {
    await invoke('update_item_state', { id: item.value.id, state });
    item.value.state = state;
  } catch (e) {
    error.value = e as string;
    console.error('Failed to update item state:', e);
  }
};

const openExternal = async () => {
  if (!item.value) return;
  
  try {
    await openUrl(item.value.url);
  } catch (e) {
    error.value = e as string;
    console.error('Failed to open URL:', e);
  }
};

const formatDate = (timestamp: number) => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
};

const stripHtml = (html: string) => {
  if (!html) return '';
  // Remove HTML tags and decode entities
  const tmp = document.createElement('div');
  tmp.innerHTML = html;
  return tmp.textContent || tmp.innerText || '';
};

const cleanedSummary = computed(() => {
  if (!item.value?.summary) return '';
  const cleaned = stripHtml(item.value.summary).trim();
  // Only filter out if it's exactly "Comments" or very short meaningless text
  const lower = cleaned.toLowerCase();
  // Filter out only exact matches or very short text that's likely just link text
  if (lower === 'comments' || lower === 'comment' || (cleaned.length < 3 && lower !== cleaned)) {
    return '';
  }
  return cleaned;
});

const hasValidSummary = computed(() => {
  return cleanedSummary.value.length > 0;
});

onMounted(() => {
  fetchItem();
});
</script>

<style scoped>
.item-detail {
  padding: 20px;
  max-width: 800px;
  margin: 0 auto;
}

.loading, .error {
  text-align: center;
  padding: 40px;
}

.error {
  color: #d32f2f;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.back-button {
  padding: 8px 16px;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
  border-radius: 4px;
}

.actions {
  display: flex;
  gap: 10px;
}

.action-button {
  padding: 8px 16px;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
  border-radius: 4px;
}

.action-button.primary {
  background: #396cd8;
  color: white;
  border-color: #396cd8;
}

.item-meta {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  font-size: 14px;
  color: #666;
}

.item-type, .item-state {
  text-transform: uppercase;
  font-weight: bold;
}

.item-state {
  color: #396cd8;
}

.item-title {
  margin: 0 0 20px 0;
  font-size: 28px;
  font-weight: 600;
  line-height: 1.3;
}

.item-summary {
  margin-bottom: 20px;
  line-height: 1.6;
  color: #333;
  font-size: 16px;
  white-space: pre-wrap;
}

.item-no-content {
  margin-bottom: 20px;
  padding: 20px;
  background: #f5f5f5;
  border-radius: 8px;
  text-align: center;
  color: #666;
  font-style: italic;
}

.item-url {
  margin-top: 20px;
  padding: 12px;
  background: #f5f5f5;
  border-radius: 4px;
}

.item-url a {
  color: #396cd8;
  text-decoration: none;
  word-break: break-all;
}

.item-url a:hover {
  text-decoration: underline;
}
</style>

