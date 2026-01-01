<template>
  <div class="source-config">
    <!-- Sticky Header -->
    <div class="sticky-header">
      <div class="header-left">
        <h1>Sources</h1>
      </div>
      <div class="header-actions">
        <button @click="openAddGroupPanel" class="action-button">
          + Add Group
        </button>
        <button @click="showAddSourceModal = true" class="action-button primary">
          + Add Source
        </button>
        <button 
          @click="handleSyncAll" 
          class="action-button"
          :disabled="syncingAll"
        >
          {{ syncingAll ? 'Syncing...' : 'Sync All' }}
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading">Loading sources and groups...</div>
    <div v-if="error" class="error-message">
      Error: {{ error }}
      <button @click="error = null" class="dismiss-error">×</button>
    </div>

    <div class="sources-list">
      <div v-for="source in sources" :key="source.id" class="source-card">
        <div class="source-header">
          <div>
            <h3>{{ source.name }}</h3>
            <div style="display: flex; gap: 8px; align-items: center; margin-top: 4px; flex-wrap: wrap;">
              <span class="source-type">{{ source.source_type }}</span>
              <span 
                v-for="groupName in getSourceGroupNames(source)" 
                :key="groupName" 
                class="source-group"
              >
                {{ groupName }}
              </span>
            </div>
          </div>
          <div class="source-actions">
            <button 
              @click="handleSyncSource(source.id)" 
              class="icon-button"
              :disabled="syncingSources.has(source.id)"
              :title="syncingSources.has(source.id) ? 'Syncing...' : 'Sync'"
            >
              ↻
            </button>
            <button 
              @click="editSource(source)" 
              class="icon-button"
              title="Edit"
            >
              ✏️
            </button>
            <button 
              @click.stop="removeSource(source.id)" 
              class="icon-button delete"
              :disabled="deletingSources.has(source.id)"
              :title="deletingSources.has(source.id) ? 'Deleting...' : 'Delete'"
              type="button"
            >
              🗑️
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

    <!-- Group Management Section -->
    <div class="group-management-section">
      <div class="section-header">
        <h2>Group Management</h2>
      </div>
      
      <div v-if="groups.length === 0" class="empty-state">
        <p>No groups yet. Create your first group to organize your sources.</p>
      </div>
      
      <div v-else class="groups-list">
        <div v-for="group in groups" :key="group.id" class="group-card">
          <div class="group-info">
            <span class="group-name">{{ group.name }}</span>
            <span class="group-count">
              {{ getGroupSourceCount(group.id) }} source{{ getGroupSourceCount(group.id) !== 1 ? 's' : '' }}
            </span>
          </div>
          <div class="group-actions">
            <button 
              @click="editGroup(group)" 
              class="icon-button"
              title="Edit"
            >
              ✏️
            </button>
            <button 
              @click="removeGroup(group.id)" 
              class="icon-button delete"
              title="Delete"
            >
              🗑️
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Source Modal -->
    <div v-if="showAddSourceModal" class="edit-panel-overlay" @click="closeAddSourceModal">
      <div class="edit-panel" @click.stop>
        <div class="edit-panel-header">
          <h2>Add Source</h2>
          <button @click="closeAddSourceModal" class="close-button" title="Close">×</button>
        </div>
        
        <div class="edit-panel-content">
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

          <!-- RSS Form -->
          <form v-if="newSourceType === 'rss'" @submit.prevent="addRssSource" class="source-form" novalidate>
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
              <label>Groups (optional)</label>
              <div class="checkbox-group">
                <label 
                  v-for="group in groups" 
                  :key="group.id"
                  class="checkbox-option"
                >
                  <input 
                    type="checkbox" 
                    :value="group.id"
                    v-model="rssForm.groupIds"
                  />
                  <span>{{ group.name }}</span>
                </label>
                <p v-if="groups.length === 0" class="no-groups-hint">
                  No groups available. Create a group in the Group Management section.
                </p>
              </div>
            </div>
            <div class="form-actions">
              <button type="button" @click="closeAddSourceModal" class="cancel-button">Cancel</button>
              <button type="submit" class="submit-button">Add RSS Feed</button>
            </div>
          </form>

          <!-- GitHub Form -->
          <form v-if="newSourceType === 'github'" @submit.prevent="addGitHubSource" class="source-form" novalidate>
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
              <label>Groups (optional)</label>
              <div class="checkbox-group">
                <label 
                  v-for="group in groups" 
                  :key="group.id"
                  class="checkbox-option"
                >
                  <input 
                    type="checkbox" 
                    :value="group.id"
                    v-model="githubForm.groupIds"
                  />
                  <span>{{ group.name }}</span>
                </label>
                <p v-if="groups.length === 0" class="no-groups-hint">
                  No groups available. Create a group in the Group Management section.
                </p>
              </div>
            </div>
            <div class="form-actions">
              <button type="button" @click="closeAddSourceModal" class="cancel-button">Cancel</button>
              <button type="submit" class="submit-button">Add GitHub Source</button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- Edit Source Panel -->
    <div v-if="editingSource" class="edit-panel-overlay" @click="closeEditPanel">
      <div class="edit-panel" @click.stop>
        <div class="edit-panel-header">
          <h2>Editing: {{ editingSource.name }}</h2>
          <button @click="closeEditPanel" class="close-button" title="Close">×</button>
        </div>
        
        <div class="edit-panel-content">
          <!-- RSS Edit Form -->
          <form v-if="editingSource && editingSource.source_type === 'rss'" @submit.prevent="saveEdit" class="source-form" novalidate>
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
              <label>Groups (optional)</label>
              <div class="checkbox-group">
                <label 
                  v-for="group in groups" 
                  :key="group.id"
                  class="checkbox-option"
                >
                  <input 
                    type="checkbox" 
                    :value="group.id"
                    v-model="editForm.groupIds"
                  />
                  <span>{{ group.name }}</span>
                </label>
                <p v-if="groups.length === 0" class="no-groups-hint">
                  No groups available. Create a group in the Group Management section.
                </p>
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
              <button type="submit" @click.prevent="saveEdit" class="submit-button">Save Changes</button>
            </div>
          </form>

          <!-- GitHub Edit Form -->
          <form v-if="editingSource && editingSource.source_type === 'github'" @submit.prevent="saveEdit" class="source-form" novalidate>
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
              <label>Groups (optional)</label>
              <div class="checkbox-group">
                <label 
                  v-for="group in groups" 
                  :key="group.id"
                  class="checkbox-option"
                >
                  <input 
                    type="checkbox" 
                    :value="group.id"
                    v-model="editForm.groupIds"
                  />
                  <span>{{ group.name }}</span>
                </label>
                <p v-if="groups.length === 0" class="no-groups-hint">
                  No groups available. Create a group in the Group Management section.
                </p>
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
              <button type="submit" @click.prevent="saveEdit" class="submit-button">Save Changes</button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- Add/Edit Group Panel -->
    <div v-if="editingGroup !== null" class="edit-panel-overlay" @click="closeGroupPanel">
      <div class="edit-panel" @click.stop>
        <div class="edit-panel-header">
          <h2>{{ editingGroup.id ? 'Edit Group' : 'Add Group' }}</h2>
          <button @click="closeGroupPanel" class="close-button" title="Close">×</button>
        </div>
        
        <div class="edit-panel-content">
          <form @submit.prevent="saveGroup" class="source-form" novalidate>
            <div class="form-group">
              <label>Group Name</label>
              <input 
                v-model="groupForm.name" 
                type="text" 
                required 
                placeholder="e.g., Work, Personal, etc." 
              />
            </div>
            <div v-if="editingGroup && editingGroup.id" class="form-group">
              <label>Sources in this Group</label>
              <div class="checkbox-group">
                <label 
                  v-for="source in sources" 
                  :key="source.id"
                  class="checkbox-option"
                >
                  <input 
                    type="checkbox" 
                    :value="source.id"
                    v-model="groupForm.sourceIds"
                  />
                  <span>{{ source.name }}</span>
                </label>
                <p v-if="sources.length === 0" class="no-groups-hint">
                  No sources available.
                </p>
              </div>
            </div>
            <div class="form-actions">
              <button type="button" @click="closeGroupPanel" class="cancel-button">Cancel</button>
              <button type="submit" class="submit-button">
                {{ editingGroup.id ? 'Save Changes' : 'Create Group' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useSources } from '../composables/useSources';
import { useGroups } from '../composables/useGroups';
import { ask, MessageDialogOptions } from '@tauri-apps/plugin-dialog';
import type { Source, SourceInput, UpdateSourceInput, Group } from '../types';

const { sources, loading, error, fetchSources, addSource, updateSource, removeSource: removeSourceAction, syncSource, syncAllSources } = useSources();
const { groups, fetchGroups, addGroup, updateGroup, removeGroup: removeGroupAction } = useGroups();

const syncingSources = ref<Set<number>>(new Set());
const deletingSources = ref<Set<number>>(new Set());
const syncingAll = ref(false);
const editingSource = ref<Source | null>(null);
const showAddSourceModal = ref(false);
const editingGroup = ref<Group | { id: null; name: string } | null>(null);
const savingGroup = ref(false);
const savingSource = ref(false);
const addingSource = ref(false);

const newSourceType = ref<'rss' | 'github'>('rss');

const rssForm = ref({
  name: '',
  url: '',
  pollInterval: '10m',
  groupIds: [] as number[],
});

const githubForm = ref({
  name: '',
  owner: '',
  repo: '',
  token: '',
  assignedOnly: false,
  groupIds: [] as number[],
});

const editForm = ref({
  name: '',
  url: '',
  pollInterval: '',
  owner: '',
  repo: '',
  token: '',
  assignedOnly: false,
  groupIds: [] as number[],
  enabled: true,
});

const groupForm = ref({
  name: '',
  sourceIds: [] as number[],
});

// Get group names for a source
const getSourceGroupNames = (source: Source): string[] => {
  if (!source.group_ids || source.group_ids.length === 0) {
    return [];
  }
  return source.group_ids
    .map(id => groups.value.find(g => g.id === id)?.name)
    .filter((name): name is string => name !== undefined);
};

// Get count of sources using a group
const getGroupSourceCount = (groupId: number): number => {
  return sources.value.filter(s => s.group_ids?.includes(groupId)).length;
};


const closeAddSourceModal = () => {
  showAddSourceModal.value = false;
};

const addRssSource = async (e?: Event) => {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  
  if (addingSource.value) return false;
  
  if (!rssForm.value.name.trim()) {
    alert('Please enter a source name');
    return false;
  }
  if (!rssForm.value.url.trim()) {
    alert('Please enter a URL');
    return false;
  }
  
  addingSource.value = true;
  try {
    const source: SourceInput = {
      source_type: 'rss',
      name: rssForm.value.name,
      config_json: {
        url: rssForm.value.url,
        poll_interval: rssForm.value.pollInterval || '10m',
      },
      group_ids: rssForm.value.groupIds.length > 0 ? rssForm.value.groupIds : null,
    };

    await addSource(source);
    await fetchGroups();
    await fetchSources();
    closeAddSourceModal();
    
    // Reset form
    rssForm.value = { name: '', url: '', pollInterval: '10m', groupIds: [] };
    return false;
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to add source: ${errorMsg}`);
    return false;
  } finally {
    addingSource.value = false;
  }
};

const addGitHubSource = async (e?: Event) => {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  
  if (addingSource.value) return false;
  
  if (!githubForm.value.name.trim()) {
    alert('Please enter a source name');
    return false;
  }
  if (!githubForm.value.owner.trim()) {
    alert('Please enter an owner');
    return false;
  }
  if (!githubForm.value.repo.trim()) {
    alert('Please enter a repository name');
    return false;
  }
  
  addingSource.value = true;
  try {
    const source: SourceInput = {
      source_type: 'github',
      name: githubForm.value.name,
      config_json: {
        owner: githubForm.value.owner,
        repo: githubForm.value.repo,
        assigned_only: githubForm.value.assignedOnly,
      },
      token: githubForm.value.token,
      group_ids: githubForm.value.groupIds.length > 0 ? githubForm.value.groupIds : null,
    };

    await addSource(source);
    await fetchGroups();
    await fetchSources();
    closeAddSourceModal();
    
    // Reset form
    githubForm.value = { name: '', owner: '', repo: '', token: '', assignedOnly: false, groupIds: [] };
    return false;
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to add source: ${errorMsg}`);
    return false;
  } finally {
    addingSource.value = false;
  }
};

