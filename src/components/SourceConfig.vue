<template>
  <div class="source-config">
    <!-- Sticky Header -->
    <div class="sticky-header">
      <div class="header-left">
        <h1>Settings</h1>
      </div>
    </div>

    <div v-if="loading" class="loading">Loading sources and groups...</div>
    <div v-if="error" class="error-message">
      Error: {{ error }}
      <button @click="error = null" class="dismiss-error">×</button>
    </div>

    <!-- Sources Section -->
    <div class="settings-section">
      <div class="section-header">
        <h2>Sources</h2>
        <div class="section-actions">
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
    </div>

    <!-- Secrets Section -->
    <div class="settings-section">
      <div class="section-header">
        <h2>Security & Secrets</h2>
        <div class="section-actions">
          <button @click="showAddSecretModal = true" class="action-button primary">
            + Add Secret
          </button>
        </div>
      </div>
      
      <div v-if="secretsLoading" class="loading">Loading secrets...</div>
      <div v-if="secretsError" class="error-message">
        Error: {{ secretsError }}
        <button @click="secretsError = null" class="dismiss-error">×</button>
      </div>
      
      <div v-if="!secretsLoading && secrets.length === 0" class="empty-state">
        <p>No secrets yet. Create your first secret to store API tokens.</p>
      </div>
      
      <div v-else-if="!secretsLoading" class="secrets-list">
        <div v-for="secret in secrets" :key="secret.id" class="secret-card" :class="getSecretHealthClass(secret)">
          <div class="secret-header">
            <div>
              <div style="display: flex; align-items: center; gap: 8px;">
                <h3>{{ secret.name }}</h3>
                <span v-if="getSecretHealthStatus(secret) === 'error'" class="token-status-badge error" title="Token has issues">
                  ⚠️ Token Issues
                </span>
                <span v-else-if="getSecretHealthStatus(secret) === 'warning'" class="token-status-badge warning" title="Token may have issues">
                  ⚠️ Warning
                </span>
                <span v-else-if="getSecretHealthStatus(secret) === 'testing'" class="token-status-badge testing" title="Testing token...">
                  🔄 Testing...
                </span>
                <span v-else-if="getSecretHealthStatus(secret) === 'good'" class="token-status-badge good" title="Token is valid">
                  ✓ Valid
                </span>
              </div>
              <div class="secret-meta">
                <span class="ttl-type">{{ secret.ttl_type }}</span>
                <span v-if="secret.expires_at" class="expiry-info">
                  Expires: {{ formatDate(secret.expires_at) }}
                </span>
                <span v-else class="expiry-info">Never expires</span>
                <span class="secret-count">
                  {{ getSecretSourceCount(secret.id) }} source{{ getSecretSourceCount(secret.id) !== 1 ? 's' : '' }}
                </span>
                <span v-if="secret.refresh_failure_count && secret.refresh_failure_count > 0" class="failure-count" style="color: #d32f2f; font-weight: 500;">
                  ({{ secret.refresh_failure_count }} refresh failure{{ secret.refresh_failure_count !== 1 ? 's' : '' }})
                </span>
              </div>
            </div>
            <div class="secret-actions">
              <button 
                @click="testSecretToken(secret.id)" 
                class="icon-button"
                :disabled="testingSecrets.has(secret.id)"
                :title="testingSecrets.has(secret.id) ? 'Testing...' : 'Test Token'"
              >
                {{ testingSecrets.has(secret.id) ? '🔄' : '🧪' }}
              </button>
              <button 
                @click="editSecret(secret)" 
                class="icon-button"
                title="Edit"
              >
                ✏️
              </button>
              <button 
                @click="removeSecret(secret.id)" 
                class="icon-button delete"
                :disabled="deletingSecrets.has(secret.id)"
                title="Delete"
              >
                🗑️
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Groups Section -->
    <div class="settings-section">
      <div class="section-header">
        <h2>Groups</h2>
        <div class="section-actions">
          <button @click="openAddGroupPanel" class="action-button primary">
            + Add Group
          </button>
        </div>
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

    <!-- Theme Section -->
    <div class="settings-section">
      <div class="section-header">
        <h2>Appearance</h2>
      </div>
      
      <div class="theme-selector-section">
      <label class="theme-selector-label">Theme</label>
      <div class="theme-options">
        <label 
          class="theme-option"
          :class="{ active: currentTheme === 'system' }"
        >
          <input 
            type="radio" 
            value="system" 
            :checked="currentTheme === 'system'"
            @change="handleThemeChange('system')"
          />
          <span class="theme-name">System</span>
          <span class="theme-preview">
            <span class="theme-swatch" style="background: #ffffff; border-color: #000000;"></span>
            <span class="theme-swatch" style="background: #1a1a1a; border-color: #ffffff;"></span>
            <span class="system-indicator" v-if="currentTheme === 'system'">
              ({{ systemPreference === 'dark' ? 'Dark' : 'Light' }})
            </span>
          </span>
        </label>
        <label 
          class="theme-option"
          :class="{ active: currentTheme === 'light' }"
        >
          <input 
            type="radio" 
            value="light" 
            :checked="currentTheme === 'light'"
            @change="handleThemeChange('light')"
          />
          <span class="theme-name">Light</span>
          <span class="theme-preview">
            <span class="theme-swatch" style="background: #ffffff;"></span>
          </span>
        </label>
        <label 
          class="theme-option"
          :class="{ active: currentTheme === 'dark' }"
        >
          <input 
            type="radio" 
            value="dark" 
            :checked="currentTheme === 'dark'"
            @change="handleThemeChange('dark')"
          />
          <span class="theme-name">Dark</span>
          <span class="theme-preview">
            <span class="theme-swatch" style="background: #1a1a1a;"></span>
          </span>
        </label>
        <label 
          class="theme-option"
          :class="{ active: currentTheme === 'blue' }"
        >
          <input 
            type="radio" 
            value="blue" 
            :checked="currentTheme === 'blue'"
            @change="handleThemeChange('blue')"
          />
          <span class="theme-name">Blue</span>
          <span class="theme-preview">
            <span class="theme-swatch" style="background: #2196f3;"></span>
          </span>
        </label>
        <label 
          class="theme-option"
          :class="{ active: currentTheme === 'liquid-glass' }"
        >
          <input 
            type="radio" 
            value="liquid-glass" 
            :checked="currentTheme === 'liquid-glass'"
            @change="handleThemeChange('liquid-glass')"
          />
          <span class="theme-name">Liquid Glass</span>
          <span class="theme-preview">
            <span class="theme-swatch" style="background: linear-gradient(135deg, rgba(255,255,255,0.7), rgba(255,255,255,0.3)); border: 1px solid rgba(255,255,255,0.3); backdrop-filter: blur(10px);"></span>
          </span>
        </label>
      </div>
        <p class="theme-description">
          Choose how UmbraRelay looks. System follows your OS preference.
        </p>
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
      <div class="form-group">
        <label>Source Type</label>
        <div class="source-type-select-wrapper">
          <select v-model="newSourceType" class="source-type-select">
            <option value="rss">RSS Feed</option>
            <option value="atom">ATOM Feed</option>
            <option value="github">GitHub</option>
            <option value="github_notifications">GitHub Personal Notifications</option>
          </select>
          <span class="source-type-icon" :class="`icon-${newSourceType}`">
            <span v-if="newSourceType === 'rss'" class="icon-emoji">📡</span>
            <span v-else-if="newSourceType === 'atom'" class="icon-emoji">⚛️</span>
            <svg v-else-if="newSourceType === 'github' || newSourceType === 'github_notifications'" 
                 class="icon-svg github-icon" 
                 width="16" 
                 height="16" 
                 viewBox="0 0 16 16" 
                 fill="currentColor" 
                 xmlns="http://www.w3.org/2000/svg">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z"/>
            </svg>
          </span>
        </div>
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

          <!-- ATOM Form -->
          <form v-if="newSourceType === 'atom'" @submit.prevent="addAtomSource" class="source-form" novalidate>
        <div class="form-group">
          <label>Name</label>
          <input v-model="atomForm.name" type="text" required placeholder="e.g., Example Blog" />
        </div>
        <div class="form-group">
          <label>URL</label>
          <input v-model="atomForm.url" type="url" required placeholder="https://example.com/atom.xml" />
        </div>
        <div class="form-group">
          <label>Poll Interval (optional)</label>
          <input v-model="atomForm.pollInterval" type="text" placeholder="10m" />
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
                    v-model="atomForm.groupIds"
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
        <button type="submit" class="submit-button">Add ATOM Feed</button>
            </div>
      </form>

          <!-- GitHub Form -->
          <form v-if="newSourceType === 'github'" @submit.prevent="addGitHubSource" class="source-form" novalidate>
        <div class="form-group">
          <label>Name</label>
          <input v-model="githubForm.name" type="text" required placeholder="e.g., My Project" />
        </div>
        <div class="form-group">
          <label>Authentication</label>
          <button 
            v-if="!githubForm.secretId" 
            type="button"
            @click="startGitHubOAuth"
            :disabled="githubForm.oauthInProgress"
            class="oauth-button"
          >
            {{ githubForm.oauthInProgress ? 'Authorizing...' : 'Authorize UmbraRelay with GitHub' }}
          </button>
          <div v-else class="auth-status">
            <span>✓ Authorized</span>
            <button 
              type="button" 
              @click="handleReauthorizeGitHub" 
              class="link-button"
            >
              Re-authorize
            </button>
          </div>
        </div>
        <div v-if="githubForm.availableRepos.length > 0" class="form-group">
          <label>Repositories</label>
          <div class="repo-multiselect">
            <div class="repo-multiselect-header">
              <input 
                type="text" 
                v-model="githubForm.repoSearch"
                placeholder="Search repositories..."
                class="repo-search-input"
                @focus="githubForm.showRepoDropdown = true"
                @blur="handleRepoSearchBlur"
              />
              <div class="repo-select-actions">
                <button type="button" @click="selectAllRepos" class="link-button small">Select All</button>
                <span>|</span>
                <button type="button" @click="deselectAllRepos" class="link-button small">Deselect All</button>
              </div>
            </div>
            <div 
              class="repo-multiselect-dropdown"
              :class="{ 'show': githubForm.showRepoDropdown }"
              @click.stop
            >
              <div class="repo-multiselect-list">
                <label 
                  v-for="repo in filteredRepos" 
                  :key="repo.id"
                  class="checkbox-option"
                  @mousedown="handleRepoCheckboxClick"
                >
                  <input 
                    type="checkbox" 
                    :value="repo.full_name"
                    v-model="githubForm.repositories"
                  />
                  <span>{{ repo.full_name }} {{ repo.private ? '(private)' : '' }}</span>
                </label>
              </div>
            </div>
            <div class="repo-selected-count" v-if="githubForm.repositories.length > 0">
              {{ githubForm.repositories.length }} repository{{ githubForm.repositories.length === 1 ? '' : 'ies' }} selected
            </div>
            <p v-if="githubForm.repositories.length === 0" class="hint">
              Select at least one repository to monitor
            </p>
          </div>
        </div>
        <div class="form-group">
          <label>Account Level Data</label>
          <div class="checkbox-group account-data">
            <label class="checkbox-option">
              <input type="checkbox" value="events" v-model="githubForm.endpoints" />
              <span>Events</span>
            </label>
          </div>
        </div>
        <div class="form-group">
          <div class="form-group-header">
            <label>Repo Level Data</label>
            <div class="select-actions">
              <button type="button" @click="selectAllRepoData" class="link-button small">Select All</button>
              <span>|</span>
              <button type="button" @click="deselectAllRepoData" class="link-button small">Deselect All</button>
            </div>
          </div>
          <div class="checkbox-group repo-data">
            <label class="checkbox-option">
              <input type="checkbox" value="actions" v-model="githubForm.endpoints" />
              <span>Actions</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="administration" v-model="githubForm.endpoints" />
              <span>Administration</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="checks" v-model="githubForm.endpoints" />
              <span>Checks</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="code_scanning_alerts" v-model="githubForm.endpoints" />
              <span>Code Scanning Alerts</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="commits" v-model="githubForm.endpoints" />
              <span>Commits</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="contents" v-model="githubForm.endpoints" />
              <span>Contents</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="discussions" v-model="githubForm.endpoints" />
              <span>Discussions</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="issues" v-model="githubForm.endpoints" />
              <span>Issues</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="metadata" v-model="githubForm.endpoints" />
              <span>Metadata</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="packages" v-model="githubForm.endpoints" />
              <span>Packages</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="projects" v-model="githubForm.endpoints" />
              <span>Projects</span>
            </label>
            <label class="checkbox-option">
              <input type="checkbox" value="prs" v-model="githubForm.endpoints" />
              <span>Pull Requests</span>
            </label>
          </div>
        </div>
        <div class="form-group">
          <label>Poll Interval (optional)</label>
          <input v-model="githubForm.pollInterval" type="text" placeholder="10m" />
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
        <button type="submit" class="submit-button" :disabled="!githubForm.secretId || githubForm.repositories.length === 0">Add GitHub Source</button>
            </div>
      </form>

          <!-- GitHub Personal Notifications Form -->
          <form v-if="newSourceType === 'github_notifications'" @submit.prevent="addGitHubNotificationsSource" class="source-form" novalidate>
        <div class="info-box" style="background: #e3f2fd; border-left: 4px solid #2196f3; padding: 12px; margin-bottom: 16px; border-radius: 4px;">
          <h4 style="margin: 0 0 8px 0; color: #1976d2; font-size: 14px; font-weight: 600;">Personal Access Token Required</h4>
          <p style="margin: 0 0 8px 0; color: #424242; font-size: 13px;">
            GitHub Personal Notifications requires a Personal Access Token (PAT) with the <strong>notifications</strong> scope.
          </p>
          <ol style="margin: 8px 0; padding-left: 20px; color: #424242; font-size: 13px;">
            <li>Go to <a href="https://github.com/settings/tokens" target="_blank" style="color: #1976d2; text-decoration: underline;">GitHub Settings → Developer settings → Personal access tokens</a></li>
            <li>Click "Generate new token (classic)"</li>
            <li>Give it a name (e.g., "UmbraRelay Notifications")</li>
            <li>Select the <strong>notifications</strong> scope (read-only)</li>
            <li>Click "Generate token" and copy it</li>
            <li>Create a secret below or select an existing one</li>
          </ol>
        </div>
        
        <div class="form-group">
          <label>Name</label>
          <input v-model="githubNotificationsForm.name" type="text" required placeholder="e.g., My GitHub Notifications" />
        </div>
        
        <div class="form-group">
          <label>Personal Access Token</label>
          <select v-model="githubNotificationsForm.secretId" @change="handleGitHubNotificationsSecretChange">
            <option :value="null">Create new secret...</option>
            <option v-for="secret in secrets" :key="secret.id" :value="secret.id">
              {{ secret.name }}
            </option>
          </select>
          <div v-if="!githubNotificationsForm.secretId" class="form-group" style="margin-top: 8px;">
            <input v-model="githubNotificationsForm.token" type="password" placeholder="ghp_..." style="width: 100%;" />
            <small style="color: #666; font-size: 12px; display: block; margin-top: 4px;">
              Paste your PAT here. It will be stored securely as a secret.
            </small>
          </div>
        </div>
        
        <div class="form-group">
          <label>Poll Interval (optional)</label>
          <input v-model="githubNotificationsForm.pollInterval" type="text" placeholder="10m" />
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
                v-model="githubNotificationsForm.groupIds"
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
          <button type="submit" class="submit-button" :disabled="!githubNotificationsForm.secretId && !githubNotificationsForm.token">Add GitHub Notifications Source</button>
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
              <div style="margin-top: 12px;">
                <label style="font-size: 14px; color: #666; margin-bottom: 4px; display: block;">Create New Groups</label>
                <input 
                  v-model="editForm.newGroupsInput" 
                  type="text" 
                  placeholder="Type group name and press comma to add"
                  @keydown="handleNewGroupKeydown('edit', $event)"
                  @blur="handleNewGroupBlur('edit')"
                  style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                />
                <div v-if="editForm.newGroups.length > 0" style="margin-top: 8px; display: flex; flex-wrap: wrap; gap: 6px;">
                  <span 
                    v-for="(groupName, index) in editForm.newGroups"
                    :key="index" 
                    style="display: inline-flex; align-items: center; gap: 6px; background: #e3f2fd; color: #1976d2; padding: 4px 8px; border-radius: 12px; font-size: 12px; font-weight: 500;"
                  >
                    {{ groupName }}
                    <button 
                      type="button"
                      @click="removeNewGroup('edit', index)"
                      style="background: none; border: none; color: #1976d2; cursor: pointer; padding: 0; width: 16px; height: 16px; display: flex; align-items: center; justify-content: center; font-size: 16px; line-height: 1;"
                      title="Remove"
                    >
                      ×
                    </button>
                  </span>
                </div>
                <small style="color: #666; font-size: 12px; margin-top: 4px; display: block;">
                  Type a group name and press comma to add it. Groups will be created when you save.
                </small>
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

          <!-- ATOM Edit Form -->
          <form v-if="editingSource && editingSource.source_type === 'atom'" @submit.prevent="saveEdit" class="source-form" novalidate>
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
              <div style="margin-top: 12px;">
                <label style="font-size: 14px; color: #666; margin-bottom: 4px; display: block;">Create New Groups</label>
                <input 
                  v-model="editForm.newGroupsInput" 
                  type="text" 
                  placeholder="Type group name and press comma to add"
                  @keydown="handleNewGroupKeydown('edit', $event)"
                  @blur="handleNewGroupBlur('edit')"
                  style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                />
                <div v-if="editForm.newGroups.length > 0" style="margin-top: 8px; display: flex; flex-wrap: wrap; gap: 6px;">
                  <span 
                    v-for="(groupName, index) in editForm.newGroups"
                    :key="index" 
                    style="display: inline-flex; align-items: center; gap: 6px; background: #e3f2fd; color: #1976d2; padding: 4px 8px; border-radius: 12px; font-size: 12px; font-weight: 500;"
                  >
                    {{ groupName }}
                    <button 
                      type="button"
                      @click="removeNewGroup('edit', index)"
                      style="background: none; border: none; color: #1976d2; cursor: pointer; padding: 0; width: 16px; height: 16px; display: flex; align-items: center; justify-content: center; font-size: 16px; line-height: 1;"
                      title="Remove"
                    >
                      ×
                    </button>
                  </span>
                </div>
                <small style="color: #666; font-size: 12px; margin-top: 4px; display: block;">
                  Type a group name and press comma to add it. Groups will be created when you save.
                </small>
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
              <label>Authentication</label>
              <div v-if="editForm.secretId" class="auth-status">
                <span>✓ Authorized</span>
                <button type="button" @click="reauthorizeGitHub" :disabled="editForm.oauthInProgress" class="link-button">
                  {{ editForm.oauthInProgress ? 'Authorizing...' : 'Re-authorize' }}
                </button>
              </div>
              <button 
                v-else
                type="button"
                @click="reauthorizeGitHub"
                :disabled="editForm.oauthInProgress"
                class="oauth-button"
              >
                {{ editForm.oauthInProgress ? 'Authorizing...' : 'Authorize UmbraRelay with GitHub' }}
              </button>
            </div>
            <div v-if="editForm.availableRepos.length > 0" class="form-group">
              <label>Repositories</label>
              <div class="repo-multiselect">
                <div class="repo-multiselect-header">
                  <input 
                    type="text" 
                    v-model="editForm.repoSearch"
                    placeholder="Search repositories..."
                    class="repo-search-input"
                    @focus="editForm.showRepoDropdown = true"
                    @blur="handleEditRepoSearchBlur"
                  />
                  <div class="repo-select-actions">
                    <button type="button" @click="selectAllEditRepos" class="link-button small">Select All</button>
                    <span>|</span>
                    <button type="button" @click="deselectAllEditRepos" class="link-button small">Deselect All</button>
                  </div>
                </div>
                <div 
                  class="repo-multiselect-dropdown"
                  :class="{ 'show': editForm.showRepoDropdown }"
                  @click.stop
                >
                  <div class="repo-multiselect-list">
                    <label 
                      v-for="repo in filteredEditRepos" 
                      :key="repo.id"
                      class="checkbox-option"
                      @mousedown="handleEditRepoCheckboxClick"
                    >
                      <input 
                        type="checkbox" 
                        :value="repo.full_name"
                        v-model="editForm.repositories"
                      />
                      <span>{{ repo.full_name }} {{ repo.private ? '(private)' : '' }}</span>
                    </label>
                  </div>
                </div>
                <div class="repo-selected-count" v-if="editForm.repositories.length > 0">
                  {{ editForm.repositories.length }} repository{{ editForm.repositories.length === 1 ? '' : 'ies' }} selected
                </div>
              </div>
            </div>
            <div class="form-group">
              <label>Account Level Data</label>
              <div class="checkbox-group account-data">
                <label class="checkbox-option">
                  <input type="checkbox" value="events" v-model="editForm.endpoints" />
                  <span>Events</span>
                </label>
              </div>
            </div>
            <div class="form-group">
              <div class="form-group-header">
                <label>Repo Level Data</label>
                <div class="select-actions">
                  <button type="button" @click="selectAllEditRepoData" class="link-button small">Select All</button>
                  <span>|</span>
                  <button type="button" @click="deselectAllEditRepoData" class="link-button small">Deselect All</button>
                </div>
              </div>
              <div class="checkbox-group repo-data">
                <label class="checkbox-option">
                  <input type="checkbox" value="actions" v-model="editForm.endpoints" />
                  <span>Actions</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="administration" v-model="editForm.endpoints" />
                  <span>Administration</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="checks" v-model="editForm.endpoints" />
                  <span>Checks</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="code_scanning_alerts" v-model="editForm.endpoints" />
                  <span>Code Scanning Alerts</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="commits" v-model="editForm.endpoints" />
                  <span>Commits</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="contents" v-model="editForm.endpoints" />
                  <span>Contents</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="discussions" v-model="editForm.endpoints" />
                  <span>Discussions</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="issues" v-model="editForm.endpoints" />
                  <span>Issues</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="metadata" v-model="editForm.endpoints" />
                  <span>Metadata</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="packages" v-model="editForm.endpoints" />
                  <span>Packages</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="projects" v-model="editForm.endpoints" />
                  <span>Projects</span>
                </label>
                <label class="checkbox-option">
                  <input type="checkbox" value="prs" v-model="editForm.endpoints" />
                  <span>Pull Requests</span>
                </label>
              </div>
            </div>
            <div class="form-group">
              <label>Poll Interval (optional)</label>
              <input v-model="editForm.pollIntervalGitHub" type="text" placeholder="10m" />
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
              <div style="margin-top: 12px;">
                <label style="font-size: 14px; color: #666; margin-bottom: 4px; display: block;">Create New Groups</label>
                <input 
                  v-model="editForm.newGroupsInput" 
                  type="text" 
                  placeholder="Type group name and press comma to add"
                  @keydown="handleNewGroupKeydown('edit', $event)"
                  @blur="handleNewGroupBlur('edit')"
                  style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                />
                <div v-if="editForm.newGroups.length > 0" style="margin-top: 8px; display: flex; flex-wrap: wrap; gap: 6px;">
                  <span
                    v-for="(groupName, index) in editForm.newGroups"
                    :key="index"
                    style="display: inline-flex; align-items: center; gap: 6px; background: #e3f2fd; color: #1976d2; padding: 4px 8px; border-radius: 12px; font-size: 12px; font-weight: 500;"
                  >
                    {{ groupName }}
                    <button
                      type="button"
                      @click="removeNewGroup('edit', index)"
                      style="background: none; border: none; color: #1976d2; cursor: pointer; padding: 0; width: 16px; height: 16px; display: flex; align-items: center; justify-content: center; font-size: 16px; line-height: 1;"
                      title="Remove"
                    >
                      ×
                    </button>
                  </span>
                </div>
                <small style="color: #666; font-size: 12px; margin-top: 4px; display: block;">
                  Type a group name and press comma to add it. Groups will be created when you save.
                </small>
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

    <!-- Add/Edit Secret Modal -->
    <div v-if="showAddSecretModal || editingSecret" class="edit-panel-overlay" @click="closeSecretModal">
      <div class="edit-panel" @click.stop>
        <div class="edit-panel-header">
          <h2>{{ editingSecret ? 'Edit Secret' : 'Add Secret' }}</h2>
          <button @click="closeSecretModal" class="close-button" title="Close">×</button>
        </div>
        
        <div class="edit-panel-content">
          <form @submit.prevent="saveSecret" class="source-form" novalidate>
            <div class="form-group">
              <label>Name</label>
              <input v-model="secretForm.name" type="text" required placeholder="e.g., GitHub Token" />
            </div>
            <div class="form-group">
              <label>Value</label>
              <input 
                v-model="secretForm.value" 
                type="password" 
                :required="!editingSecret"
                placeholder="Enter secret value"
              />
              <small v-if="editingSecret" class="hint">Leave blank to keep existing value</small>
              <small v-if="!editingSecret" class="hint">
                For GitHub tokens, expiration will be automatically detected if possible.
              </small>
            </div>
            <div class="form-group">
              <label>Expiry Type</label>
              <select v-model="secretForm.ttlType" @change="onTtlTypeChange">
                <option value="forever">Forever (No expiration)</option>
                <option value="relative">Relative (e.g., 30 days)</option>
                <option value="absolute">Absolute Date</option>
              </select>
              <small class="hint">If "Forever" is selected, we'll try to detect expiration from the token.</small>
            </div>
            <div v-if="secretForm.ttlType === 'relative'" class="form-group">
              <label>Duration</label>
              <input 
                v-model="secretForm.ttlValue" 
                type="text" 
                placeholder="e.g., 30d, 1w, 6M, 1y"
              />
              <small class="hint">Format: number + unit (s, m, h, d, w, M, y)</small>
            </div>
            <div v-if="secretForm.ttlType === 'absolute'" class="form-group">
              <label>Expiry Date</label>
              <input 
                v-model="secretForm.ttlValue" 
                type="datetime-local"
              />
            </div>
            <div class="form-actions">
              <button type="button" @click="closeSecretModal" class="cancel-button">Cancel</button>
              <button type="submit" class="submit-button">
                {{ editingSecret ? 'Save Changes' : 'Create Secret' }}
              </button>
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
              <label>Group Name{{ editingGroup && !editingGroup.id ? 's' : '' }}</label>
              <input 
                v-if="editingGroup && editingGroup.id"
                v-model="groupForm.name" 
                type="text" 
                required
                placeholder="e.g., Work, Personal, etc." 
              />
              <div v-else>
                <input 
                  v-model="groupForm.newGroupsInput" 
                  type="text" 
                  required
                  placeholder="Type group name and press comma to add"
                  @keydown="handleNewGroupKeydown('group', $event)"
                  @blur="handleNewGroupBlur('group')"
                  style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                />
                <div v-if="groupForm.newGroups.length > 0" style="margin-top: 8px; display: flex; flex-wrap: wrap; gap: 6px;">
                  <span
                    v-for="(groupName, index) in groupForm.newGroups"
                    :key="index"
                    style="display: inline-flex; align-items: center; gap: 6px; background: #e3f2fd; color: #1976d2; padding: 4px 8px; border-radius: 12px; font-size: 12px; font-weight: 500;"
                  >
                    {{ groupName }}
                    <button
                      type="button"
                      @click="removeNewGroup('group', index)"
                      style="background: none; border: none; color: #1976d2; cursor: pointer; padding: 0; width: 16px; height: 16px; display: flex; align-items: center; justify-content: center; font-size: 16px; line-height: 1;"
                      title="Remove"
                    >
                      ×
                    </button>
                  </span>
                </div>
                <small style="color: #666; font-size: 12px; margin-top: 4px; display: block;">
                  Type a group name and press comma to add it. Groups will be created when you save.
                </small>
              </div>
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

    <!-- GitHub Auth Modal -->
    <div v-if="showGitHubAuthModal" class="github-auth-modal" @click.self="showGitHubAuthModal = false">
      <div class="github-auth-modal-content">
        <h2>GitHub Device Flow Authorization</h2>
        <p>Enter this code when prompted:</p>
        <div class="github-auth-code">{{ githubAuthCode }}</div>
        <p style="font-size: 14px; color: var(--color-text-secondary); margin-top: 8px;">
          After authorizing in GitHub, click OK to start polling.
        </p>
        <div class="github-auth-actions">
          <button 
            class="copy-button" 
            @click="copyGitHubCode"
          >
            Copy Code
          </button>
          <button 
            class="ok-button" 
            @click="handleGitHubAuthProceed"
          >
            OK
          </button>
          <button 
            class="cancel-button" 
            @click="handleGitHubAuthCancel"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { useSources } from '../composables/useSources';
