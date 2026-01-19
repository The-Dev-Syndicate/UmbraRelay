<template>
  <div class="app">
    <nav class="sidebar">
      <h1 class="app-title">UmbraRelay</h1>
      
      <!-- Search Input -->
      <div class="nav-search">
        <span class="nav-search-icon">🔍</span>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search sources and views..."
          class="nav-search-input"
        />
      </div>

      <div class="nav-links">
        <!-- Collections Section (always visible) -->
        <div class="nav-section">
          <div class="nav-section-header">Collections</div>
          <div class="nav-section-items">
            <button
              :class="{ active: currentView === 'today' }"
              @click="currentView = 'today'; selectedItemId = null"
              class="nav-button nav-sub-item"
            >
              <span class="nav-icon">☀️</span>
              <span>Today</span>
            </button>
            <button
              :class="{ active: currentView === 'all-items' }"
              @click="currentView = 'all-items'; selectedItemId = null"
              class="nav-button nav-sub-item"
            >
              <span class="nav-icon">📦</span>
              <span>All Items</span>
            </button>
            <button
              :class="{ active: currentView === 'leaving-today' }"
              @click="currentView = 'leaving-today'; selectedItemId = null"
              class="nav-button nav-sub-item"
            >
              <span class="nav-icon">⏰</span>
              <span>Leaving Today</span>
            </button>
            <button
              :class="{ active: currentView === 'trash' }"
              @click="currentView = 'trash'; selectedItemId = null"
              class="nav-button nav-sub-item"
            >
              <span class="nav-icon">🗑️</span>
              <span>Trash</span>
            </button>
          </div>
        </div>

        <!-- Sources Section (collapsible) -->
        <div class="nav-group">
          <button
            @click="sourcesExpanded = !sourcesExpanded"
            class="nav-group-header"
          >
            <span class="nav-icon">📡</span>
            <span>Sources</span>
            <span class="nav-arrow" :class="{ expanded: sourcesExpanded }">▼</span>
          </button>
          <div v-if="sourcesExpanded" class="nav-group-items nav-scrollable">
            <div
              v-for="source in filteredSources"
              :key="source.id"
              class="nav-source-item"
            >
              <button
                :class="{ active: currentView === `source-${source.id}` }"
                @click="currentView = `source-${source.id}`; selectedItemId = null"
                class="nav-button nav-sub-item"
              >
                {{ source.name }}
              </button>
            </div>
            <div v-if="filteredSources.length === 0" class="nav-empty-state">
              <span v-if="searchQuery">No sources match</span>
              <span v-else>No sources yet</span>
            </div>
          </div>
        </div>

        <!-- My Views Group (collapsible) -->
        <div class="nav-group">
          <button
            @click="myViewsExpanded = !myViewsExpanded"
            class="nav-group-header"
          >
            <span class="nav-icon">📋</span>
            <span>My Views</span>
            <span class="nav-arrow" :class="{ expanded: myViewsExpanded }">▼</span>
          </button>
          <div v-if="myViewsExpanded" class="nav-group-items nav-scrollable">
            <div
              v-for="view in filteredViews"
              :key="view.id"
              class="nav-view-item"
            >
              <button
                :class="{ active: currentView === view.id }"
                @click="currentView = view.id; selectedItemId = null"
                class="nav-button nav-sub-item"
              >
                {{ view.name }}
              </button>
              <button
                @click.stop="deleteView(view.id)"
                class="nav-delete-button"
                title="Delete view"
              >
                ×
              </button>
            </div>
            <div v-if="filteredViews.length === 0" class="nav-empty-state">
              <span v-if="searchQuery">No views match</span>
              <span v-else>No custom views yet</span>
            </div>
          </div>
        </div>

        <!-- Bottom Action Icons -->
        <div class="nav-spacer"></div>
        <div class="nav-bottom-actions">
          <button
            @click="openCreateView"
            class="nav-icon-button"
            data-tooltip="Add View"
            :class="{ active: showViewConfig }"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </button>
          <button
            @click="handleAddSource"
            class="nav-icon-button"
            data-tooltip="Add Source"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </button>
          <button
            @click="handleSyncAll"
            class="nav-icon-button"
            data-tooltip="Sync All Sources"
            :disabled="syncingAll"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :class="{ spinning: syncingAll }">
              <polyline points="23 4 23 10 17 10"></polyline>
              <polyline points="1 20 1 14 7 14"></polyline>
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
            </svg>
          </button>
          <button
            :class="{ active: currentView === 'sources' }"
            @click="currentView = 'sources'; selectedItemId = null"
            class="nav-icon-button"
            data-tooltip="Settings"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="3"></circle>
              <path d="M12 1v6m0 6v6M5.64 5.64l4.24 4.24m4.24 4.24l4.24 4.24M1 12h6m6 0h6M5.64 18.36l4.24-4.24m4.24-4.24l4.24-4.24"></path>
            </svg>
          </button>
        </div>
      </div>
    </nav>

    <main class="main-content">
      <TodayView
        v-if="currentView === 'today' && !selectedItemId"
        @select-item="selectItem"
        key="today"
      />
      <InboxView
        v-else-if="currentView === 'all-items' && !selectedItemId"
        @select-item="selectItem"
        key="all-items"
      />
      <TrashView
        v-else-if="currentView === 'trash' && !selectedItemId"
        @select-item="selectItem"
        key="trash"
      />
      <LeavingTodayView
        v-else-if="currentView === 'leaving-today' && !selectedItemId"
        @select-item="selectItem"
        key="leaving-today"
      />
      <InboxView
        v-else-if="currentViewType === 'source' && !selectedItemId"
        :source-id="currentSourceId ?? undefined"
        @select-item="selectItem"
        :key="`source-${currentSourceId}`"
      />
      <ItemDetail
        v-else-if="(currentView === 'today' || currentView === 'all-items' || currentView === 'trash' || currentView === 'leaving-today' || currentViewType === 'source' || currentView === 'leaving-soon' || typeof currentView === 'number') && selectedItemId"
        :item-id="selectedItemId"
        @back="selectedItemId = null"
        key="item-detail"
      />
      <LeavingSoonView
        v-else-if="currentView === 'leaving-soon' && !selectedItemId"
        @select-item="selectItem"
        key="leaving-soon"
      />
      <CustomView
        v-else-if="typeof currentView === 'number' && !selectedItemId"
        :view-id="currentView"
        @select-item="selectItem"
        @edit-view="openEditView(currentView)"
        key="custom-view"
      />
      <Config 
        v-else-if="currentView === 'sources'" 
        key="sources"
        ref="configRef"
      />
      <div v-else class="fallback">
        <p>Select a view from the sidebar</p>
      </div>
    </main>
    <CustomViewConfig
      v-if="showViewConfig"
      :view-id="editingViewId"
      @close="closeViewConfig"
      @saved="handleViewSaved"
    />
    <LinkHoverPreview />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch, nextTick } from 'vue';
