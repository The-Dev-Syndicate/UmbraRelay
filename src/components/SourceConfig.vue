<template>
  <div class="source-config">
    <h1>Sources</h1>

    <div v-if="loading" class="loading">Loading...</div>
    <div v-if="error" class="error-message">
      {{ error }}
      <button @click="error = null" class="dismiss-error">×</button>
    </div>

    <div class="sources-list">
      <div v-for="source in sources" :key="source.id" class="source-card">
        <div class="source-header">
          <div>
            <h3>{{ source.name }}</h3>
            <div style="display: flex; gap: 8px; align-items: center; margin-top: 4px;">
              <span class="source-type">{{ source.source_type }}</span>
              <span v-if="source.group" class="source-group">{{ source.group }}</span>
            </div>
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
            <button 
              @click="handleSyncSource(source.id)" 
              class="sync-button"
              :disabled="syncingSources.has(source.id)"
            >
              {{ syncingSources.has(source.id) ? 'Syncing...' : 'Sync' }}
            </button>
            <button @click="editSource(source)" class="edit-button">Edit</button>
            <button 
              @click.stop="removeSource(source.id)" 
              class="delete-button"
              :disabled="deletingSources.has(source.id)"
              type="button"
            >
              {{ deletingSources.has(source.id) ? 'Deleting...' : 'Delete' }}
            </button>
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
        <div class="form-group">
          <label>Groups (optional) - Type groups separated by commas</label>
          <div class="group-input-container">
            <div class="group-pills">
              <span 
                v-for="(group, index) in parseGroups(rssForm.group)" 
                :key="index" 
                class="group-pill"
              >
                {{ group }}
                <button 
                  type="button"
                  @click="removeGroup('rss', index)"
                  class="pill-remove"
                >×</button>
              </span>
            </div>
            <input 
              v-model="rssForm.groupInput" 
              type="text" 
              placeholder="Type group name, press comma..." 
              class="group-input"
              @keydown="handleGroupKeydown($event, 'rss')"
              @blur="handleGroupBlur('rss')"
              list="groups-list-rss"
              autocomplete="off"
            />
            <datalist id="groups-list-rss">
              <option v-for="group in availableGroups" :key="group" :value="group" />
            </datalist>
          </div>
          <small style="color: #666; font-size: 12px; margin-top: 4px; display: block;">
            Type a group name and press comma to add it as a tag
          </small>
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
        <div class="form-group">
          <label>Groups (optional) - Type groups separated by commas</label>
          <div class="group-input-container">
            <div class="group-pills">
              <span 
                v-for="(group, index) in parseGroups(githubForm.group)" 
                :key="index" 
                class="group-pill"
              >
                {{ group }}
                <button 
                  type="button"
                  @click="removeGroup('github', index)"
                  class="pill-remove"
                >×</button>
              </span>
            </div>
            <input 
              v-model="githubForm.groupInput" 
              type="text" 
              placeholder="Type group name, press comma..." 
              class="group-input"
              @keydown="handleGroupKeydown($event, 'github')"
              @blur="handleGroupBlur('github')"
              list="groups-list-github"
              autocomplete="off"
            />
            <datalist id="groups-list-github">
              <option v-for="group in availableGroups" :key="group" :value="group" />
            </datalist>
          </div>
          <small style="color: #666; font-size: 12px; margin-top: 4px; display: block;">
            Type a group name and press comma to add it as a tag
          </small>
        </div>
        <button type="submit" class="submit-button">Add GitHub Source</button>
      </form>
    </div>

    <!-- Slide-in Edit Panel -->
    <div v-if="editingSource" class="edit-panel-overlay" @click="closeEditPanel">
      <div class="edit-panel" @click.stop>
        <div class="edit-panel-header">
          <h2>Editing: {{ editingSource.name }}</h2>
          <button @click="closeEditPanel" class="close-button" title="Close">×</button>
        </div>
        
        <div class="edit-panel-content">
          <!-- RSS Edit Form -->
          <form v-if="editingSource && editingSource.source_type === 'rss'" @submit.prevent="saveEdit" class="source-form">
            <div class="form-group">
              <label>Name</label>
              <input v-model="editForm.name" type="text" required placeholder="e.g., Hacker News" />
            </div>
            <div class="form-group">
              <label>URL</label>
              <input v-model="editForm.url" type="url" required placeholder="https://example.com/feed.xml" />
            </div>
            <div class="form-group">
              <label>Poll Interval (optional)</label>
              <input v-model="editForm.pollInterval" type="text" placeholder="10m" />
            </div>
            <div class="form-group">
              <label>Groups (optional) - Type groups separated by commas</label>
              <div class="group-input-container">
                <div class="group-pills">
                  <span 
                    v-for="(group, index) in parseGroups(editForm.group)" 
                    :key="index" 
                    class="group-pill"
                  >
                    {{ group }}
                    <button 
                      type="button"
                      @click="removeGroup('edit', index)"
                      class="pill-remove"
                    >×</button>
                  </span>
                </div>
                <input 
                  v-model="editForm.groupInput" 
                  type="text" 
                  placeholder="Type group name, press comma..." 
                  class="group-input"
                  @keydown="handleGroupKeydown($event, 'edit')"
                  @blur="handleGroupBlur('edit')"
                  list="edit-groups-list-rss"
                  autocomplete="off"
                />
                <datalist id="edit-groups-list-rss">
                  <option v-for="group in availableGroups" :key="group" :value="group" />
                </datalist>
              </div>
            </div>
            <div class="form-group">
              <label>
                <input v-model="editForm.enabled" type="checkbox" />
                Enabled
              </label>
            </div>
            <div class="form-actions">
              <button type="button" @click="closeEditPanel" class="cancel-button">Cancel</button>
              <button type="submit" class="submit-button">Save Changes</button>
            </div>
          </form>

          <!-- GitHub Edit Form -->
          <form v-if="editingSource && editingSource.source_type === 'github'" @submit.prevent="saveEdit" class="source-form">
            <div class="form-group">
              <label>Name</label>
              <input v-model="editForm.name" type="text" required placeholder="e.g., My Project" />
            </div>
            <div class="form-group">
              <label>Owner</label>
              <input v-model="editForm.owner" type="text" required placeholder="username or org" />
            </div>
            <div class="form-group">
              <label>Repository</label>
              <input v-model="editForm.repo" type="text" required placeholder="repo-name" />
            </div>
            <div class="form-group">
              <label>GitHub Token (leave blank to keep existing)</label>
              <input v-model="editForm.token" type="password" placeholder="ghp_..." />
            </div>
            <div class="form-group">
              <label>
                <input v-model="editForm.assignedOnly" type="checkbox" />
                Only show assigned issues/PRs
              </label>
            </div>
            <div class="form-group">
              <label>Groups (optional) - Type groups separated by commas</label>
              <div class="group-input-container">
                <div class="group-pills">
                  <span 
                    v-for="(group, index) in parseGroups(editForm.group)" 
                    :key="index" 
                    class="group-pill"
                  >
                    {{ group }}
                    <button 
                      type="button"
                      @click="removeGroup('edit', index)"
                      class="pill-remove"
                    >×</button>
                  </span>
                </div>
                <input 
                  v-model="editForm.groupInput" 
                  type="text" 
                  placeholder="Type group name, press comma..." 
                  class="group-input"
                  @keydown="handleGroupKeydown($event, 'edit')"
                  @blur="handleGroupBlur('edit')"
                  list="edit-groups-list-github"
                  autocomplete="off"
                />
                <datalist id="edit-groups-list-github">
                  <option v-for="group in availableGroups" :key="group" :value="group" />
                </datalist>
              </div>
            </div>
            <div class="form-group">
              <label>
                <input v-model="editForm.enabled" type="checkbox" />
                Enabled
              </label>
            </div>
            <div class="form-actions">
              <button type="button" @click="closeEditPanel" class="cancel-button">Cancel</button>
              <button type="submit" class="submit-button">Save Changes</button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useSources } from '../composables/useSources';
