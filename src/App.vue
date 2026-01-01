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

        <!-- Sources and Add View (moved to bottom, horizontal) -->
        <div class="nav-spacer"></div>
        <div class="nav-bottom-actions">
          <button
            @click="openCreateView"
            class="nav-button nav-bottom"
            title="Add View"
          >
            <span class="nav-icon">➕</span>
            <span>Add View</span>
          </button>
          <button
            :class="{ active: currentView === 'sources' }"
            @click="currentView = 'sources'; selectedItemId = null"
            class="nav-button nav-bottom"
          >
            <span class="nav-icon">⚙️</span>
            <span>Sources</span>
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
      <InboxView
        v-else-if="currentViewType === 'source' && !selectedItemId"
        :source-id="currentSourceId"
        @select-item="selectItem"
        :key="`source-${currentSourceId}`"
      />
      <ItemDetail
        v-else-if="(currentView === 'today' || currentView === 'all-items' || currentViewType === 'source' || currentView === 'leaving-soon' || typeof currentView === 'number') && selectedItemId"
        :item-id="selectedItemId"
        @back="selectedItemId = null"
        key="item-detail"
      />
      <LeavingSoonView
        v-else-if="currentView === 'leaving-soon' && !selectedItemId"
        @select-item="selectItem"
        key="leaving-soon"
      />
      <CustomViewView
        v-else-if="typeof currentView === 'number' && !selectedItemId"
        :view-id="currentView"
        @select-item="selectItem"
        @edit-view="openEditView(currentView)"
        key="custom-view"
      />
      <SourceConfig v-else-if="currentView === 'sources'" key="sources" />
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
import { ref, onMounted, computed, watch } from 'vue';
import InboxView from './components/InboxView.vue';
import TodayView from './components/TodayView.vue';
import LeavingSoonView from './components/LeavingSoonView.vue';
import CustomViewView from './components/CustomViewView.vue';
import CustomViewConfig from './components/CustomViewConfig.vue';
import ItemDetail from './components/ItemDetail.vue';
import SourceConfig from './components/SourceConfig.vue';
import LinkHoverPreview from './components/LinkHoverPreview.vue';
import { useCustomViews } from './composables/useCustomViews';
import { useSources } from './composables/useSources';
import { useTheme } from './composables/useTheme';

// Initialize theme system
useTheme();

type ViewType = 'today' | 'all-items' | 'leaving-soon' | 'sources' | number | string;

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
const { sources, fetchSources } = useSources();

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

onMounted(() => {
  console.log('UmbraRelay app mounted');
  fetchCustomViews();
  fetchSources();
});
</script>

