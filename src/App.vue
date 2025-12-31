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

<style>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html, body {
  width: 100%;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  background: #f5f5f5;
  overflow: hidden;
}

#app {
  width: 100%;
  height: 100%;
}

.app {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.sidebar {
  width: 200px;
  min-width: 200px;
  background: #2c3e50;
  color: white;
  padding: 20px;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.app-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 30px;
  color: white;
}

.nav-links {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.nav-group {
  margin-bottom: 8px;
}

.nav-group-header {
  width: 100%;
  padding: 10px 12px;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.9);
  text-align: left;
  cursor: pointer;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-group-header:hover {
  background: rgba(255, 255, 255, 0.1);
  color: white;
}


.nav-icon {
  font-size: 14px;
  width: 18px;
  text-align: center;
  flex-shrink: 0;
}

.nav-arrow {
  margin-left: auto;
  font-size: 9px;
  transition: transform 0.2s;
  flex-shrink: 0;
}

.nav-arrow.expanded {
  transform: rotate(180deg);
}

.nav-group-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-left: 12px;
  margin-top: 4px;
}

.nav-button {
  padding: 8px 12px;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.7);
  text-align: left;
  cursor: pointer;
  border-radius: 4px;
  font-size: 12px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-sub-item {
  padding-left: 24px;
  font-size: 11px;
}

.nav-button:hover {
  background: rgba(255, 255, 255, 0.1);
  color: white;
}

.nav-button.active {
  background: #396cd8;
  color: white;
}

.nav-empty-state {
  padding: 8px 12px;
  padding-left: 24px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 11px;
  font-style: italic;
}

.nav-view-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.nav-view-item .nav-button {
  flex: 1;
}

.nav-delete-button {
  padding: 4px 8px;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  border-radius: 4px;
  font-size: 18px;
  line-height: 1;
  transition: all 0.2s;
  opacity: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-view-item:hover .nav-delete-button {
  opacity: 1;
}

.nav-delete-button:hover {
  background: rgba(255, 0, 0, 0.2);
  color: #ff6b6b;
}

.nav-spacer {
  flex: 1;
}

.nav-bottom-actions {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: auto;
}

.nav-bottom {
  width: 100%;
}

.nav-add-view {
  margin-bottom: 4px;
}

.main-content {
  flex: 1;
  overflow-y: auto;
  background: #f5f5f5;
  min-height: 0;
  position: relative;
}

.fallback {
  padding: 40px;
  text-align: center;
  color: #666;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
