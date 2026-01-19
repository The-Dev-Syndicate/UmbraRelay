<template>
  <div class="inbox-view">
    <div class="sticky-header">
      <div class="header-left">
        <h1>{{ sourceName }}</h1>
      </div>
      <div class="header-right">
        <div class="filters-wrapper">
          <div class="filters-container">
            <button
              v-if="sourceId !== undefined"
              @click="handleSyncSource"
              class="sync-source-button"
              :disabled="syncing"
              :title="syncing ? 'Syncing...' : 'Sync Source'"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :class="{ spinning: syncing }">
                <polyline points="23 4 23 10 17 10"></polyline>
                <polyline points="1 20 1 14 7 14"></polyline>
                <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
              </svg>
            </button>
            <div class="filters">
              <button
                v-for="filter in filters"
                :key="filter"
                :class="{ active: currentFilter === filter }"
                @click="setFilter(filter)"
              >
                {{ filter }}
              </button>
            </div>
            <div class="group-filter">
              <div class="group-filter-dropdown" :class="{ open: showGroupDropdown }">
                <button 
                  @click="showGroupDropdown = !showGroupDropdown" 
                  class="group-filter-button"
                >
                  <span v-if="selectedGroups.length === 0">All Groups</span>
                  <span v-else>{{ selectedGroups.length }} selected</span>
                  <span class="dropdown-arrow">▼</span>
                </button>
                <div v-if="showGroupDropdown" class="group-filter-menu">
                  <div class="group-filter-header">
                    <span>Filter by Groups</span>
                    <button @click="clearAllGroups" class="clear-all-btn">Clear All</button>
                  </div>
                  <div class="group-filter-options">
                    <label 
                      v-for="group in availableGroups" 
                      :key="group" 
                      class="group-option"
                    >
                      <input 
                        type="checkbox" 
                        :value="group"
                        v-model="selectedGroups"
                        @change="applyGroupFilter"
                      />
                      <span class="group-option-label">{{ group }}</span>
                    </label>
                  </div>
                </div>
              </div>
            </div>
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
      No items found.
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
import { useSources } from '../../composables/useSources';
import { formatDate, truncate, stripHtml, parseGroups } from '../../utils/formatting';

const props = defineProps<{
  sourceId?: number;
}>();

const emit = defineEmits<{
  (e: 'select-item', id: number): void;
}>();

const { items, loading, error, fetchItems, updateItemState } = useItems();
const { sources, fetchSources, syncSource } = useSources();
const syncing = ref(false);
const currentFilter = ref<string | null>(null);
const selectedGroups = ref<string[]>([]);
const showGroupDropdown = ref(false);
const searchQuery = ref('');
const searchExpanded = ref(false);
const searchInput = ref<HTMLInputElement | null>(null);

const filters = ['All', 'Unread', 'Read', 'Archived'];

// Get source name if filtering by source
const sourceName = computed(() => {
  if (props.sourceId !== undefined) {
    const source = sources.value.find(s => s.id === props.sourceId);
    return source?.name || 'Source';
  }
  return 'All Items';
});

// Get available groups from items' source_group field
const availableGroups = computed(() => {
  const groups = new Set<string>();
  items.value.forEach(item => {
    if (item.source_group) {
      const parsed = parseGroups(item.source_group);
      parsed.forEach(g => groups.add(g));
    }
  });
  return Array.from(groups).sort();
});

const filteredItems = computed(() => {
  let filtered = items.value;
  
  // Apply source filter if sourceId prop is provided
  if (props.sourceId !== undefined) {
    filtered = filtered.filter(item => item.source_id === props.sourceId);
  }
  
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
  
  // Apply state filter
  if (currentFilter.value) {
    filtered = filtered.filter(item => item.state === currentFilter.value);
  }
  
  // Apply group filter (multi-select)
  if (selectedGroups.value.length > 0) {
    filtered = filtered.filter(item => {
      if (!item.source_group) return false;
      const itemGroups = parseGroups(item.source_group);
      // Check if any of the selected groups match any of the item's groups
      return selectedGroups.value.some(selectedGroup => 
        itemGroups.includes(selectedGroup)
      );
    });
  }
  
  return filtered;
});

const setFilter = (filter: string) => {
  const filterValue = filter === 'All' ? null : filter.toLowerCase();
  currentFilter.value = filterValue;
  // Fetch all items, filtering will be done in computed
  fetchItems();
};

const applyGroupFilter = () => {
  // Filtering is done in computed, no need to fetch again
  // Just close dropdown after a short delay
  setTimeout(() => {
    if (selectedGroups.value.length === 0) {
      showGroupDropdown.value = false;
    }
  }, 200);
};


const clearAllGroups = () => {
  selectedGroups.value = [];
  showGroupDropdown.value = false;
};

const selectItem = async (id: number) => {
  // Mark item as read if it's currently unread
  const item = items.value.find(i => i.id === id);
  if (item && item.state === 'unread') {
    await updateItemState(id, 'read');
  }
  emit('select-item', id);
};


onMounted(() => {
  fetchSources();
  loadItems();
  
  // Close dropdown when clicking outside
  document.addEventListener('click', (e) => {
    const target = e.target as HTMLElement;
    if (!target.closest('.group-filter')) {
      showGroupDropdown.value = false;
    }
  });
});

const loadItems = () => {
  if (props.sourceId !== undefined) {
    fetchItems(undefined, undefined, [props.sourceId]);
  } else {
    fetchItems();
  }
};

// Watch for sourceId changes
watch(() => props.sourceId, () => {
  loadItems();
});

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

const handleSyncSource = async () => {
  if (props.sourceId === undefined || syncing.value) return;
  
  syncing.value = true;
  try {
    await syncSource(props.sourceId);
    await fetchItems();
  } catch (e) {
    console.error('Failed to sync source:', e);
  } finally {
    syncing.value = false;
  }
};
</script>


