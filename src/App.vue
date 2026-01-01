<template>
  <div class="app">
    <nav class="sidebar">
      <h1 class="app-title">UmbraRelay</h1>
      <div class="nav-links">
        <!-- Inbox Group -->
        <div class="nav-group">
          <button
            @click="inboxExpanded = !inboxExpanded"
            class="nav-group-header"
          >
            <span class="nav-icon">📥</span>
            <span>Inbox</span>
            <span class="nav-arrow" :class="{ expanded: inboxExpanded }">▼</span>
          </button>
          <div v-if="inboxExpanded" class="nav-group-items">
            <button
              :class="{ active: currentView === 'inbox' }"
              @click="currentView = 'inbox'; selectedItemId = null"
              class="nav-button nav-sub-item"
            >
              Inbox
            </button>
            <button
              :class="{ active: currentView === 'leaving-soon' }"
              @click="currentView = 'leaving-soon'; selectedItemId = null"
              class="nav-button nav-sub-item"
            >
              Leaving Soon
            </button>
          </div>
        </div>

        <!-- My Views Group -->
        <div class="nav-group">
          <button
            @click="myViewsExpanded = !myViewsExpanded"
            class="nav-group-header"
          >
            <span class="nav-icon">📋</span>
            <span>My Views</span>
            <span class="nav-arrow" :class="{ expanded: myViewsExpanded }">▼</span>
          </button>
          <div v-if="myViewsExpanded" class="nav-group-items">
            <div
              v-for="view in customViews"
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
            <div v-if="customViews.length === 0" class="nav-empty-state">
              No custom views yet
            </div>
          </div>
        </div>

        <!-- Sources and Add View (moved to bottom) -->
        <div class="nav-spacer"></div>
        <div class="nav-bottom-actions">
          <button
            @click="openCreateView"
            class="nav-button nav-bottom nav-add-view"
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
      <InboxView
        v-if="currentView === 'inbox' && !selectedItemId"
        @select-item="selectItem"
        key="inbox"
      />
      <ItemDetail
        v-else-if="(currentView === 'inbox' || currentView === 'leaving-soon' || typeof currentView === 'number') && selectedItemId"
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
import { ref, onMounted } from 'vue';
import InboxView from './components/InboxView.vue';
import LeavingSoonView from './components/LeavingSoonView.vue';
import CustomViewView from './components/CustomViewView.vue';
import CustomViewConfig from './components/CustomViewConfig.vue';
import ItemDetail from './components/ItemDetail.vue';
import SourceConfig from './components/SourceConfig.vue';
import LinkHoverPreview from './components/LinkHoverPreview.vue';
import { useCustomViews } from './composables/useCustomViews';
import { useTheme } from './composables/useTheme';

// Initialize theme system
useTheme();

const currentView = ref<'inbox' | 'leaving-soon' | 'sources' | number>('inbox');
const selectedItemId = ref<number | null>(null);
const inboxExpanded = ref(false);
const myViewsExpanded = ref(false);
const showViewConfig = ref(false);
const editingViewId = ref<number | null>(null);

const { customViews, fetchCustomViews, deleteCustomView } = useCustomViews();

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
      // If we're currently viewing this view, switch to inbox
      if (currentView.value === id) {
        currentView.value = 'inbox';
        selectedItemId.value = null;
      }
    } catch (e) {
      console.error('Failed to delete view:', e);
    }
  }
};

onMounted(() => {
  console.log('UmbraRelay app mounted');
  fetchCustomViews();
});
</script>