import { ask, MessageDialogOptions } from '@tauri-apps/plugin-dialog';
import type { Source, SourceInput, UpdateSourceInput } from '../types';

const { sources, loading, error, fetchSources, addSource, updateSource, removeSource: removeSourceAction, syncSource } = useSources();

const syncingSources = ref<Set<number>>(new Set());
const deletingSources = ref<Set<number>>(new Set());
const editingSource = ref<Source | null>(null);

const newSourceType = ref<'rss' | 'github'>('rss');

// Get available groups from existing sources (parse comma-separated)
const availableGroups = computed(() => {
  const groups = new Set<string>();
  sources.value.forEach(source => {
    if (source.group) {
      const parsed = source.group.split(',').map(g => g.trim()).filter(g => g.length > 0);
      parsed.forEach(g => groups.add(g));
    }
  });
  return Array.from(groups).sort();
});

const rssForm = ref({
  name: '',
  url: '',
  pollInterval: '10m',
  group: '',
  groupInput: '',
});

const githubForm = ref({
  name: '',
  owner: '',
  repo: '',
  token: '',
  assignedOnly: false,
  group: '',
  groupInput: '',
});

// Edit form - will be populated when editing
const editForm = ref({
  name: '',
  url: '',
  pollInterval: '',
  owner: '',
  repo: '',
  token: '',
  assignedOnly: false,
  group: '',
  groupInput: '',
  enabled: true,
});