import { useGroups } from '../composables/useGroups';
import { useTheme } from '../composables/useTheme';
import { ask, MessageDialogOptions } from '@tauri-apps/plugin-dialog';
import type { Source, SourceInput, UpdateSourceInput, Group } from '../types';
import { formatDate } from '../utils/formatting';

const { sources, loading, error, fetchSources, addSource, updateSource, removeSource: removeSourceAction, syncSource, syncAllSources } = useSources();
const { groups, fetchGroups, addGroup, updateGroup, removeGroup: removeGroupAction } = useGroups();
const { currentTheme, systemPreference, setTheme } = useTheme();

const syncingSources = ref<Set<number>>(new Set());
const deletingSources = ref<Set<number>>(new Set());
const syncingAll = ref(false);
const editingSource = ref<Source | null>(null);
const showAddSourceModal = ref(false);
const editingGroup = ref<Group | { id: null; name: string } | null>(null);
const savingGroup = ref(false);
const savingSource = ref(false);
const addingSource = ref(false);
const secrets = ref<any[]>([]);
const secretsLoading = ref(false);
const secretsError = ref<string | null>(null);
const showAddSecretModal = ref(false);
const editingSecret = ref<any | null>(null);
const deletingSecrets = ref<Set<number>>(new Set());
const testingSecrets = ref<Set<number>>(new Set());
const secretHealthStatus = ref<Map<number, 'good' | 'warning' | 'error' | 'testing'>>(new Map());
const sourceSecretMap = ref<Map<number, number>>(new Map()); // Maps source_id -> secret_id

