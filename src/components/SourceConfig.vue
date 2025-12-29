<template>
  <div class="source-config">
    <h1>Sources</h1>

    <div v-if="loading" class="loading">Loading...</div>
    <div v-else-if="error" class="error">{{ error }}</div>

    <div class="sources-list">
      <div v-for="source in sources" :key="source.id" class="source-card">
        <div class="source-header">
          <div>
            <h3>{{ source.name }}</h3>
            <span class="source-type">{{ source.type }}</span>
          </div>
          <div class="source-actions">
            <label class="toggle">
              <input
                type="checkbox"
                :checked="source.enabled"
                @change="toggleSource(source.id, ($event.target as HTMLInputElement).checked)"
              />
              <span>{{ source.enabled ? 'Enabled' : 'Disabled' }}</span>
            </label>
            <button @click="syncSource(source.id)" class="sync-button">Sync</button>
            <button @click="editSource(source)" class="edit-button">Edit</button>
            <button @click="removeSource(source.id)" class="delete-button">Delete</button>
          </div>
        </div>
        <div class="source-info">
          <p v-if="source.last_synced_at">
            Last synced: {{ formatDate(source.last_synced_at) }}
          </p>
          <p v-else>Never synced</p>
        </div>
      </div>
    </div>

    <div class="add-source-section">
      <h2>Add Source</h2>
      <div class="tabs">
        <button
          :class="{ active: newSourceType === 'rss' }"
          @click="newSourceType = 'rss'"
        >
          RSS Feed
        </button>
        <button
          :class="{ active: newSourceType === 'github' }"
          @click="newSourceType = 'github'"
        >
          GitHub
        </button>
      </div>

      <form v-if="newSourceType === 'rss'" @submit.prevent="addRssSource" class="source-form">
        <div class="form-group">
          <label>Name</label>
          <input v-model="rssForm.name" type="text" required placeholder="e.g., Hacker News" />
        </div>
        <div class="form-group">
          <label>URL</label>
          <input v-model="rssForm.url" type="url" required placeholder="https://example.com/feed.xml" />
        </div>
        <div class="form-group">
          <label>Poll Interval (optional)</label>
          <input v-model="rssForm.pollInterval" type="text" placeholder="10m" />
        </div>
        <button type="submit" class="submit-button">Add RSS Feed</button>
      </form>

      <form v-if="newSourceType === 'github'" @submit.prevent="addGitHubSource" class="source-form">
        <div class="form-group">
          <label>Name</label>
          <input v-model="githubForm.name" type="text" required placeholder="e.g., My Project" />
        </div>
        <div class="form-group">
          <label>Owner</label>
          <input v-model="githubForm.owner" type="text" required placeholder="username or org" />
        </div>
        <div class="form-group">
          <label>Repository</label>
          <input v-model="githubForm.repo" type="text" required placeholder="repo-name" />
        </div>
        <div class="form-group">
          <label>GitHub Token</label>
          <input v-model="githubForm.token" type="password" required placeholder="ghp_..." />
        </div>
        <div class="form-group">
          <label>
            <input v-model="githubForm.assignedOnly" type="checkbox" />
            Only show assigned issues/PRs
          </label>
        </div>
        <button type="submit" class="submit-button">Add GitHub Source</button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useSources } from '../composables/useSources';
import type { Source, SourceInput } from '../types';

const { sources, loading, error, fetchSources, addSource, updateSource, removeSource: removeSourceAction, syncSource } = useSources();

const newSourceType = ref<'rss' | 'github'>('rss');

const rssForm = ref({
  name: '',
  url: '',
  pollInterval: '10m',
});

const githubForm = ref({
  name: '',
  owner: '',
  repo: '',
  token: '',
  assignedOnly: false,
});

const addRssSource = async () => {
  const source: SourceInput = {
    source_type: 'rss',
    name: rssForm.value.name,
    config_json: {
      url: rssForm.value.url,
      poll_interval: rssForm.value.pollInterval || '10m',
    },
  };

  await addSource(source);
  
  // Reset form
  rssForm.value = { name: '', url: '', pollInterval: '10m' };
};

const addGitHubSource = async () => {
  const source: SourceInput = {
    source_type: 'github',
    name: githubForm.value.name,
    config_json: {
      owner: githubForm.value.owner,
      repo: githubForm.value.repo,
      assigned_only: githubForm.value.assignedOnly,
    },
    token: githubForm.value.token,
  };

  await addSource(source);
  
  // Reset form
  githubForm.value = { name: '', owner: '', repo: '', token: '', assignedOnly: false };
};

const toggleSource = async (id: number, enabled: boolean) => {
  await updateSource(id, { enabled });
};

const editSource = (source: Source) => {
  // TODO: Implement edit modal/form
  console.log('Edit source:', source);
};

const removeSource = async (id: number) => {
  if (confirm('Are you sure you want to remove this source?')) {
    await removeSourceAction(id);
  }
};

const formatDate = (timestamp: number) => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
};

onMounted(() => {
  fetchSources();
});
</script>

<style scoped>
.source-config {
  padding: 20px;
  max-width: 1000px;
  margin: 0 auto;
}

.loading, .error {
  text-align: center;
  padding: 20px;
}

.error {
  color: #d32f2f;
}

.sources-list {
  margin-bottom: 40px;
}

.source-card {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 12px;
  background: white;
}

.source-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.source-header h3 {
  margin: 0 0 4px 0;
}

.source-type {
  font-size: 12px;
  text-transform: uppercase;
  color: #666;
}

.source-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.sync-button, .edit-button, .delete-button {
  padding: 6px 12px;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
  border-radius: 4px;
  font-size: 14px;
}

.delete-button {
  color: #d32f2f;
  border-color: #d32f2f;
}

.source-info {
  font-size: 14px;
  color: #666;
}

.add-source-section {
  border-top: 2px solid #ddd;
  padding-top: 20px;
}

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
}

.tabs button {
  padding: 10px 20px;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
  border-radius: 4px;
}

.tabs button.active {
  background: #396cd8;
  color: white;
  border-color: #396cd8;
}

.source-form {
  max-width: 500px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-weight: 500;
}

.form-group input[type="text"],
.form-group input[type="url"],
.form-group input[type="password"] {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.form-group input[type="checkbox"] {
  margin-right: 8px;
}

.submit-button {
  padding: 10px 20px;
  background: #396cd8;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
}

.submit-button:hover {
  background: #2952b8;
}
</style>