const addRssSource = async () => {
  const source: SourceInput = {
    source_type: 'rss',
    name: rssForm.value.name,
    config_json: {
      url: rssForm.value.url,
      poll_interval: rssForm.value.pollInterval || '10m',
    },
    group: rssForm.value.group || null,
  };

  await addSource(source);
  
  // Reset form
  rssForm.value = { name: '', url: '', pollInterval: '10m', group: '', groupInput: '' };
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
    group: githubForm.value.group || null,
  };

  await addSource(source);
  
  // Reset form
  githubForm.value = { name: '', owner: '', repo: '', token: '', assignedOnly: false, group: '', groupInput: '' };
};

const toggleSource = async (id: number, enabled: boolean) => {
  await updateSource(id, { enabled });
};

const editSource = (source: Source) => {
  console.log('Editing source:', source);
  editingSource.value = source;
  
  // Parse config_json if it's a string
  let config: Record<string, any> = {};
  if (typeof source.config_json === 'string') {
    try {
      config = JSON.parse(source.config_json);
    } catch (e) {
      console.error('Failed to parse config_json:', e);
      config = {};
    }
  } else {
    config = source.config_json || {};
  }
  
  // Populate edit form based on source type
  editForm.value.name = source.name;
  editForm.value.enabled = source.enabled;
  editForm.value.group = source.group || '';
  editForm.value.groupInput = '';
  
  if (source.source_type === 'rss') {
    editForm.value.url = config.url || '';
    editForm.value.pollInterval = config.poll_interval || '10m';
    console.log('RSS form populated:', editForm.value);
  } else if (source.source_type === 'github') {
    editForm.value.owner = config.owner || '';
    editForm.value.repo = config.repo || '';
    editForm.value.assignedOnly = config.assigned_only || false;
    // Token is stored separately, so we leave it blank (user can update if needed)
    editForm.value.token = '';
    console.log('GitHub form populated:', editForm.value);
  }
};

const closeEditPanel = () => {
  editingSource.value = null;
  // Reset form
  editForm.value = {
    name: '',
    url: '',
    pollInterval: '',
    owner: '',
    repo: '',
    token: '',
    assignedOnly: false,
    group: '',
    groupInput: '',
    enabled: true,
  };
};

// Parse comma-separated groups
const parseGroups = (groupString: string | null | undefined): string[] => {
  if (!groupString) return [];
  return groupString.split(',').map(g => g.trim()).filter(g => g.length > 0);
};

// Handle group input keydown
const handleGroupKeydown = (event: KeyboardEvent, formType: 'rss' | 'github' | 'edit') => {
  if (event.key === ',' || event.key === 'Enter') {
    event.preventDefault();
    addGroupFromInput(formType);
  } else if (event.key === 'Backspace') {
    const form = formType === 'rss' ? rssForm.value : formType === 'github' ? githubForm.value : editForm.value;
    if (form.groupInput === '' && form.group) {
      // Remove last group if input is empty
      const groups = parseGroups(form.group);
      if (groups.length > 0) {
        groups.pop();
        form.group = groups.join(', ');
        if (formType === 'rss') {
          rssForm.value = { ...rssForm.value, group: form.group };
        } else if (formType === 'github') {
          githubForm.value = { ...githubForm.value, group: form.group };
        } else {
          editForm.value = { ...editForm.value, group: form.group };
        }
      }
    }
  }
};

// Handle group input blur
const handleGroupBlur = (formType: 'rss' | 'github' | 'edit') => {
  const form = formType === 'rss' ? rssForm.value : formType === 'github' ? githubForm.value : editForm.value;
  if (form.groupInput.trim()) {
    addGroupFromInput(formType);
  }
};