const secretForm = ref({
  name: '',
  value: '',
  ttlType: 'forever' as 'forever' | 'relative' | 'absolute',
  ttlValue: '',
});

const newSourceType = ref<'rss' | 'atom' | 'github' | 'github_notifications'>('rss');

const rssForm = ref({
  name: '',
  url: '',
  pollInterval: '10m',
  groupIds: [] as number[],
});

const atomForm = ref({
  name: '',
  url: '',
  pollInterval: '10m',
  groupIds: [] as number[],
});

const githubForm = ref({
  name: '',
  secretId: null as number | null,
  repositories: [] as string[],
  endpoints: ['commits', 'prs'] as string[],
  pollInterval: '10m',
  groupIds: [] as number[],
  oauthInProgress: false,
  availableRepos: [] as any[],
  repoSearch: '' as string,
  showRepoDropdown: false as boolean,
});

const githubNotificationsForm = ref({
  name: '',
  secretId: null as number | null,
  token: '' as string,
  pollInterval: '10m',
  groupIds: [] as number[],
});

const showGitHubAuthModal = ref(false);
const githubAuthCode = ref('');
const githubAuthProceed = ref<(() => void) | null>(null);

const editForm = ref({
  name: '',
  url: '',
  pollInterval: '',
  owner: '',
  repo: '',
  token: '',
  assignedOnly: false,
  secretId: null as number | null,
  repositories: [] as string[],
  endpoints: [] as string[],
  pollIntervalGitHub: '10m',
  groupIds: [] as number[],
  newGroupsInput: '',
  newGroups: [] as string[], // Chips for new groups to create
  repoSearch: '' as string,
  showRepoDropdown: false as boolean,
  enabled: true,
  availableRepos: [] as any[],
  oauthInProgress: false,
});