import InboxView from './components/views/InboxView.vue';
import TodayView from './components/views/TodayView.vue';
import LeavingSoonView from './components/views/LeavingSoonView.vue';
import LeavingTodayView from './components/views/LeavingTodayView.vue';
import CustomView from './components/views/CustomView.vue';
import CustomViewConfig from './components/views/CustomViewConfig.vue';
import ItemDetail from './components/views/ItemDetail.vue';
import TrashView from './components/views/TrashView.vue';
import Config from './components/settings/Config.vue';
import LinkHoverPreview from './components/base/LinkHoverPreview.vue';
import { useCustomViews } from './composables/useCustomViews';
import { useSources } from './composables/useSources';
import { useTheme } from './composables/useTheme';

// Initialize theme system
useTheme();

type ViewType = 'today' | 'all-items' | 'trash' | 'leaving-today' | 'leaving-soon' | 'sources' | number | string;

const currentView = ref<ViewType>('today');
const selectedItemId = ref<number | null>(null);
const sourcesExpanded = ref(false);
const myViewsExpanded = ref(false);
const showViewConfig = ref(false);
const editingViewId = ref<number | null>(null);
const searchQuery = ref('');

// Store previous expand states to restore when search is cleared
const previousSourcesExpanded = ref(false);
const previousMyViewsExpanded = ref(false);