const editSource = (source: Source) => {
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
  
  // Populate edit form
  editForm.value.name = source.name;
  editForm.value.enabled = source.enabled;
  editForm.value.groupIds = source.group_ids || [];
  
  if (source.source_type === 'rss') {
    editForm.value.url = config.url || '';
    editForm.value.pollInterval = config.poll_interval || '10m';
  } else if (source.source_type === 'github') {
    editForm.value.owner = config.owner || '';
    editForm.value.repo = config.repo || '';
    editForm.value.assignedOnly = config.assigned_only || false;
    editForm.value.token = '';
  }
};

const closeEditPanel = () => {
  editingSource.value = null;
  editForm.value = {
    name: '',
    url: '',
    pollInterval: '',
    owner: '',
    repo: '',
    token: '',
    assignedOnly: false,
    groupIds: [],
    enabled: true,
  };
};

const saveEdit = async (e?: Event) => {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  
  console.log('saveEdit called', { savingSource: savingSource.value, editingSource: editingSource.value });
  
  if (savingSource.value) {
    console.log('Already saving, returning');
    return false;
  }
  if (!editingSource.value) {
    console.log('No editing source, returning');
    return false;
  }
  
  if (!editForm.value.name.trim()) {
    alert('Please enter a source name');
    return false;
  }
  
  if (editingSource.value.source_type === 'rss' && !editForm.value.url.trim()) {
    alert('Please enter a URL');
    return false;
  }
  
  if (editingSource.value.source_type === 'github') {
    if (!editForm.value.owner.trim()) {
      alert('Please enter an owner');
      return false;
    }
    if (!editForm.value.repo.trim()) {
      alert('Please enter a repository name');
      return false;
    }
  }
  
  savingSource.value = true;
  try {
    console.log('Starting save, form data:', editForm.value);
    // Convert Proxy array to plain array to ensure proper serialization
    const groupIds = Array.isArray(editForm.value.groupIds) 
      ? [...editForm.value.groupIds] 
      : [];
    
    console.log('Group IDs (plain array):', groupIds, 'Type:', typeof groupIds, 'IsArray:', Array.isArray(groupIds));
    
    const update: UpdateSourceInput = {
      name: editForm.value.name,
      enabled: editForm.value.enabled,
      // Rust expects Option<Vec<i64>>: None = don't update, Some([]) = clear, Some([1,2]) = set
      // Always send the array (even if empty) to update groups
      group_ids: groupIds,
    };
    
    console.log('Update object before serialization:', JSON.stringify(update, null, 2));
    
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
      if (editForm.value.token) {
        update.token = editForm.value.token;
      }
    }
    
    console.log('Calling updateSource with:', { id: editingSource.value.id, update });
    // Update the source
    try {
      await updateSource(editingSource.value.id, update);
      console.log('updateSource completed successfully');
    } catch (updateError) {
      console.error('updateSource threw error:', updateError);
      throw updateError;
    }
    
    console.log('Refreshing data...');
    // Refresh data
    await fetchGroups();
    console.log('fetchGroups completed');
    await fetchSources();
    console.log('fetchSources completed');
    
    console.log('Closing panel...');
    // Close panel only after successful update
    closeEditPanel();
    console.log('Panel closed');
    return false;
  } catch (e) {
    console.error('Error in saveEdit catch block:', e);
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to update source: ${errorMsg}`);
    error.value = `Failed to update source: ${errorMsg}`;
    return false;
  } finally {
    savingSource.value = false;
    console.log('saveEdit finally block - savingSource set to false');
  }
};

const removeSource = async (id: number) => {
  const sourceName = sources.value.find(s => s.id === id)?.name || 'source';
  
  const confirmed = await ask(
    `Are you sure you want to remove the source: ${sourceName}? This will also delete all items from this source.`,
    {
      title: 'Delete Source',
      kind: 'warning',
      okLabel: 'Confirm',
      cancelLabel: 'Cancel',
    } as MessageDialogOptions
  );
  
  if (!confirmed) return;
  
  deletingSources.value.add(id);
  
  try {
    await removeSourceAction(id);
    await fetchSources();
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    error.value = `Failed to remove source: ${errorMsg}`;
  } finally {
    deletingSources.value.delete(id);
  }
};

const handleSyncSource = async (id: number) => {
  syncingSources.value.add(id);
  try {
    await syncSource(id);
    await fetchSources();
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    console.error('Failed to sync source:', e);
    alert(`Failed to sync source: ${errorMsg}`);
  } finally {
    syncingSources.value.delete(id);
  }
};

const handleSyncAll = async () => {
  syncingAll.value = true;
  try {
    await syncAllSources();
    await fetchSources();
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    console.error('Failed to sync all sources:', e);
    alert(`Failed to sync all sources: ${errorMsg}`);
  } finally {
    syncingAll.value = false;
  }
};

const openAddGroupPanel = () => {
  editingGroup.value = { id: null, name: '' };
  groupForm.value.name = '';
  groupForm.value.sourceIds = [];
};

const editGroup = (group: Group) => {
  editingGroup.value = group;
  groupForm.value.name = group.name;
  // Get sources that belong to this group
  groupForm.value.sourceIds = sources.value
    .filter(s => s.group_ids?.includes(group.id))
    .map(s => s.id);
};

const closeGroupPanel = () => {
  editingGroup.value = null;
  groupForm.value.name = '';
  groupForm.value.sourceIds = [];
};

const saveGroup = async (e?: Event) => {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  
  if (savingGroup.value) return false;
  if (!editingGroup.value) return false;
  
  if (!groupForm.value.name.trim()) {
    alert('Please enter a group name');
    return false;
  }
  
  savingGroup.value = true;
  try {
    let groupId: number;
    if (editingGroup.value.id) {
      // Update existing group
      await updateGroup(editingGroup.value.id, groupForm.value.name);
      groupId = editingGroup.value.id;
      
      // Update source-group relationships
      for (const source of sources.value) {
        const shouldHaveGroup = groupForm.value.sourceIds.includes(source.id);
        const currentlyHasGroup = source.group_ids?.includes(groupId) || false;
        
        if (shouldHaveGroup && !currentlyHasGroup) {
          const newGroupIds = [...(source.group_ids || []), groupId];
          await updateSource(source.id, { group_ids: newGroupIds });
        } else if (!shouldHaveGroup && currentlyHasGroup) {
            const newGroupIds = (source.group_ids || []).filter(id => id !== groupId);
            await updateSource(source.id, { group_ids: newGroupIds.length > 0 ? newGroupIds : [] });
        }
      }
    } else {
      // Create new group
      groupId = await addGroup(groupForm.value.name);
      
      // Assign sources to the new group
      if (groupForm.value.sourceIds.length > 0) {
        await fetchGroups();
        for (const sourceId of groupForm.value.sourceIds) {
          const source = sources.value.find(s => s.id === sourceId);
          if (source) {
            const newGroupIds = [...(source.group_ids || []), groupId];
            await updateSource(sourceId, { group_ids: newGroupIds });
          }
        }
      }
    }
    
    closeGroupPanel();
    await fetchGroups();
    await fetchSources();
    return false;
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to save group: ${errorMsg}`);
    return false;
  } finally {
    savingGroup.value = false;
  }
};