const groupForm = ref({
  name: '',
  newGroupsInput: '',
  newGroups: [] as string[], // Chips for new groups to create
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

// Get count of sources using a secret
const getSecretSourceCount = (secretId: number): number => {
  let count = 0;
  for (const [, mappedSecretId] of sourceSecretMap.value.entries()) {
    if (mappedSecretId === secretId) {
      count++;
    }
  }
  return count;
};


const closeAddSourceModal = () => {
  showAddSourceModal.value = false;
  // Reset forms when closing
  if (newSourceType.value === 'github') {
    githubForm.value = {
      name: '',
      secretId: null,
      repositories: [],
      endpoints: ['commits', 'prs'],
      pollInterval: '10m',
      groupIds: [],
      oauthInProgress: false,
      availableRepos: [],
      repoSearch: '',
      showRepoDropdown: false,
    };
  } else if (newSourceType.value === 'github_notifications') {
    githubNotificationsForm.value = {
      name: '',
      secretId: null,
      token: '',
      pollInterval: '10m',
      groupIds: [],
    };
  }
};

const handleGitHubNotificationsSecretChange = () => {
  // Clear token input when a secret is selected
  if (githubNotificationsForm.value.secretId) {
    githubNotificationsForm.value.token = '';
  }
};

const addGitHubNotificationsSource = async (e?: Event) => {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  
  if (addingSource.value) return false;
  
  if (!githubNotificationsForm.value.name.trim()) {
    alert('Please enter a name for the source');
    return false;
  }
  
  if (!githubNotificationsForm.value.secretId && !githubNotificationsForm.value.token.trim()) {
    alert('Please select an existing secret or enter a Personal Access Token');
    return false;
  }
  
  addingSource.value = true;
  try {
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore || !tauriCore.invoke) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    const { invoke } = tauriCore;
    
    let secretId = githubNotificationsForm.value.secretId;
    
    // If token is provided, create a new secret
    if (!secretId && githubNotificationsForm.value.token.trim()) {
      const secretName = `GitHub PAT - ${githubNotificationsForm.value.name}`;
      secretId = await invoke<number>('create_secret', {
        name: secretName,
        value: githubNotificationsForm.value.token,
        ttlType: 'forever',
        ttlValue: null,
        refreshToken: null,
      });
    }
    
    if (!secretId) {
      alert('Failed to create or select secret');
      return false;
    }
    
    const source: SourceInput = {
      source_type: 'github_notifications',
      name: githubNotificationsForm.value.name,
      config_json: {
        poll_interval: githubNotificationsForm.value.pollInterval || '10m',
      },
      secret_id: secretId,
      group_ids: githubNotificationsForm.value.groupIds.length > 0 ? githubNotificationsForm.value.groupIds : null,
    };
    
    await invoke('add_source', { source });
    await fetchSourcesAndRebuildMap();
    closeAddSourceModal();
    
    // Reset form
    githubNotificationsForm.value = {
      name: '',
      secretId: null,
      token: '',
      pollInterval: '10m',
      groupIds: [],
    };
    return false;
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to add source: ${errorMsg}`);
    return false;
  } finally {
    addingSource.value = false;
  }
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
    await fetchSourcesAndRebuildMap();
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

const addAtomSource = async (e?: Event) => {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  
  if (addingSource.value) return false;
  
  if (!atomForm.value.name.trim()) {
    alert('Please enter a source name');
    return false;
  }
  if (!atomForm.value.url.trim()) {
    alert('Please enter a URL');
    return false;
  }
  
  addingSource.value = true;
  try {
  const source: SourceInput = {
    source_type: 'atom',
    name: atomForm.value.name,
    config_json: {
      url: atomForm.value.url,
      poll_interval: atomForm.value.pollInterval || '10m',
    },
      group_ids: atomForm.value.groupIds.length > 0 ? atomForm.value.groupIds : null,
  };

  await addSource(source);
    await fetchGroups();
    await fetchSourcesAndRebuildMap();
    closeAddSourceModal();
  
  // Reset form
    atomForm.value = { name: '', url: '', pollInterval: '10m', groupIds: [] };
    return false;
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to add source: ${errorMsg}`);
    return false;
  } finally {
    addingSource.value = false;
  }
};