// Add group from input
const addGroupFromInput = async (formType: 'rss' | 'github' | 'edit') => {
  const form = formType === 'rss' ? rssForm.value : formType === 'github' ? githubForm.value : editForm.value;
  const input = form.groupInput.trim();
  if (input) {
    const groups = parseGroups(form.group);
    if (!groups.includes(input)) {
      groups.push(input);
      const newGroupString = groups.join(', ');
      form.group = newGroupString;
      form.groupInput = '';
      
      if (formType === 'rss') {
        rssForm.value = { ...rssForm.value, group: newGroupString, groupInput: '' };
      } else if (formType === 'github') {
        githubForm.value = { ...githubForm.value, group: newGroupString, groupInput: '' };
      } else {
        // In edit mode, immediately save to database
        editForm.value = { ...editForm.value, group: newGroupString, groupInput: '' };
        if (editingSource.value) {
          try {
            await updateSource(editingSource.value.id, { 
              group: newGroupString || null 
            });
            // Refresh sources to update available groups
            await fetchSources();
          } catch (e) {
            console.error('Failed to update source groups:', e);
            // Revert on error
            const groups = parseGroups(editingSource.value.group || '');
            groups.pop(); // Remove the group we just added
            editForm.value.group = groups.join(', ');
          }
        }
      }
    } else {
      form.groupInput = '';
    }
  }
};

// Remove group by index
const removeGroup = async (formType: 'rss' | 'github' | 'edit', index: number) => {
  const form = formType === 'rss' ? rssForm.value : formType === 'github' ? githubForm.value : editForm.value;
  const groups = parseGroups(form.group);
  if (index < 0 || index >= groups.length) return; // Safety check
  groups.splice(index, 1);
  const newGroupString = groups.length > 0 ? groups.join(', ') : '';
  form.group = newGroupString;
  
  if (formType === 'rss') {
    rssForm.value = { ...rssForm.value, group: newGroupString };
  } else if (formType === 'github') {
    githubForm.value = { ...githubForm.value, group: newGroupString };
  } else {
    // In edit mode, immediately save to database
    editForm.value = { ...editForm.value, group: newGroupString };
    if (editingSource.value) {
      try {
        // Pass empty string to clear groups (backend will treat empty string as NULL)
        // We need to pass an empty string, not null, because JSON null deserializes to None (don't update)
        // but we want to update to NULL, so we pass "" and let backend handle it
        const groupValue = newGroupString.trim() === '' ? '' : newGroupString;
        await updateSource(editingSource.value.id, { 
          group: groupValue
        });
        // Refresh sources to update available groups and update editingSource
        await fetchSources();
        // Update editingSource to reflect the change
        const updatedSource = sources.value.find(s => s.id === editingSource.value!.id);
        if (updatedSource) {
          editingSource.value = updatedSource;
          editForm.value.group = updatedSource.group || '';
        }
      } catch (e) {
        console.error('Failed to update source groups:', e);
        // Revert on error
        editForm.value.group = editingSource.value.group || '';
      }
    }
  }
};

const saveEdit = async () => {
  if (!editingSource.value) return;
  
  try {
    const update: UpdateSourceInput = {
      name: editForm.value.name,
      enabled: editForm.value.enabled,
      group: editForm.value.group || null,
    };
    
    if (editingSource.value.source_type === 'rss') {
      update.config_json = {
        url: editForm.value.url,
        poll_interval: editForm.value.pollInterval || '10m',
      };
    } else if (editingSource.value.source_type === 'github') {
      update.config_json = {
        owner: editForm.value.owner,
        repo: editForm.value.repo,
        assigned_only: editForm.value.assignedOnly,
      };
      // Only update token if a new one was provided
      if (editForm.value.token) {
        update.token = editForm.value.token;
      }
    }
    
    await updateSource(editingSource.value.id, update);
    closeEditPanel();
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    console.error('Failed to update source:', e);
    error.value = `Failed to update source: ${errorMsg}`;
  }
};

const removeSource = async (id: number) => {
  console.log('=== DELETE SOURCE START ===');
  console.log('removeSource called with id:', id);
  
  // Find source name for feedback
  const sourceName = sources.value.find(s => s.id === id)?.name || 'source';
  
  // Show confirmation dialog using Tauri's dialog plugin
  const confirmed = await ask(
    `Are you sure you want to remove the source: ${sourceName}? This will also delete all items from this source.`,
    {
      title: 'Delete Source',
      kind: 'warning',
      okLabel: 'Confirm',
      cancelLabel: 'Cancel',
    } as MessageDialogOptions
  );
  
  console.log('User confirmed:', confirmed);
  
  if (!confirmed) {
    console.log('User cancelled deletion');
    return;
  }
  
  console.log('Starting deletion process...');
  deletingSources.value.add(id);
  
  try {
    console.log('Calling removeSourceAction with id:', id);
    await removeSourceAction(id);
    console.log('removeSourceAction completed successfully');
    
    // Force refresh to ensure UI updates
    console.log('Refreshing sources list...');
    await fetchSources();
    console.log('Sources list refreshed. Current sources:', sources.value.map(s => ({ id: s.id, name: s.name })));
    
    console.log(`Source "${sourceName}" deleted successfully`);
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    console.error('=== DELETE SOURCE ERROR ===', e);
    error.value = `Failed to remove source: ${errorMsg}`;
  } finally {
    deletingSources.value.delete(id);
    console.log('=== DELETE SOURCE END ===');
  }
};

