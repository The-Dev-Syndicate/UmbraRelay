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

<style scoped>
.item-detail {
  padding: 0;
  max-width: 800px;
  margin: 0 auto;
}

.item-content {
  padding: 20px;
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

.sticky-header {
  position: sticky;
  top: 0;
  z-index: 100;
  background: white;
  padding: 12px 20px;
  margin: -20px -20px 20px -20px;
  border-bottom: 1px solid #e0e0e0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  backdrop-filter: blur(10px);
  background: rgba(255, 255, 255, 0.95);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.item-meta-inline {
  display: flex;
  gap: 12px;
  font-size: 14px;
}

.item-meta-inline .item-type,
.item-meta-inline .item-state {
  text-transform: uppercase;
  font-weight: bold;
}

.item-meta-inline .item-state {
  color: #396cd8;
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

.item-metadata {
  margin-bottom: 20px;
  padding: 12px 0;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.item-author {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.item-categories {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 14px;
}

.metadata-label {
  font-weight: 500;
  color: #666;
}

.metadata-value {
  color: #1a1a1a;
}

.category-tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  max-width: 500px;
  overflow: hidden;
  max-height: 32px; /* Approximately one line of tags */
  transition: max-height 0.3s ease;
}

.category-tags-container.expanded {
  max-height: none;
}

.category-tag {
  display: inline-block;
  padding: 4px 10px;
  background: #e3f2fd;
  color: #1976d2;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.category-expand-button {
  padding: 4px 8px;
  background: transparent;
  border: none;
  color: #666;
  cursor: pointer;
  font-size: 10px;
  line-height: 1;
  margin-top: 2px;
  transition: color 0.2s ease;
}

.category-expand-button:hover {
  color: #396cd8;
}

.item-comments {
  margin-top: 4px;
}

.comments-button {
  padding: 6px 14px;
  background: #f5f5f5;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  font-size: 13px;
  color: #396cd8;
  cursor: pointer;
  transition: all 0.2s ease;
}

.comments-button:hover {
  background: #e8f0fe;
  border-color: #396cd8;
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

.item-image-container {
  margin-bottom: 24px;
  border-radius: 8px;
  overflow: hidden;
  background: #f0f0f0;
}

.item-image {
  width: 100%;
  height: auto;
  display: block;
  max-height: 500px;
  object-fit: contain;
}

.item-content-html-wrapper {
  margin-bottom: 24px;
}

.item-content-html {
  line-height: 1.8;
  color: #1a1a1a;
  font-size: 16px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

.item-content-html :deep(p) {
  margin-bottom: 20px;
  margin-top: 0;
  line-height: 1.8;
  color: #1a1a1a;
}

.item-content-html :deep(p:first-child) {
  margin-top: 0;
}

.item-content-html :deep(p:last-child) {
  margin-bottom: 0;
}

.item-content-html :deep(a) {
  color: #396cd8;
  text-decoration: none;
  border-bottom: 1px solid transparent;
  transition: border-color 0.2s ease;
}

.item-content-html :deep(a:hover) {
  border-bottom-color: #396cd8;
  text-decoration: none;
}

.item-content-html :deep(a strong),
.item-content-html :deep(strong a) {
  font-weight: 700;
  color: #2952b8;
}

.item-content-html :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 8px;
  margin: 24px 0;
  display: block;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.item-content-html :deep(h1),
.item-content-html :deep(h2),
.item-content-html :deep(h3),
.item-content-html :deep(h4),
.item-content-html :deep(h5),
.item-content-html :deep(h6) {
  margin-top: 32px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.3;
  color: #1a1a1a;
}

.item-content-html :deep(h1) {
  font-size: 28px;
  margin-top: 0;
}

.item-content-html :deep(h2) {
  font-size: 24px;
}

.item-content-html :deep(h3) {
  font-size: 20px;
}

.item-content-html :deep(h4) {
  font-size: 18px;
}

.item-content-html :deep(ul),
.item-content-html :deep(ol) {
  margin-left: 24px;
  margin-bottom: 20px;
  padding-left: 8px;
}

.item-content-html :deep(li) {
  margin-bottom: 8px;
  line-height: 1.8;
}

.item-content-html :deep(blockquote) {
  border-left: 4px solid #396cd8;
  padding: 16px 20px;
  margin: 24px 0;
  background: #f8f9fa;
  border-radius: 4px;
  color: #555;
  font-style: italic;
  line-height: 1.7;
}

.item-content-html :deep(blockquote p) {
  margin-bottom: 12px;
}

.item-content-html :deep(blockquote p:last-child) {
  margin-bottom: 0;
}

.item-content-html :deep(strong),
.item-content-html :deep(b) {
  font-weight: 600;
  color: #1a1a1a;
}

.item-content-html :deep(em),
.item-content-html :deep(i) {
  font-style: italic;
}

.item-content-html :deep(code) {
  background: #f5f5f5;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 14px;
  color: #d32f2f;
}

.item-content-html :deep(pre) {
  background: #f5f5f5;
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 20px 0;
  line-height: 1.6;
}

.item-content-html :deep(pre code) {
  background: none;
  padding: 0;
  color: inherit;
}

.item-content-html :deep(hr) {
  border: none;
  border-top: 1px solid #e0e0e0;
  margin: 32px 0;
}

.item-content-html :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 20px 0;
}

.item-content-html :deep(table th),
.item-content-html :deep(table td) {
  padding: 12px;
  border: 1px solid #e0e0e0;
  text-align: left;
}

.item-content-html :deep(table th) {
  background: #f8f9fa;
  font-weight: 600;
}

.item-description-preview {
  margin-bottom: 20px;
  padding: 12px 16px;
  background: #f8f9fa;
  border-left: 3px solid #396cd8;
  border-radius: 4px;
  color: #666;
  font-size: 14px;
  line-height: 1.5;
  font-style: italic;
}

.item-content-missing {
  margin-bottom: 20px;
  padding: 16px;
  background: #fff3cd;
  border: 1px solid #ffc107;
  border-radius: 4px;
  color: #856404;
  font-size: 14px;
}
</style>

