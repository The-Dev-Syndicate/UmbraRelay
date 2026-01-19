<template>
  <div class="custom-view-view">
    <div class="sticky-header">
      <div class="header-left">
        <h1>{{ viewName }}</h1>
        <button @click="editView" class="edit-view-button">Edit</button>
      </div>
      <div class="header-right">
        <div class="filters-wrapper">
          <div class="filters-container">
            <button
              v-if="!searchExpanded"
              @click="searchExpanded = true"
              class="search-toggle"
              title="Search articles"
            >
              <span class="search-icon">🔍</span>
            </button>
          </div>
          <div v-if="searchExpanded" class="search-bar-expanded">
            <span class="search-icon">🔍</span>
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Search articles..."
              class="search-input"
              ref="searchInput"
              @blur="handleSearchBlur"
            />
            <button
              @click="closeSearch"
              class="search-close"
              title="Close search"
            >
              ×
            </button>
          </div>
        </div>
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
import { ref, computed, onMounted, watch } from 'vue';
import { useItems } from '../../composables/useItems';
import { useCustomViews } from '../../composables/useCustomViews';
import type { CustomView } from '../../types';
import { formatDate, truncate, stripHtml, parseGroups } from '../../utils/formatting';

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
const searchQuery = ref('');
const searchExpanded = ref(false);
const searchInput = ref<HTMLInputElement | null>(null);

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
  let filtered = items.value;
  
  // Apply text search filter
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase();
    filtered = filtered.filter(item => {
      const title = item.title?.toLowerCase() || '';
      const summary = stripHtml(item.summary || '').toLowerCase();
      const sourceName = item.source_name?.toLowerCase() || '';
      const author = item.author?.toLowerCase() || '';
      
      return title.includes(query) || 
             summary.includes(query) || 
             sourceName.includes(query) ||
             author.includes(query);
    });
  }
  
  return filtered;
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


// Focus search input when expanded
watch(searchExpanded, (expanded) => {
  if (expanded && searchInput.value) {
    setTimeout(() => {
      searchInput.value?.focus();
    }, 100);
  }
});

const closeSearch = () => {
  searchExpanded.value = false;
  searchQuery.value = '';
};

const handleSearchBlur = (e: FocusEvent) => {
  // Don't close if clicking on the close button
  const relatedTarget = e.relatedTarget as HTMLElement;
  if (relatedTarget?.closest('.search-bar-expanded')) {
    return;
  }
  // Close if search is empty
  if (!searchQuery.value.trim()) {
    searchExpanded.value = false;
  }
};

onMounted(async () => {
  // Load view data
  view.value = await getCustomView(props.viewId);
  
  // Fetch items with view filters
  await fetchItems(undefined, undefined, sourceIds.value, groupNames.value);
});
</script>