const reauthorizeGitHub = async () => {
  editForm.value.oauthInProgress = true;
  try {
    const tauriCore = await import('@tauri-apps/api/core');
    const openerPlugin = await import('@tauri-apps/plugin-opener');
    if (!tauriCore?.invoke || !openerPlugin?.openUrl) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    const { invoke } = tauriCore;
    const { openUrl } = openerPlugin;
    
    // Start device flow
    const deviceInfo = await invoke<any>('start_github_oauth');
    
    const userCode = deviceInfo.user_code;
    const verificationUrl = deviceInfo.verification_uri_complete;
    
    await openUrl(verificationUrl);
    
    // Show custom modal with copy button
    githubAuthCode.value = userCode;
    showGitHubAuthModal.value = true;
    
    // Wait for user to click OK or Cancel
    const proceed = await new Promise<boolean>((resolve) => {
      githubAuthProceed.value = () => {
        showGitHubAuthModal.value = false;
        githubAuthProceed.value = null;
        resolve(true);
      };
      
      // Also resolve false if modal is closed (cancel button or outside click)
      const checkInterval = setInterval(() => {
        if (!showGitHubAuthModal.value) {
          clearInterval(checkInterval);
          if (githubAuthProceed.value) {
            githubAuthProceed.value = null;
            resolve(false);
          }
        }
      }, 100);
    });
    
    if (!proceed) {
      editForm.value.oauthInProgress = false;
      return;
    }
    
    // Poll for token
    // Calculate max attempts with buffer for potential slow_down interval increases
    // Assume worst case: interval could increase by 5 seconds multiple times
    const baseInterval = deviceInfo.interval;
    const maxAttempts = Math.floor(deviceInfo.expires_in / baseInterval) + 20;
    let attempts = 0;
    let currentPollInterval = baseInterval * 1000; // Convert to milliseconds
    
    // Wait a moment before first poll to give GitHub time to process authorization
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    while (attempts < maxAttempts) {
      if (attempts > 0) {
        await new Promise(resolve => setTimeout(resolve, currentPollInterval));
      }
      
      try {
        const result = await invoke<any>('poll_github_oauth_token', {
          deviceCode: deviceInfo.device_code
        });
        
        if (result.status === 'success') {
          // Success! Got the token
          editForm.value.secretId = result.secret_id;
          
          // Immediately update the source's secret_id in the database
          if (editingSource.value) {
            try {
              await updateSource(editingSource.value.id, {
                secret_id: result.secret_id
              });
            } catch (e) {
              const errorMsg = e instanceof Error ? e.message : String(e);
              console.error('Failed to update source secret_id:', errorMsg);
              alert(`Warning: Authorization succeeded but failed to link to source: ${errorMsg}`);
            }
          }
          
          const repos = await invoke<any[]>('get_github_repositories', { secretId: result.secret_id });
          editForm.value.availableRepos = repos;
          
          alert('Successfully re-authorized with GitHub!');
          break;
        } else if (result.status === 'pending') {
          // Still waiting for user authorization
          attempts++;
          continue;
        } else if (result.status === 'slow_down') {
          // Update interval and continue polling
          currentPollInterval = result.interval * 1000;
          attempts++;
          continue;
        } else {
          throw new Error(`Unexpected response status: ${result.status}`);
        }
      } catch (e) {
        const errorMsg = e instanceof Error ? e.message : String(e);
        // Check if it's an error response from the backend
        if (errorMsg.includes('expired') || errorMsg.includes('Invalid device code')) {
          throw new Error('Authorization code expired. Please start a new authorization.');
        } else if (errorMsg.includes('cancelled') || errorMsg.includes('access_denied')) {
          throw new Error('Authorization was cancelled. Please try again.');
        } else if (errorMsg.includes('Device flow is not enabled')) {
          throw new Error('Device flow is not enabled for this application.');
        } else {
          throw e;
        }
      }
    }
    
    if (attempts >= maxAttempts) {
      throw new Error('Authorization timed out. Please try again.');
    }
    
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to re-authorize: ${errorMsg}`);
  } finally {
    editForm.value.oauthInProgress = false;
  }
};

// Computed properties for filtered repositories
const filteredRepos = computed(() => {
  if (!githubForm.value.repoSearch) {
    return githubForm.value.availableRepos;
  }
  const search = githubForm.value.repoSearch.toLowerCase();
  return githubForm.value.availableRepos.filter((repo: any) =>
    repo.full_name.toLowerCase().includes(search)
  );
});

const filteredEditRepos = computed(() => {
  if (!editForm.value.repoSearch) {
    return editForm.value.availableRepos;
  }
  const search = editForm.value.repoSearch.toLowerCase();
  return editForm.value.availableRepos.filter((repo: any) =>
    repo.full_name.toLowerCase().includes(search)
  );
});

// Select all / deselect all functions
const selectAllRepos = () => {
  githubForm.value.repositories = filteredRepos.value.map((repo: any) => repo.full_name);
};

const deselectAllRepos = () => {
  githubForm.value.repositories = [];
};

const selectAllEditRepos = () => {
  editForm.value.repositories = filteredEditRepos.value.map((repo: any) => repo.full_name);
};

const deselectAllEditRepos = () => {
  editForm.value.repositories = [];
};

// Repo level data select all/deselect all
const repoLevelDataOptions = ['actions', 'administration', 'checks', 'code_scanning_alerts', 'commits', 'contents', 'discussions', 'issues', 'metadata', 'packages', 'projects', 'prs'];

const selectAllRepoData = () => {
  githubForm.value.endpoints = [...new Set([...githubForm.value.endpoints, ...repoLevelDataOptions])];
};

const deselectAllRepoData = () => {
  githubForm.value.endpoints = githubForm.value.endpoints.filter((ep: string) => !repoLevelDataOptions.includes(ep));
};

const selectAllEditRepoData = () => {
  editForm.value.endpoints = [...new Set([...editForm.value.endpoints, ...repoLevelDataOptions])];
};

const deselectAllEditRepoData = () => {
  editForm.value.endpoints = editForm.value.endpoints.filter((ep: string) => !repoLevelDataOptions.includes(ep));
};

// Handle dropdown close on blur (check if focus is moving to dropdown)
const handleRepoSearchBlur = (e: FocusEvent) => {
  const relatedTarget = e.relatedTarget as HTMLElement;
  // Don't close if focus is moving to the dropdown
  if (relatedTarget && relatedTarget.closest('.repo-multiselect-dropdown')) {
    return;
  }
  // Close after a short delay
  setTimeout(() => {
    githubForm.value.showRepoDropdown = false;
  }, 150);
};

const handleEditRepoSearchBlur = (e: FocusEvent) => {
  const relatedTarget = e.relatedTarget as HTMLElement;
  // Don't close if focus is moving to the dropdown
  if (relatedTarget && relatedTarget.closest('.repo-multiselect-dropdown')) {
    return;
  }
  // Close after a short delay
  setTimeout(() => {
    editForm.value.showRepoDropdown = false;
  }, 150);
};

// Handle closing dropdown after checkbox click (using mousedown to fire before blur)
const handleRepoCheckboxClick = (e: MouseEvent) => {
  // Only close if clicking on the label, not the checkbox itself
  const target = e.target as HTMLElement;
  if (target.tagName === 'INPUT') {
    // If clicking directly on checkbox, close after a short delay
    setTimeout(() => {
      githubForm.value.showRepoDropdown = false;
    }, 100);
  } else {
    // If clicking on label, close immediately
    setTimeout(() => {
      githubForm.value.showRepoDropdown = false;
    }, 50);
  }
};

const handleEditRepoCheckboxClick = (e: MouseEvent) => {
  // Only close if clicking on the label, not the checkbox itself
  const target = e.target as HTMLElement;
  if (target.tagName === 'INPUT') {
    // If clicking directly on checkbox, close after a short delay
    setTimeout(() => {
      editForm.value.showRepoDropdown = false;
    }, 100);
  } else {
    // If clicking on label, close immediately
    setTimeout(() => {
      editForm.value.showRepoDropdown = false;
    }, 50);
  }
};

// Check for existing GitHub secret and load repos
const checkExistingGitHubAuth = async () => {
  try {
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore?.invoke) {
      console.warn('Tauri API not available');
      return;
    }
    const { invoke } = tauriCore;
    const secrets = await invoke<any[]>('get_secrets');
    const githubSecret = secrets.find((s: any) => s.name === 'GitHub Device Flow Token');
    
    if (githubSecret) {
      githubForm.value.secretId = githubSecret.id;
      // Load repos automatically
      try {
        const repos = await invoke<any[]>('get_github_repositories', { secretId: githubSecret.id });
        githubForm.value.availableRepos = repos;
      } catch (e) {
        console.warn('Failed to load repos for existing secret:', e);
        // Secret might be invalid, clear it
        githubForm.value.secretId = null;
      }
    }
  } catch (e) {
    console.warn('Failed to check for existing GitHub auth:', e);
  }
};

const startGitHubOAuth = async () => {
  if (githubForm.value.oauthInProgress) return;
  
  // If already authorized, just fetch repos
  if (githubForm.value.secretId) {
    try {
      const tauriCore = await import('@tauri-apps/api/core');
      if (!tauriCore?.invoke) {
        console.warn('Tauri API not available');
        return;
      }
      const repos = await tauriCore.invoke<any[]>('get_github_repositories', { secretId: githubForm.value.secretId });
      githubForm.value.availableRepos = repos;
      return;
    } catch (e) {
      // If fetching repos fails, clear secretId and re-authorize
      githubForm.value.secretId = null;
    }
  }
  
  githubForm.value.oauthInProgress = true;
  try {
    const tauriCore = await import('@tauri-apps/api/core');
    const openerPlugin = await import('@tauri-apps/plugin-opener');
    if (!tauriCore?.invoke || !openerPlugin?.openUrl) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    const { invoke } = tauriCore;
    const { openUrl } = openerPlugin;
    
    // Start device flow
    const deviceInfo = await invoke<any>('start_github_oauth');
    
    // Show user code and open verification URL
    const userCode = deviceInfo.user_code;
    const verificationUrl = deviceInfo.verification_uri_complete;
    
    // Open the verification URL in browser (it should have the code pre-filled)
    await openUrl(verificationUrl);
    
    // Show custom modal with copy button
    githubAuthCode.value = userCode;
    showGitHubAuthModal.value = true;
    
    // Wait for user to click OK or Cancel
    const proceed = await new Promise<boolean>((resolve) => {
      githubAuthProceed.value = () => {
        showGitHubAuthModal.value = false;
        githubAuthProceed.value = null;
        resolve(true);
      };
      
      // Also resolve false if modal is closed (cancel button or outside click)
      const checkInterval = setInterval(() => {
        if (!showGitHubAuthModal.value) {
          clearInterval(checkInterval);
          if (githubAuthProceed.value) {
            githubAuthProceed.value = null;
            resolve(false);
          }
        }
      }, 100);
    });
    
    if (!proceed) {
      githubForm.value.oauthInProgress = false;
      return;
    }
    
    // Poll for token
    // Calculate max attempts with buffer for potential slow_down interval increases
    // Assume worst case: interval could increase by 5 seconds multiple times
    const baseInterval = deviceInfo.interval;
    const maxAttempts = Math.floor(deviceInfo.expires_in / baseInterval) + 20;
    let attempts = 0;
    let currentPollInterval = baseInterval * 1000; // Convert to milliseconds
    
    // Wait a moment before first poll to give GitHub time to process authorization
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    while (attempts < maxAttempts) {
      if (attempts > 0) {
        await new Promise(resolve => setTimeout(resolve, currentPollInterval));
      }
      
      try {
        const result = await invoke<any>('poll_github_oauth_token', {
          deviceCode: deviceInfo.device_code
        });
        
        if (result.status === 'success') {
          // Success! Got the token
          githubForm.value.secretId = result.secret_id;
          
          // Fetch repositories
          const repos = await invoke<any[]>('get_github_repositories', { secretId: result.secret_id });
          githubForm.value.availableRepos = repos;
          
          alert('Successfully authorized with GitHub!');
          break;
        } else if (result.status === 'pending') {
          // Still waiting for user authorization
          attempts++;
          continue;
        } else if (result.status === 'slow_down') {
          // Update interval and continue polling
          currentPollInterval = result.interval * 1000;
          attempts++;
          continue;
        } else {
          throw new Error(`Unexpected response status: ${result.status}`);
        }
      } catch (e) {
        const errorMsg = e instanceof Error ? e.message : String(e);
        // Check if it's an error response from the backend
        if (errorMsg.includes('expired') || errorMsg.includes('Invalid device code')) {
          throw new Error('Authorization code expired. Please start a new authorization.');
        } else if (errorMsg.includes('cancelled') || errorMsg.includes('access_denied')) {
          throw new Error('Authorization was cancelled. Please try again.');
        } else if (errorMsg.includes('Device flow is not enabled')) {
          throw new Error('Device flow is not enabled for this application.');
        } else {
          throw e; // Real error
        }
      }
    }
    
    if (attempts >= maxAttempts) {
      throw new Error('Authorization timed out. Please try again.');
    }
    
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    // Show user-friendly error messages
    if (errorMsg.includes('temporarily unavailable') || errorMsg.includes('404')) {
      alert(
        'GitHub Authorization Unavailable\n\n' +
        'We were unable to connect to GitHub for authorization.\n\n' +
        'Please try again in a few moments. If the problem persists, ' +
        'check your internet connection or try again later.'
      );
    } else if (errorMsg.includes('Unable to connect') || errorMsg.includes('network')) {
      alert(
        'Connection Error\n\n' +
        'Unable to connect to GitHub. Please check your internet connection and try again.'
      );
    } else {
      alert(`Unable to authorize with GitHub: ${errorMsg}\n\nPlease try again.`);
    }
  } finally {
    githubForm.value.oauthInProgress = false;
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
  if (!githubForm.value.secretId) {
    alert('Please authorize with GitHub first');
    return false;
  }
  if (githubForm.value.repositories.length === 0) {
    alert('Please select at least one repository');
    return false;
  }
  if (githubForm.value.endpoints.length === 0) {
    alert('Please select at least one endpoint');
    return false;
  }
  
  addingSource.value = true;
  try {
    const source: SourceInput = {
      source_type: 'github',
      name: githubForm.value.name,
      config_json: {
        repositories: githubForm.value.repositories,
        endpoints: githubForm.value.endpoints,
        poll_interval: githubForm.value.pollInterval || '10m',
      },
      secret_id: githubForm.value.secretId,
      group_ids: githubForm.value.groupIds.length > 0 ? githubForm.value.groupIds : null,
    };

    await addSource(source);
    await fetchGroups();
    await fetchSourcesAndRebuildMap();
    closeAddSourceModal();
  
    // Reset form
    githubForm.value = { 
      name: '', 
      secretId: null, 
      repositories: [], 
      endpoints: ['commits', 'prs'], 
      pollInterval: '10m',
      groupIds: [],
      oauthInProgress: false,
      availableRepos: [],
      repoSearch: '',
      showRepoDropdown: false,
    };
    return false;
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to add source: ${errorMsg}`);
    return false;
  } finally {
    addingSource.value = false;
  }
};

