<template>
  <div class="inbox-view">
    <div class="sticky-header">
      <div class="header-left">
        <h1>Trash</h1>
        <p class="subtitle">Items marked for deletion. They will be permanently removed after 30 days.</p>
        <div v-if="selectedItems.size > 0" class="selection-info">
          {{ selectedItems.size }} item{{ selectedItems.size !== 1 ? 's' : '' }} selected
        </div>
      </div>
      <div class="header-right">
        <div class="filters-wrapper">
          <div class="filters-container">
            <div v-if="selectedItems.size > 0" class="bulk-actions">
              <button @click="handleBulkRecover" class="bulk-action-btn" title="Recover Items">
                Recover
              </button>
              <button @click="handleBulkPermanentDelete" class="bulk-action-btn delete" title="Permanently Delete">
                Delete Forever
              </button>
              <button @click="clearSelection" class="bulk-action-btn" title="Clear Selection">
                Clear
              </button>
            </div>
            <label v-else class="select-all-checkbox">
              <input 
                type="checkbox" 
                :checked="allItemsSelected"
                @change="toggleSelectAll"
              />
              <span>Select All</span>
            </label>
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
      No items in trash.
    </div>
    <div v-else class="items-list">
      <div
        v-for="item in paginatedItems"
        :key="item.id"
        class="item-card"
        :class="[item.state, { selected: selectedItems.has(item.id) }]"
        @click="handleItemClick(item.id, $event)"
      >
        <label class="item-checkbox" @click.stop>
          <input 
            type="checkbox" 
            :checked="selectedItems.has(item.id)"
            @change="toggleItemSelection(item.id)"
          />
        </label>
        <div class="item-card-header">
          <span class="item-source-name">{{ item.source_name || 'Unknown Source' }}</span>
          <div class="item-badges">
            <span class="item-type-badge">{{ item.item_type.toUpperCase() }}</span>
            <span class="item-state-badge deleted">Marked for Deletion</span>
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
    <PaginationControls
      v-if="filteredItems.length > 0"
      :current-page="currentPage"
      :total-pages="totalPages"
      :total-items="filteredItems.length"
      :items-per-page="itemsPerPage"
      :items-per-page-options="itemsPerPageOptions"
      @go-to-page="goToPage"
      @next-page="nextPage"
      @previous-page="previousPage"
      @items-per-page-change="setItemsPerPage"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useItems } from '../../composables/useItems';
import { usePagination } from '../../composables/usePagination';
import PaginationControls from '../base/PaginationControls.vue';
import { formatDate, truncate, stripHtml, parseGroups } from '../../utils/formatting';

const emit = defineEmits<{
  (e: 'select-item', id: number): void;
}>();

const { items, loading, error, fetchItems, bulkUpdateItemState } = useItems();
const searchQuery = ref('');
const searchExpanded = ref(false);
const searchInput = ref<HTMLInputElement | null>(null);
const selectedItems = ref<Set<number>>(new Set());

// Fetch only deleted items
const filteredItems = computed(() => {
  let filtered = items.value.filter(item => item.state === 'deleted');
  
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

// Pagination
const {
  currentPage,
  totalPages,
  paginatedItems,
  itemsPerPage,
  itemsPerPageOptions,
  goToPage,
  nextPage,
  previousPage,
  resetPage,
  setItemsPerPage,
  checkPageBounds,
} = usePagination(() => filteredItems.value);

// Reset pagination when search changes
watch(() => searchQuery.value, () => {
  resetPage();
});

// Check page bounds when filtered items change
watch(() => filteredItems.value.length, () => {
  checkPageBounds();
});

const handleItemClick = (id: number, event: MouseEvent) => {
  // If clicking on checkbox, don't trigger item selection
  const target = event.target as HTMLElement;
  if (target.closest('.item-checkbox') || target.closest('input[type="checkbox"]')) {
    return;
  }
  emit('select-item', id);
};

const toggleItemSelection = (id: number) => {
  if (selectedItems.value.has(id)) {
    selectedItems.value.delete(id);
  } else {
    selectedItems.value.add(id);
  }
};

const toggleSelectAll = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.checked) {
    filteredItems.value.forEach(item => {
      selectedItems.value.add(item.id);
    });
  } else {
    selectedItems.value.clear();
  }
};

const allItemsSelected = computed(() => {
  return filteredItems.value.length > 0 && 
         filteredItems.value.every(item => selectedItems.value.has(item.id));
});

const clearSelection = () => {
  selectedItems.value.clear();
};

const handleBulkRecover = async () => {
  if (selectedItems.value.size === 0) return;
  const ids = Array.from(selectedItems.value);
  try {
    // Recover items by marking them as read
    await bulkUpdateItemState(ids, 'read');
    clearSelection();
    // Items are automatically removed from list when recovered (handled in useItems)
    // But we need to refetch to ensure the list is updated
    await fetchItems('deleted');
  } catch (e) {
    console.error('Failed to recover items:', e);
    alert('Failed to recover items. Please try again.');
  }
};

const handleBulkPermanentDelete = async () => {
  if (selectedItems.value.size === 0) return;
  
  const confirmed = confirm(
    `Are you sure you want to permanently delete ${selectedItems.value.size} item${selectedItems.value.size !== 1 ? 's' : ''}? ` +
    `This action cannot be undone.`
  );
  
  if (!confirmed) return;
  
  const ids = Array.from(selectedItems.value);
  try {
    // For now, we'll just keep them as deleted - actual permanent deletion happens via cleanup task
    // But we could add a command to permanently delete if needed
    alert('Permanent deletion will be handled by the cleanup task. Items marked for deletion will be removed after 30 days.');
    clearSelection();
  } catch (e) {
    console.error('Failed to permanently delete items:', e);
    alert('Failed to permanently delete items. Please try again.');
  }
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

onMounted(() => {
  // Fetch deleted items specifically
  fetchItems('deleted');
});
</script>