const removeGroup = async (id: number) => {
  const group = groups.value.find(g => g.id === id);
  if (!group) return;
  
  const sourceCount = getGroupSourceCount(id);
  const confirmed = await ask(
    `Are you sure you want to delete the group "${group.name}"?${sourceCount > 0 ? ` This will remove it from ${sourceCount} source${sourceCount !== 1 ? 's' : ''}.` : ''}`,
    {
      title: 'Delete Group',
      kind: 'warning',
      okLabel: 'Confirm',
      cancelLabel: 'Cancel',
    } as MessageDialogOptions
  );
  
  if (!confirmed) return;
  
  try {
    console.log('removeGroup: calling removeGroupAction with id:', id);
    await removeGroupAction(id);
    console.log('removeGroup: removeGroupAction completed');
    await fetchGroups();
    console.log('removeGroup: fetchGroups completed');
    await fetchSources();
    console.log('removeGroup: fetchSources completed');
  } catch (e) {
    console.error('removeGroup: error caught:', e);
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to remove group: ${errorMsg}`);
    error.value = `Failed to remove group: ${errorMsg}`;
  }
};


const formatDate = (timestamp: number) => {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
};

onMounted(async () => {
  await fetchSources();
  await fetchGroups();
});
</script>

<style scoped>
.source-config {
  padding: 20px;
  max-width: 1000px;
  margin: 0 auto;
}

.sticky-header {
  position: sticky;
  top: 0;
  z-index: 100;
  background: white;
  padding: 12px 20px;
  margin: -20px -20px 20px -20px;
  border-bottom: 1px solid #e0e0e0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  backdrop-filter: blur(10px);
  background: rgba(255, 255, 255, 0.95);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.action-button {
  padding: 8px 16px;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
  border-radius: 4px;
  font-size: 14px;
}

.action-button.primary {
  background: #396cd8;
  color: white;
  border-color: #396cd8;
}

.action-button:hover:not(:disabled) {
  background: #f5f5f5;
}

.action-button.primary:hover:not(:disabled) {
  background: #2952b8;
}

.action-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
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
  gap: 8px;
  align-items: center;
}

.icon-button {
  padding: 6px 10px;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
  border-radius: 4px;
  font-size: 16px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 32px;
  height: 32px;
}

.icon-button:hover:not(:disabled) {
  background: #f5f5f5;
}

.icon-button.delete {
  color: #d32f2f;
  border-color: #d32f2f;
}

.icon-button.delete:hover:not(:disabled) {
  background: #ffebee;
}

.icon-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.source-info {
  font-size: 14px;
  color: #666;
}

.group-management-section {
  border-top: 2px solid #ddd;
  margin-top: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #666;
  font-style: italic;
}

.groups-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.group-card {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 16px;
  background: white;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.group-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.group-name {
  font-weight: 500;
  font-size: 16px;
}

.group-count {
  font-size: 12px;
  color: #666;
}

.group-actions {
  display: flex;
  gap: 8px;
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
  max-width: 100%;
  width: 100%;
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
  box-sizing: border-box;
}

.form-group input[type="checkbox"] {
  margin-right: 8px;
}


.checkbox-group {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  max-height: 200px;
  overflow-y: auto;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: #f9f9f9;
}

@media (max-width: 768px) {
  .checkbox-group {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 480px) {
  .checkbox-group {
    grid-template-columns: 1fr;
  }
}

.checkbox-option {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  padding: 4px;
}

.checkbox-option:hover {
  background: #f0f0f0;
  border-radius: 4px;
}

.no-groups-hint {
  font-size: 12px;
  color: #666;
  font-style: italic;
  margin: 0;
  padding: 8px;
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

