<template>
  <div class="item-detail">
    <div v-if="loading" class="loading">Loading...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <div v-else-if="item" class="item-content">
      <div class="header sticky-header">
        <div class="header-left">
          <button @click="$emit('back')" class="back-button">← Back</button>
          <div class="item-meta-inline">
            <span class="item-type">{{ item.item_type }}</span>
            <span class="item-state">{{ item.state }}</span>
          </div>
        </div>
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
        <span class="item-date">{{ formatDate(item.created_at) }}</span>
      </div>

      <h1 class="item-title">{{ item.title }}</h1>

      <!-- RSS 2.0 metadata: author, categories, comments -->
      <div v-if="item.author || categories.length > 0 || item.comments" class="item-metadata">
        <div v-if="item.author" class="item-author">
          <span class="metadata-label">Author:</span>
          <span class="metadata-value">{{ item.author }}</span>
        </div>
        <div v-if="categories.length > 0" class="item-categories">
          <div class="category-tags-container" :class="{ 'expanded': categoriesExpanded }">
            <span v-for="cat in categories" :key="cat" class="category-tag">{{ cat }}</span>
          </div>
          <button 
            v-if="hasCategoryOverflow" 
            @click="categoriesExpanded = !categoriesExpanded" 
            class="category-expand-button"
            :aria-label="categoriesExpanded ? 'Collapse categories' : 'Expand categories'"
          >
            <span v-if="!categoriesExpanded">▼</span>
            <span v-else>▲</span>
          </button>
        </div>
        <div v-if="item.comments" class="item-comments">
          <button @click="openComments" class="comments-button">
            View Comments
          </button>
        </div>
      </div>

      <div v-if="item.image_url" class="item-image-container">
        <img :src="item.image_url" :alt="item.title" class="item-image" />
      </div>

      <!-- Show description as muted preview text if we have full content -->
      <div v-if="item.content_html && hasValidSummary" class="item-description-preview">
        {{ cleanedSummary }}
      </div>

      <!-- Show full HTML content if available - this should render as HTML, not text -->
      <div v-if="hasContentHtml" class="item-content-html-wrapper">
        <div class="item-content-html" v-html="decodedContentHtml"></div>
      </div>
      
      <!-- Debug: Show if content_html is expected but missing -->
      <div v-else-if="item.image_url" class="item-content-missing">
        <p><em>Full article content not available. This item may need to be re-synced to load the full content.</em></p>
        <p style="margin-top: 8px; font-size: 12px; color: #999;">
          If this feed supports full content (like Fox News), try syncing the source again.
        </p>
      </div>
      
      <!-- Fallback to summary if no content_html (show as plain text, not HTML) -->
      <template v-else>
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
      </template>

      <!-- <div class="item-url">
        <a :href="item.url" target="_blank" rel="noopener noreferrer">
          {{ item.url }}
        </a>
      </div> -->
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { Item } from '../types';
import { formatDate, stripHtml } from '../utils/formatting';

defineEmits<{
  (e: 'back'): void;
}>();

const props = defineProps<{
  itemId: number;
}>();

const item = ref<Item | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const categoriesExpanded = ref(false);

const fetchItem = async () => {
  loading.value = true;
  error.value = null;
  try {
    item.value = await invoke<Item>('get_item', { id: props.itemId });
    // Debug: log content_html
    if (item.value) {
      console.log('Item content_html exists:', !!item.value.content_html);
      console.log('Item content_html length:', item.value.content_html?.length || 0);
      console.log('Item content_html preview:', item.value.content_html?.substring(0, 200) || 'N/A');
    }
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

const hasContentHtml = computed(() => {
  return !!(item.value?.content_html && item.value.content_html.trim().length > 0);
});

// Decode HTML entities (fixes double-encoding issue)
// The RSS feed may have HTML entities like &lt; &gt; &quot; etc. that need to be decoded
// We decode entities but preserve the HTML structure
// Also filter out duplicate "Comments" links if we have a comments URL
const decodedContentHtml = computed(() => {
  if (!item.value?.content_html) return '';
  
  // Use a textarea element to decode HTML entities
  // This is the standard way to decode HTML entities in the browser
  const textarea = document.createElement('textarea');
  textarea.innerHTML = item.value.content_html;
  let decoded = textarea.value;
  
  // If we have a comments URL, remove "Comments" links from the content to avoid duplicates
  if (item.value.comments) {
    // Remove links that contain "comment" in the text (case insensitive)
    decoded = decoded.replace(/<a[^>]*>.*?[Cc]omment[s]?.*?<\/a>/gi, '');
    // Also remove standalone "Comments" text that might be leftover
    decoded = decoded.replace(/\b[Cc]omment[s]?\b(?=\s|$|\.|,)/g, '');
  }
  
  return decoded;
});

// Parse category JSON array string
const categories = computed(() => {
  if (!item.value?.category) return [];
  try {
    const parsed = JSON.parse(item.value.category);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
});

// Check if categories overflow (more than 3 tags typically fit in one line)
const hasCategoryOverflow = computed(() => {
  return categories.value.length > 3;
});

const openComments = async () => {
  if (!item.value?.comments) return;
  
  try {
    await openUrl(item.value.comments);
  } catch (e) {
    error.value = e as string;
    console.error('Failed to open comments URL:', e);
  }
};

onMounted(() => {
  fetchItem();
});
</script>