const { customViews, fetchCustomViews, deleteCustomView } = useCustomViews();
const { sources, fetchSources, syncAllSources } = useSources();

const configRef = ref<InstanceType<typeof Config> | null>(null);
const syncingAll = ref(false);

// Parse current view to determine if it's a source view
const currentViewType = computed(() => {
  if (typeof currentView.value === 'string' && currentView.value.startsWith('source-')) {
    return 'source';
  }
  return null;
});

const currentSourceId = computed(() => {
  if (currentViewType.value === 'source') {
    const match = (currentView.value as string).match(/^source-(\d+)$/);
    return match ? parseInt(match[1], 10) : null;
  }
  return null;
});

// Filter sources and views based on search query
const filteredSources = computed(() => {
  if (!searchQuery.value.trim()) {
    return sources.value;
  }
  const query = searchQuery.value.toLowerCase();
  return sources.value.filter(source => 
    source.name.toLowerCase().includes(query)
  );
});

const filteredViews = computed(() => {
  if (!searchQuery.value.trim()) {
    return customViews.value;
  }
  const query = searchQuery.value.toLowerCase();
  return customViews.value.filter(view => 
    view.name.toLowerCase().includes(query)
  );
});

const selectItem = (id: number) => {
  selectedItemId.value = id;
};

const openCreateView = () => {
  editingViewId.value = null;
  showViewConfig.value = true;
};

const openEditView = (id: number) => {
  editingViewId.value = id;
  showViewConfig.value = true;
};

const closeViewConfig = () => {
  showViewConfig.value = false;
  editingViewId.value = null;
};

const handleViewSaved = () => {
  fetchCustomViews();
  closeViewConfig();
};

const deleteView = async (id: number) => {
  if (confirm('Are you sure you want to delete this view? This will not delete any items.')) {
    try {
      await deleteCustomView(id);
      // If we're currently viewing this view, switch to today
      if (currentView.value === id) {
        currentView.value = 'today';
        selectedItemId.value = null;
      }
    } catch (e) {
      console.error('Failed to delete view:', e);
    }
  }
};

// Watch search query to expand/collapse sections
watch(searchQuery, (newQuery, oldQuery) => {
  const hasSearch = newQuery.trim().length > 0;
  const hadSearch = oldQuery && oldQuery.trim().length > 0;
  
  if (hasSearch && !hadSearch) {
    // Search just started - save current state and expand sections
    previousSourcesExpanded.value = sourcesExpanded.value;
    previousMyViewsExpanded.value = myViewsExpanded.value;
    
    // Expand sections if they have matching results
    if (filteredSources.value.length > 0) {
      sourcesExpanded.value = true;
    }
    if (filteredViews.value.length > 0) {
      myViewsExpanded.value = true;
    }
  } else if (!hasSearch && hadSearch) {
    // Search was cleared - restore previous state
    sourcesExpanded.value = previousSourcesExpanded.value;
    myViewsExpanded.value = previousMyViewsExpanded.value;
  } else if (hasSearch) {
    // Search query changed - keep sections expanded if they have results
    if (filteredSources.value.length > 0) {
      sourcesExpanded.value = true;
    }
    if (filteredViews.value.length > 0) {
      myViewsExpanded.value = true;
    }
  }
});

const handleAddSource = () => {
  // Navigate to sources page and trigger add source modal
  currentView.value = 'sources';
  selectedItemId.value = null;
  // Use nextTick to ensure Config is mounted before accessing ref
  nextTick(() => {
    if (configRef.value && 'openAddSourceModal' in configRef.value) {
      (configRef.value as any).openAddSourceModal();
    }
  });
};

const handleSyncAll = async () => {
  syncingAll.value = true;
  try {
    await syncAllSources();
    await fetchSources();
  } catch (e) {
    console.error('Failed to sync all sources:', e);
  } finally {
    syncingAll.value = false;
  }
};

onMounted(() => {
  console.log('UmbraRelay app mounted');
  fetchCustomViews();
  fetchSources();
});
</script>

