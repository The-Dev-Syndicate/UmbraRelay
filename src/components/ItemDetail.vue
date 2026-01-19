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

      <!-- Title -->
      <h1 class="item-title">
        {{ displayTitle }}
        <span 
          v-if="contentResolution.isFetching" 
          class="extraction-spinner" 
          title="Fetching Full Content"
        >
          ⏳
        </span>
        <span 
          v-if="contentResolution.source === 'extracted'" 
          class="content-source-badge extracted"
          title="Full article content extracted"
        >
          Full Article
        </span>
        <span 
          v-else-if="contentResolution.source === 'feed' && item.content_html"
          class="content-source-badge"
          title="Content from feed"
        >
          From Feed
        </span>
      </h1>

      <!-- Subheading -->
      <div v-if="itemSubheading" class="item-subheading">{{ itemSubheading }}</div>

      <!-- Expandable Metadata Section -->
      <div class="metadata-section">
        <button 
          @click="metadataExpanded = !metadataExpanded" 
          class="metadata-toggle"
          :aria-expanded="metadataExpanded"
          :aria-label="metadataExpanded ? 'Collapse metadata' : 'Expand metadata'"
        >
          <span class="metadata-toggle-text">Metadata</span>
          <span class="metadata-toggle-icon">{{ metadataExpanded ? '▲' : '▼' }}</span>
        </button>
        <div v-if="metadataExpanded" class="metadata-content">
          <div class="metadata-grid">
            <!-- Source Information -->
            <div v-if="item.source_name" class="metadata-item">
              <span class="metadata-item-label">Source:</span>
              <span class="metadata-item-value">{{ item.source_name }}</span>
            </div>
            <div v-if="item.source_group" class="metadata-item">
              <span class="metadata-item-label">Group:</span>
              <span class="metadata-item-value">{{ item.source_group }}</span>
            </div>
            <div class="metadata-item">
              <span class="metadata-item-label">Type:</span>
              <span class="metadata-item-value">{{ formattedItemType || item.item_type }}</span>
            </div>
            <div class="metadata-item">
              <span class="metadata-item-label">State:</span>
              <span class="metadata-item-value metadata-state" :class="`state-${item.state}`">{{ item.state }}</span>
            </div>

            <!-- Publication Info -->
            <div class="metadata-item">
              <span class="metadata-item-label">Published:</span>
              <span class="metadata-item-value">{{ formatDate(item.created_at) }}</span>
            </div>
            <div v-if="item.updated_at && item.updated_at !== item.created_at" class="metadata-item">
              <span class="metadata-item-label">Updated:</span>
              <span class="metadata-item-value">{{ formatDate(item.updated_at) }}</span>
            </div>

            <!-- Author -->
            <div v-if="item.author" class="metadata-item">
              <span class="metadata-item-label">Author:</span>
              <span class="metadata-item-value">{{ item.author }}</span>
            </div>

            <!-- GitHub-specific Info -->
            <div v-if="githubNotificationInfo" class="metadata-item">
              <span class="metadata-item-label">Notification Type:</span>
              <span class="metadata-item-value">{{ githubNotificationInfo.type }}</span>
            </div>
            <div v-if="githubNotificationInfo?.reason" class="metadata-item">
              <span class="metadata-item-label">Reason:</span>
              <span class="metadata-item-value">{{ githubNotificationInfo.reason }}</span>
            </div>
            <div v-if="githubRepo" class="metadata-item">
              <span class="metadata-item-label">Repository:</span>
              <a :href="`https://github.com/${githubRepo}`" target="_blank" rel="noopener noreferrer" class="metadata-link">{{ githubRepo }}</a>
            </div>

            <!-- Categories -->
            <div v-if="categories.length > 0" class="metadata-item metadata-item-full">
              <span class="metadata-item-label">Categories:</span>
              <div class="metadata-categories">
                <span v-for="cat in categories" :key="cat" class="metadata-category-tag">{{ cat }}</span>
              </div>
            </div>

            <!-- External ID (for debugging) -->
            <div class="metadata-item">
              <span class="metadata-item-label">ID:</span>
              <span class="metadata-item-value metadata-id">{{ item.external_id }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Image (if available) -->
      <div v-if="item.image_url" class="item-image-container">
        <img :src="item.image_url" :alt="item.title" class="item-image" />
      </div>

      <!-- Formatted Content Section -->
      <div class="item-content-formatted">
        <!-- Error message if extraction failed -->
        <div v-if="contentResolution.hasError" class="extraction-error">
          <p><em>⚠️ {{ contentResolution.errorMessage || 'Failed to extract full content' }}</em></p>
          <p style="margin-top: 8px; font-size: 12px; color: #666;">
            Showing feed content instead.
          </p>
        </div>

        <!-- Show resolved content -->
        <div v-if="resolvedContentHtml" class="item-content-html-wrapper">
          <div class="item-content-html" v-html="resolvedContentHtml"></div>
        </div>
        
        <!-- Fallback to summary if no content -->
        <div v-else-if="hasValidSummary" class="item-summary">{{ cleanedSummary }}</div>
        <div v-else class="item-no-content">
          <p><strong>No content available</strong></p>
          <p style="margin-top: 8px; font-size: 14px; color: #666;">
            Click "Open Link" above to view the full article content on the original website.
          </p>
        </div>
      </div>

      <!-- Footer -->
      <div class="item-footer">
        <a 
          :href="item.url" 
          target="_blank" 
          rel="noopener noreferrer"
          @click.prevent="openExternal"
          class="footer-link"
        >
          View Original Article
        </a>
        <a 
          v-if="item.comments"
          :href="item.comments" 
          target="_blank" 
          rel="noopener noreferrer"
          @click.prevent="openComments"
          class="footer-link"
        >
          View Comments
        </a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { Item } from '../types';
import { 
  formatDate, 
  stripHtml, 
  parseGitHubNotificationSummary,
  extractGitHubRepo
} from '../utils/formatting';
import { useContent } from '../composables/useContent';

defineEmits<{
  (e: 'back'): void;
}>();

const props = defineProps<{
  itemId: number;
}>();

const item = ref<Item | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const metadataExpanded = ref(false);

// Content resolution
const { loadPreferences, getDisplayContent } = useContent();
const contentResolution = computed(() => getDisplayContent(item.value));

// Polling interval for extraction status
let pollingInterval: number | null = null;

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

// Start/stop polling when extraction is in progress
watch(() => contentResolution.value.isFetching, (isFetching) => {
  if (isFetching) {
    startPolling();
  } else {
    stopPolling();
  }
});

const startPolling = () => {
  stopPolling(); // Clear any existing interval
  pollingInterval = window.setInterval(() => {
    fetchItem();
  }, 2000); // Poll every 2 seconds
};

const stopPolling = () => {
  if (pollingInterval !== null) {
    clearInterval(pollingInterval);
    pollingInterval = null;
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

// Keep for backward compatibility but use resolved content instead
const hasContentHtml = computed(() => {
  return resolvedContentHtml.value.length > 0;
});

// Get resolved content HTML
const resolvedContentHtml = computed(() => {
  const content = contentResolution.value.content;
  if (!content) return '';
  
  // Decode HTML entities
  const textarea = document.createElement('textarea');
  textarea.innerHTML = content;
  let decoded = textarea.value;
  
  // If we have a comments URL, remove "Comments" links from the content to avoid duplicates
  if (item.value?.comments) {
    decoded = decoded.replace(/<a[^>]*>.*?[Cc]omment[s]?.*?<\/a>/gi, '');
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

// GitHub-specific computed properties
const isGitHubNotification = computed(() => {
  return item.value?.item_type === 'notification';
});

const isGitHubEvent = computed(() => {
  return item.value?.item_type === 'event' || 
         (item.value?.summary?.includes('Event type:') ?? false);
});

const isGitHubItem = computed(() => {
  return ['notification', 'event', 'issue', 'pr', 'commit'].includes(item.value?.item_type || '');
});

const githubNotificationInfo = computed(() => {
  if (!item.value?.summary) return null;
  return parseGitHubNotificationSummary(item.value.summary);
});

const githubRepo = computed(() => {
  if (!item.value) return null;
  return extractGitHubRepo(item.value);
});

const formattedItemType = computed(() => {
  if (!item.value) return '';
  const type = item.value.item_type;
  const typeMap: Record<string, string> = {
    'issue': 'Issue',
    'pr': 'Pull Request',
    'commit': 'Commit',
    'notification': 'Notification',
    'event': 'Event',
  };
  return typeMap[type] || type.charAt(0).toUpperCase() + type.slice(1);
});

const displayTitle = computed(() => {
  if (!item.value) return '';
  
  // For GitHub events, clean up the title (remove "PullRequestEvent: repo/name" format)
  if (isGitHubEvent.value && item.value.title.includes(':')) {
    const parts = item.value.title.split(':');
    if (parts.length > 1) {
      // Return just the repo name, or a cleaner format
      return parts[1].trim() || item.value.title;
    }
  }
  
  return item.value.title;
});

// Subheading based on source type
const itemSubheading = computed(() => {
  if (!item.value) return '';
  
  const parts: string[] = [];
  
  // For GitHub items, prefer repository name
  if (isGitHubItem.value) {
    if (githubRepo.value) {
      parts.push(githubRepo.value);
    } else if (item.value.source_name) {
      parts.push(item.value.source_name);
    }
  } else {
    // For RSS/Atom items
    if (item.value.author) {
      parts.push(item.value.author);
    }
    if (item.value.source_name) {
      parts.push(item.value.source_name);
    }
  }
  
  // Add source group if available
  if (item.value.source_group) {
    parts.push(`(${item.value.source_group})`);
  }
  
  return parts.join(' • ');
});

onMounted(async () => {
  await loadPreferences();
  await fetchItem();
});

onUnmounted(() => {
  stopPolling();
});
</script>