const editSource = async (source: Source) => {
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
  editForm.value.newGroupsInput = '';
  editForm.value.newGroups = [];
  
  if (source.source_type === 'rss' || source.source_type === 'atom') {
    editForm.value.url = config.url || '';
    editForm.value.pollInterval = config.poll_interval || '10m';
  } else if (source.source_type === 'github') {
    // For GitHub, load repositories and endpoints from config
    editForm.value.repositories = config.repositories || [];
    editForm.value.endpoints = config.endpoints || ['commits', 'prs'];
    editForm.value.pollIntervalGitHub = config.poll_interval || '10m';
    // Load secret_id from backend
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore?.invoke) {
      console.warn('Tauri API not available');
      editForm.value.secretId = null;
      return;
    }
    try {
      const secretId = await tauriCore.invoke<number | null>('get_source_secret_id', { id: source.id });
      editForm.value.secretId = secretId;
      // If we have a secret_id, try to load repos
      if (secretId) {
        try {
          const repos = await tauriCore.invoke<any[]>('get_github_repositories', { secretId });
          editForm.value.availableRepos = repos;
        } catch (e) {
          console.warn('Failed to load repositories:', e);
        }
      }
    } catch (e) {
      console.warn('Failed to load secret_id:', e);
      editForm.value.secretId = null;
    }
    editForm.value.token = ''; // Legacy field, not used
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
    secretId: null,
    repositories: [],
    endpoints: [],
    pollIntervalGitHub: '10m',
    groupIds: [],
    repoSearch: '',
    showRepoDropdown: false,
    newGroupsInput: '',
    newGroups: [],
    enabled: true,
    availableRepos: [],
    oauthInProgress: false,
  };
};