const handleSyncSource = async (id: number) => {
  syncingSources.value.add(id);
  try {
    await syncSource(id);
    // Show success feedback
    const source = sources.value.find(s => s.id === id);
    if (source) {
      // Force refresh to show updated sync time
      await fetchSources();
    }
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    console.error('Failed to sync source:', e);
    alert(`Failed to sync source: ${errorMsg}`);
  } finally {
    syncingSources.value.delete(id);
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

.loading {
  text-align: center;
  padding: 20px;
}

.error-message {
  background: #ffebee;
  border: 1px solid #d32f2f;
  color: #d32f2f;
  padding: 12px 16px;
  border-radius: 4px;
  margin-bottom: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.dismiss-error {
  background: none;
  border: none;
  color: #d32f2f;
  font-size: 20px;
  cursor: pointer;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.dismiss-error:hover {
  background: rgba(211, 47, 47, 0.1);
  border-radius: 50%;
}

.error-message {
  background: #ffebee;
  border: 1px solid #d32f2f;
  color: #d32f2f;
  padding: 12px 16px;
  border-radius: 4px;
  margin-bottom: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.dismiss-error {
  background: none;
  border: none;
  color: #d32f2f;
  font-size: 20px;
  cursor: pointer;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dismiss-error:hover {
  background: rgba(211, 47, 47, 0.1);
  border-radius: 50%;
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

.source-group {
  font-size: 12px;
  padding: 2px 8px;
  background: #e3f2fd;
  color: #1976d2;
  border-radius: 12px;
  font-weight: 500;
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

.sync-button:disabled, .edit-button:disabled, .delete-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
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

.group-input-container {
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 8px;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  min-height: 40px;
  background: white;
}

.group-input-container:focus-within {
  border-color: #396cd8;
  outline: none;
}

.group-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.group-pill {
  background: #e3f2fd;
  color: #1976d2;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.pill-remove {
  background: none;
  border: none;
  color: #1976d2;
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  padding: 0;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: background 0.2s;
}

.pill-remove:hover {
  background: rgba(25, 118, 210, 0.2);
}

.group-input {
  flex: 1;
  min-width: 120px;
  border: none;
  outline: none;
  padding: 4px 8px;
  font-size: 14px;
  background: transparent;
}

.submit-button {
  padding: 10px 20px;
  background: #396cd8;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.2s;
}

.submit-button:hover {
  background: #2952b8;
}

/* Edit Panel Styles */
.edit-panel-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
  display: flex;
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.edit-panel {
  position: fixed;
  top: 0;
  right: 0;
  width: 500px;
  max-width: 90vw;
  height: 100vh;
  background: white;
  box-shadow: -2px 0 10px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  z-index: 1001;
  animation: slideInRight 0.3s ease-out;
  overflow-y: auto;
}

@keyframes slideInRight {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}

.edit-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #e0e0e0;
  background: #f8f9fa;
  position: relative;
}

.edit-panel-header h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  flex: 1;
}

.close-button {
  background: none;
  border: none;
  font-size: 32px;
  line-height: 1;
  cursor: pointer;
  color: #666;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
  position: absolute;
  top: 20px;
  right: 20px;
}

.close-button:hover {
  background: #e0e0e0;
  color: #333;
}

.edit-panel-content {
  padding: 20px;
  flex: 1;
  overflow-y: auto;
}

.edit-panel-content .source-form {
  max-width: 100%;
  width: 100%;
}

.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid #e0e0e0;
}

.cancel-button {
  flex: 1;
  padding: 12px 24px;
  border: 1px solid #ddd;
  background: white;
  color: #333;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.2s;
}

.cancel-button:hover {
  background: #f5f5f5;
  border-color: #bbb;
}

.form-actions .submit-button {
  flex: 1;
  padding: 12px 24px;
  font-weight: 500;
}
</style>

