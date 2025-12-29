<template>
  <div class="app">
    <nav class="sidebar">
      <h1 class="app-title">UmbraRelay</h1>
      <div class="nav-links">
        <button
          :class="{ active: currentView === 'inbox' }"
          @click="currentView = 'inbox'"
          class="nav-button"
        >
          Inbox
        </button>
        <button
          :class="{ active: currentView === 'sources' }"
          @click="currentView = 'sources'"
          class="nav-button"
        >
          Sources
        </button>
      </div>
    </nav>

    <main class="main-content">
      <InboxView
        v-if="currentView === 'inbox' && !selectedItemId"
        @select-item="selectItem"
        key="inbox"
      />
      <ItemDetail
        v-else-if="currentView === 'inbox' && selectedItemId"
        :item-id="selectedItemId"
        @back="selectedItemId = null"
        key="item-detail"
      />
      <SourceConfig v-else-if="currentView === 'sources'" key="sources" />
      <div v-else class="fallback">
        <p>Select a view from the sidebar</p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import InboxView from './components/InboxView.vue';
import ItemDetail from './components/ItemDetail.vue';
import SourceConfig from './components/SourceConfig.vue';

const currentView = ref<'inbox' | 'sources'>('inbox');
const selectedItemId = ref<number | null>(null);

const selectItem = (id: number) => {
  selectedItemId.value = id;
};

onMounted(() => {
  console.log('UmbraRelay app mounted');
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
  gap: 8px;
}

.nav-button {
  padding: 12px 16px;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.7);
  text-align: left;
  cursor: pointer;
  border-radius: 4px;
  font-size: 16px;
  transition: all 0.2s;
}

.nav-button:hover {
  background: rgba(255, 255, 255, 0.1);
  color: white;
}

.nav-button.active {
  background: #396cd8;
  color: white;
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