const saveEdit = async (e?: Event) => {
  if (e) {
    e.preventDefault();
    e.stopPropagation();
  }
  
  if (savingSource.value) return false;
  if (!editingSource.value) return false;
  
  if (!editForm.value.name.trim()) {
    alert('Please enter a source name');
    return false;
  }
  
  if ((editingSource.value.source_type === 'rss' || editingSource.value.source_type === 'atom') && !editForm.value.url.trim()) {
    alert('Please enter a URL');
    return false;
  }
  
  if (editingSource.value.source_type === 'github') {
    if (!editForm.value.secretId) {
      alert('Please authorize with GitHub first');
      return false;
    }
    if (editForm.value.repositories.length === 0) {
      alert('Please select at least one repository');
      return false;
    }
    if (editForm.value.endpoints.length === 0) {
      alert('Please select at least one endpoint');
      return false;
    }
  }
  
  savingSource.value = true;
  try {
    // Convert Proxy array to plain array to ensure proper serialization
    let groupIds = Array.isArray(editForm.value.groupIds) 
      ? [...editForm.value.groupIds] 
      : [];
    
    // Create new groups from chips
    for (const groupName of editForm.value.newGroups) {
      // Check if group already exists
      let existingGroup = groups.value.find(g => g.name.toLowerCase() === groupName.toLowerCase());
      
      if (!existingGroup) {
        // Create new group
        const newGroupId = await addGroup(groupName);
        if (newGroupId) {
          await fetchGroups();
          existingGroup = groups.value.find(g => g.id === newGroupId);
          if (existingGroup && !groupIds.includes(existingGroup.id)) {
            groupIds.push(existingGroup.id);
          }
        }
      } else if (!groupIds.includes(existingGroup.id)) {
        // Group exists but not selected, add it
        groupIds.push(existingGroup.id);
      }
    }
    
    const update: UpdateSourceInput = {
      name: editForm.value.name,
      enabled: editForm.value.enabled,
      group_ids: groupIds,
    };
    
    if (editingSource.value.source_type === 'rss' || editingSource.value.source_type === 'atom') {
      update.config_json = {
        url: editForm.value.url,
        poll_interval: editForm.value.pollInterval || '10m',
      };
    } else if (editingSource.value.source_type === 'github') {
      update.config_json = {
        repositories: editForm.value.repositories,
        endpoints: editForm.value.endpoints,
        poll_interval: editForm.value.pollIntervalGitHub || '10m',
      };
      // Update secret_id if it was set (e.g., after re-authorization)
      if (editForm.value.secretId !== null && editForm.value.secretId !== undefined) {
        update.secret_id = editForm.value.secretId;
      }
    }
    
    // Update the source
    await updateSource(editingSource.value.id, update);
    
    // Refresh data
    await fetchGroups();
    await fetchSourcesAndRebuildMap();
    
    // Clear new groups input and chips
    editForm.value.newGroupsInput = '';
    editForm.value.newGroups = [];
    
    // Close panel only after successful update
    closeEditPanel();
    return false;
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to update source: ${errorMsg}`);
    error.value = `Failed to update source: ${errorMsg}`;
    return false;
  } finally {
    savingSource.value = false;
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
    await fetchSourcesAndRebuildMap();
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
    await buildSourceSecretMap();
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
    await buildSourceSecretMap();
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
  
  if (editingGroup.value.id) {
    // Editing existing group - validate name
    if (!groupForm.value.name.trim()) {
      alert('Please enter a group name');
      return false;
    }
  } else {
    // Creating new groups - validate chips
    if (groupForm.value.newGroups.length === 0) {
      alert('Please enter at least one group name');
      return false;
    }
  }
  
  savingGroup.value = true;
  try {
    if (editingGroup.value.id) {
      // Editing existing group - single name only
      const groupId = editingGroup.value.id;
      // Update existing group
      await updateGroup(editingGroup.value.id, groupForm.value.name);
      
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
      // Creating new group(s) - use chips array
      if (groupForm.value.newGroups.length === 0) {
        throw new Error('Please enter at least one group name');
      }
      
      const createdGroupIds: number[] = [];
      
      // Create all groups from chips
      for (const groupName of groupForm.value.newGroups) {
        // Check if group already exists
        let existingGroup = groups.value.find(g => g.name.toLowerCase() === groupName.toLowerCase());
        
        if (!existingGroup) {
          // Create new group
          const newGroupId = await addGroup(groupName);
          if (newGroupId) {
            await fetchGroups();
            existingGroup = groups.value.find(g => g.id === newGroupId);
            if (existingGroup) {
              createdGroupIds.push(existingGroup.id);
            }
          }
        } else {
          // Group already exists, use it
          if (!createdGroupIds.includes(existingGroup.id)) {
            createdGroupIds.push(existingGroup.id);
          }
        }
      }
      
      // Assign sources to all created groups
      if (groupForm.value.sourceIds.length > 0 && createdGroupIds.length > 0) {
        await fetchGroups();
        for (const sourceId of groupForm.value.sourceIds) {
          const source = sources.value.find(s => s.id === sourceId);
          if (source) {
            const existingGroupIds = source.group_ids || [];
            const newGroupIds = [...new Set([...existingGroupIds, ...createdGroupIds])];
            await updateSource(sourceId, { group_ids: newGroupIds });
          }
        }
      }
    }
    
    closeGroupPanel();
    await fetchGroups();
    await fetchSourcesAndRebuildMap();
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
    await removeGroupAction(id);
    await fetchGroups();
    await fetchSourcesAndRebuildMap();
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to remove group: ${errorMsg}`);
    error.value = `Failed to remove group: ${errorMsg}`;
  }
};


// Handle new group input keydown (comma or Enter to add chip)
const handleNewGroupKeydown = (formType: 'edit' | 'group', event: KeyboardEvent) => {
  if (event.key === ',' || event.key === 'Enter') {
    event.preventDefault();
    event.stopPropagation();
    addNewGroupChip(formType);
  }
};

// Handle new group input blur (add chip if there's text)
const handleNewGroupBlur = (formType: 'edit' | 'group') => {
  const form = formType === 'edit' ? editForm.value : groupForm.value;
  if (form.newGroupsInput.trim()) {
    addNewGroupChip(formType);
  }
};

// Add a new group chip from input
const addNewGroupChip = (formType: 'edit' | 'group') => {
  const form = formType === 'edit' ? editForm.value : groupForm.value;
  const trimmed = form.newGroupsInput.trim();
  if (trimmed && !form.newGroups.includes(trimmed)) {
    form.newGroups.push(trimmed);
    form.newGroupsInput = '';
  }
};

// Remove a new group chip
const removeNewGroup = (formType: 'edit' | 'group', index: number) => {
  const form = formType === 'edit' ? editForm.value : groupForm.value;
  form.newGroups.splice(index, 1);
};

// Secrets management functions
const fetchSecrets = async () => {
  secretsLoading.value = true;
  secretsError.value = null;
  try {
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore?.invoke) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    secrets.value = await tauriCore.invoke<any[]>('get_secrets');
  } catch (e) {
    secretsError.value = e instanceof Error ? e.message : String(e);
    console.error('Failed to fetch secrets:', e);
  } finally {
    secretsLoading.value = false;
  }
};

const saveSecret = async () => {
  if (!secretForm.value.name.trim()) {
    alert('Please enter a secret name');
    return;
  }
  
  if (!editingSecret.value && !secretForm.value.value.trim()) {
    alert('Please enter a secret value');
    return;
  }
  
  try {
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore?.invoke) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    const { invoke } = tauriCore;
    
    // If creating a new secret and ttlType is "forever", try to detect expiration from token
    let ttlType = secretForm.value.ttlType;
    let ttlValue = secretForm.value.ttlValue || undefined;
    
    if (!editingSecret.value && ttlType === 'forever' && secretForm.value.value.trim()) {
      // Try to detect expiration from GitHub token
      try {
        const detected = await invoke<{ ttl_type: string; ttl_value?: string } | null>('detect_github_token_expiration', {
          token: secretForm.value.value
        });
        if (detected) {
          ttlType = detected.ttl_type as 'forever' | 'relative' | 'absolute';
          ttlValue = detected.ttl_value;
        }
      } catch (e) {
        // If detection fails, continue with "forever"
        console.warn('Failed to detect token expiration:', e);
      }
    }
    
    if (editingSecret.value) {
      await invoke('update_secret', {
        id: editingSecret.value.id,
        name: secretForm.value.name,
        value: secretForm.value.value || undefined,
        ttl_type: ttlType,
        ttl_value: ttlValue,
      });
    } else {
      await invoke('create_secret', {
        name: secretForm.value.name,
        value: secretForm.value.value,
        ttl_type: ttlType,
        ttl_value: ttlValue,
        refresh_token: null,
      });
    }
    
    await fetchSecrets();
    closeSecretModal();
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to save secret: ${errorMsg}`);
    secretsError.value = errorMsg;
  }
};

const editSecret = (secret: any) => {
  editingSecret.value = secret;
  secretForm.value = {
    name: secret.name,
    value: '', // Don't show existing value
    ttlType: secret.ttl_type as 'forever' | 'relative' | 'absolute',
    ttlValue: secret.ttl_value || '',
  };
};

const removeSecret = async (id: number) => {
  const secret = secrets.value.find(s => s.id === id);
  if (!secret) return;
  
  const sourceCount = getSecretSourceCount(id);
  const confirmed = await ask(
    `Are you sure you want to delete the secret "${secret.name}"?${sourceCount > 0 ? ` This secret is used by ${sourceCount} source${sourceCount !== 1 ? 's' : ''}.` : ''}`,
    {
      title: 'Delete Secret',
      kind: 'warning',
      okLabel: 'Confirm',
      cancelLabel: 'Cancel',
    } as MessageDialogOptions
  );
  
  if (!confirmed) return;
  
  deletingSecrets.value.add(id);
  try {
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore?.invoke) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    await tauriCore.invoke('delete_secret', { id });
    await fetchSecrets();
    secretHealthStatus.value.delete(id);
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    alert(`Failed to delete secret: ${errorMsg}`);
    secretsError.value = errorMsg;
  } finally {
    deletingSecrets.value.delete(id);
  }
};

// Get secret health status
const getSecretHealthStatus = (secret: any): 'good' | 'warning' | 'error' | 'testing' => {
  if (testingSecrets.value.has(secret.id)) {
    return 'testing';
  }
  
  // Check if we have a cached status
  if (secretHealthStatus.value.has(secret.id)) {
    return secretHealthStatus.value.get(secret.id)!;
  }
  
  // Check refresh failure count
  if (secret.refresh_failure_count && secret.refresh_failure_count >= 3) {
    return 'error';
  }
  
  if (secret.refresh_failure_count && secret.refresh_failure_count > 0) {
    return 'warning';
  }
  
  // Check if expired
  if (secret.expires_at && secret.expires_at < Date.now() / 1000) {
    return 'error';
  }
  
  // Check if expiring soon (within 7 days)
  if (secret.expires_at) {
    const daysUntilExpiry = (secret.expires_at - Date.now() / 1000) / (24 * 60 * 60);
    if (daysUntilExpiry > 0 && daysUntilExpiry <= 7) {
      return 'warning';
    }
  }
  
  return 'good';
};

// Get CSS class for secret card based on health
const getSecretHealthClass = (secret: any): string => {
  const status = getSecretHealthStatus(secret);
  if (status === 'error') return 'secret-card-error';
  if (status === 'warning') return 'secret-card-warning';
  return '';
};

// Test a secret token
const testSecretToken = async (secretId: number) => {
  testingSecrets.value.add(secretId);
  secretHealthStatus.value.set(secretId, 'testing');
  
  try {
    const tauriCore = await import('@tauri-apps/api/core');
    if (!tauriCore?.invoke) {
      throw new Error('Tauri API not available. Make sure you are running in a Tauri app.');
    }
    
    const result = await tauriCore.invoke<string>('test_github_token', { secretId });
    secretHealthStatus.value.set(secretId, 'good');
    alert(`✓ ${result}`);
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    secretHealthStatus.value.set(secretId, 'error');
    
    // Check if it's a 401/403 error
    if (errorMsg.includes('401') || errorMsg.includes('expired') || errorMsg.includes('invalid')) {
      alert(`❌ Token Test Failed: ${errorMsg}\n\nThis token appears to be invalid or expired. Please update it in the secret settings.`);
    } else if (errorMsg.includes('403') || errorMsg.includes('Forbidden')) {
      secretHealthStatus.value.set(secretId, 'warning');
      alert(`⚠️ Token Test Warning: ${errorMsg}\n\nThe token may be missing required permissions.`);
    } else {
      alert(`❌ Token Test Failed: ${errorMsg}`);
    }
  } finally {
    testingSecrets.value.delete(secretId);
  }
};

const closeSecretModal = () => {
  showAddSecretModal.value = false;
  editingSecret.value = null;
  secretForm.value = {
    name: '',
    value: '',
    ttlType: 'forever',
    ttlValue: '',
  };
};

const onTtlTypeChange = () => {
  if (secretForm.value.ttlType === 'forever') {
    secretForm.value.ttlValue = '';
  }
};

// Build source -> secret mapping
const buildSourceSecretMap = async () => {
  sourceSecretMap.value.clear();
  const tauriCore = await import('@tauri-apps/api/core');
  if (!tauriCore?.invoke) {
    console.warn('Tauri API not available, skipping secret map build');
    return;
  }
  for (const source of sources.value) {
    try {
      const secretId = await tauriCore.invoke<number | null>('get_source_secret_id', { id: source.id });
      if (secretId !== null) {
        sourceSecretMap.value.set(source.id, secretId);
      }
    } catch (e) {
      // Ignore errors - source might not have a secret
      console.warn(`Failed to get secret_id for source ${source.id}:`, e);
    }
  }
};

// Wrapper to fetch sources and rebuild secret map
const fetchSourcesAndRebuildMap = async () => {
  await fetchSources();
  await buildSourceSecretMap();
};

const handleThemeChange = async (theme: 'system' | 'light' | 'dark' | 'blue' | 'liquid-glass') => {
  try {
    await setTheme(theme);
  } catch (error) {
    console.error('Failed to change theme:', error);
    alert('Failed to change theme. Please try again.');
  }
};

const copyGitHubCode = async () => {
  const code = githubAuthCode.value;
  
  // Try Web Clipboard API first (works in most browsers)
  if (navigator.clipboard && navigator.clipboard.writeText) {
    try {
      await navigator.clipboard.writeText(code);
      alert('Code copied to clipboard!');
      return;
    } catch (e) {
      console.warn('Web clipboard API failed:', e);
    }
  }
  
  // Fallback to Tauri clipboard plugin
  try {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    await writeText(code);
    alert('Code copied to clipboard!');
  } catch (e) {
    console.warn('Tauri clipboard failed:', e);
    
    // Final fallback: select text in a temporary input
    try {
      const input = document.createElement('input');
      input.value = code;
      input.style.position = 'fixed';
      input.style.opacity = '0';
      input.style.pointerEvents = 'none';
      document.body.appendChild(input);
      input.select();
      input.setSelectionRange(0, code.length);
      const success = document.execCommand('copy');
      document.body.removeChild(input);
      
      if (success) {
        alert('Code copied to clipboard!');
      } else {
        throw new Error('execCommand failed');
      }
    } catch (fallbackError) {
      console.error('All copy methods failed:', fallbackError);
      alert('Failed to copy code. Please copy manually: ' + code);
    }
  }
};

const handleGitHubAuthProceed = () => {
  if (githubAuthProceed.value) {
    githubAuthProceed.value();
  }
};

const handleGitHubAuthCancel = () => {
  showGitHubAuthModal.value = false;
  githubAuthProceed.value = null;
  // Cancel the OAuth flow
  githubForm.value.oauthInProgress = false;
  editForm.value.oauthInProgress = false;
};

const handleReauthorizeGitHub = async () => {
  githubForm.value.secretId = null;
  githubForm.value.availableRepos = [];
  await startGitHubOAuth();
};

// Watch for GitHub form being shown and check for existing auth
// Use a flag to ensure component is mounted before running
let isComponentMounted = false;
watch(newSourceType, async (newType) => {
  if (newType === 'github' && isComponentMounted) {
    // Use setTimeout to ensure component is ready
    setTimeout(() => {
      checkExistingGitHubAuth().catch(e => {
        console.warn('Failed to check existing GitHub auth:', e);
      });
    }, 50);
  } else if (newType === 'github_notifications' && isComponentMounted) {
    // Load secrets for dropdown
    setTimeout(async () => {
      try {
        const tauriCore = await import('@tauri-apps/api/core');
        if (tauriCore?.invoke) {
          secrets.value = await tauriCore.invoke<any[]>('get_secrets');
        }
      } catch (e) {
        console.warn('Failed to fetch secrets:', e);
      }
    }, 50);
  }
});

onMounted(async () => {
  await fetchSourcesAndRebuildMap();
  await fetchGroups();
  await fetchSecrets();
  isComponentMounted = true;
  // Check for existing GitHub auth if GitHub form is already selected
  // Use setTimeout to ensure component is fully mounted
  setTimeout(() => {
    if (newSourceType.value === 'github') {
      checkExistingGitHubAuth().catch(e => {
        console.warn('Failed to check existing GitHub auth on mount:', e);
      });
    }
  }, 100);
  
  // Close dropdowns when clicking outside
  const handleClickOutside = (e: MouseEvent) => {
    const target = e.target as HTMLElement;
    if (!target.closest('.repo-multiselect')) {
      githubForm.value.showRepoDropdown = false;
      editForm.value.showRepoDropdown = false;
    }
  };
  
  document.addEventListener('click', handleClickOutside);
  
  // Store handler for cleanup
  (window as any).__repoDropdownClickHandler = handleClickOutside;
});

onUnmounted(() => {
  const handler = (window as any).__repoDropdownClickHandler;
  if (handler) {
    document.removeEventListener('click', handler);
    delete (window as any).__repoDropdownClickHandler;
  }
});

// Expose methods for parent component
defineExpose({
  openAddSourceModal: () => {
    showAddSourceModal.value = true;
  }
});
</script>

<style scoped>
/* Styles moved to src/styles/components/_source-config.scss */
</style>

