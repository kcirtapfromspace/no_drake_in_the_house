/**
 * Settings Page Script for Kiro Extension
 */

class SettingsController {
    constructor() {
        this.settings = {
            serverUrl: 'http://localhost:3000',
            authToken: '',
            autoSkip: true,
            showNotifications: true,
            hideContent: true,
            syncInterval: 5
        };

        this.stats = {
            blockedArtists: 0,
            tracksSkipped: 0,
            contentHidden: 0,
            lastSync: null
        };

        this.init();
    }

    async init() {
        await this.loadSettings();
        await this.loadStats();
        this.updateUI();
        this.setupEventListeners();
    }

    async loadSettings() {
        const result = await chrome.storage.sync.get([
            'serverUrl', 'authToken', 'autoSkip', 'showNotifications',
            'hideContent', 'syncInterval'
        ]);

        this.settings = { ...this.settings, ...result };
    }

    async loadStats() {
        const syncResult = await chrome.storage.sync.get(['dnpList', 'lastSyncTime']);
        const localResult = await chrome.storage.local.get(['actionLog']);

        this.stats.blockedArtists = (syncResult.dnpList || []).length;
        this.stats.lastSync = syncResult.lastSyncTime;

        if (localResult.actionLog) {
            const actionLog = localResult.actionLog;
            this.stats.tracksSkipped = actionLog.filter(entry =>
                entry.action.type === 'track_skipped' || entry.action.type === 'track_auto_skipped'
            ).length;

            this.stats.contentHidden = actionLog.filter(entry =>
                entry.action.type === 'content_hidden'
            ).length;
        }
    }

    updateUI() {
        // Update form fields
        document.getElementById('server-url').value = this.settings.serverUrl;
        document.getElementById('auth-token').value = this.settings.authToken;
        document.getElementById('sync-interval').value = this.settings.syncInterval;

        // Update toggles
        this.updateToggle('auto-skip-toggle', this.settings.autoSkip);
        this.updateToggle('notifications-toggle', this.settings.showNotifications);
        this.updateToggle('hide-content-toggle', this.settings.hideContent);

        // Update stats
        document.getElementById('blocked-artists-count').textContent = this.stats.blockedArtists;
        document.getElementById('tracks-skipped-count').textContent = this.stats.tracksSkipped;
        document.getElementById('content-hidden-count').textContent = this.stats.contentHidden;

        const lastSyncElement = document.getElementById('last-sync-time');
        if (this.stats.lastSync) {
            const date = new Date(this.stats.lastSync);
            lastSyncElement.textContent = date.toLocaleString();
        } else {
            lastSyncElement.textContent = 'Never';
        }
    }

    updateToggle(toggleId, active) {
        const toggle = document.getElementById(toggleId);
        if (active) {
            toggle.classList.add('active');
        } else {
            toggle.classList.remove('active');
        }
    }

    setupEventListeners() {
        // Connect button
        document.getElementById('connect-btn').addEventListener('click', () => {
            this.testConnection();
        });

        // Save token button
        document.getElementById('save-token-btn').addEventListener('click', () => {
            this.saveAuthToken();
        });

        // Toggles
        document.getElementById('auto-skip-toggle').addEventListener('click', () => {
            this.toggleSetting('autoSkip', 'auto-skip-toggle');
        });

        document.getElementById('notifications-toggle').addEventListener('click', () => {
            this.toggleSetting('showNotifications', 'notifications-toggle');
        });

        document.getElementById('hide-content-toggle').addEventListener('click', () => {
            this.toggleSetting('hideContent', 'hide-content-toggle');
        });

        // Sync interval
        document.getElementById('sync-interval').addEventListener('change', (e) => {
            this.updateSetting('syncInterval', parseInt(e.target.value));
        });

        // Server URL
        document.getElementById('server-url').addEventListener('change', (e) => {
            this.updateSetting('serverUrl', e.target.value);
        });

        // Export/Import buttons
        document.getElementById('export-btn').addEventListener('click', () => {
            this.exportData();
        });

        document.getElementById('import-btn').addEventListener('click', () => {
            document.getElementById('import-file').click();
        });

        document.getElementById('import-file').addEventListener('change', (e) => {
            this.importData(e.target.files[0]);
        });

        // Clear data button
        document.getElementById('clear-data-btn').addEventListener('click', () => {
            this.clearAllData();
        });
    }

    async testConnection() {
        const serverUrl = document.getElementById('server-url').value;
        const connectBtn = document.getElementById('connect-btn');

        connectBtn.textContent = 'Testing...';
        connectBtn.disabled = true;

        try {
            const response = await fetch(`${serverUrl}/api/health`);
            if (response.ok) {
                this.showStatus('Connection successful!', 'success');
                await this.updateSetting('serverUrl', serverUrl);
            } else {
                this.showStatus('Server responded but may not be healthy', 'error');
            }
        } catch (error) {
            this.showStatus('Connection failed. Check server URL.', 'error');
        } finally {
            connectBtn.textContent = 'Connect';
            connectBtn.disabled = false;
        }
    }

    async saveAuthToken() {
        const token = document.getElementById('auth-token').value;
        const saveBtn = document.getElementById('save-token-btn');

        if (!token.trim()) {
            this.showStatus('Please enter a valid token', 'error');
            return;
        }

        saveBtn.textContent = 'Saving...';
        saveBtn.disabled = true;

        try {
            await this.updateSetting('authToken', token);
            this.showStatus('Token saved successfully!', 'success');

            // Test the token by trying to sync
            chrome.runtime.sendMessage({
                type: 'SYNC_WITH_SERVER',
                authToken: token
            });

        } catch (error) {
            this.showStatus('Failed to save token', 'error');
        } finally {
            saveBtn.textContent = 'Save';
            saveBtn.disabled = false;
        }
    }

    async toggleSetting(settingKey, toggleId) {
        const currentValue = this.settings[settingKey];
        const newValue = !currentValue;

        await this.updateSetting(settingKey, newValue);
        this.updateToggle(toggleId, newValue);
    }

    async updateSetting(key, value) {
        this.settings[key] = value;
        await chrome.storage.sync.set({ [key]: value });
    }

    async exportData() {
        try {
            const result = await chrome.storage.sync.get(['dnpList']);
            const data = {
                version: '1.0',
                exportDate: new Date().toISOString(),
                dnpList: result.dnpList || [],
                settings: this.settings
            };

            const blob = new Blob([JSON.stringify(data, null, 2)], {
                type: 'application/json'
            });

            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `kiro-blocklist-${new Date().toISOString().split('T')[0]}.json`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);

            this.showStatus('Data exported successfully!', 'success');
        } catch (error) {
            this.showStatus('Export failed', 'error');
        }
    }

    async importData(file) {
        if (!file) return;

        try {
            const text = await file.text();
            const data = JSON.parse(text);

            if (!data.dnpList || !Array.isArray(data.dnpList)) {
                throw new Error('Invalid file format');
            }

            // Confirm import
            const confirmed = confirm(
                `Import ${data.dnpList.length} blocked artists? This will replace your current list.`
            );

            if (confirmed) {
                await chrome.storage.sync.set({ dnpList: data.dnpList });

                // Import settings if available
                if (data.settings) {
                    const settingsToImport = {
                        autoSkip: data.settings.autoSkip,
                        showNotifications: data.settings.showNotifications,
                        hideContent: data.settings.hideContent,
                        syncInterval: data.settings.syncInterval
                    };

                    await chrome.storage.sync.set(settingsToImport);
                    this.settings = { ...this.settings, ...settingsToImport };
                }

                await this.loadStats();
                this.updateUI();
                this.showStatus(`Imported ${data.dnpList.length} blocked artists!`, 'success');
            }
        } catch (error) {
            this.showStatus('Import failed. Please check file format.', 'error');
        }
    }

    async clearAllData() {
        const confirmed = confirm(
            'Are you sure you want to clear all data? This will remove all blocked artists and reset settings.'
        );

        if (confirmed) {
            try {
                // Clear sync storage
                await chrome.storage.sync.clear();

                // Clear local storage
                await chrome.storage.local.clear();

                // Reset to defaults
                this.settings = {
                    serverUrl: 'http://localhost:3000',
                    authToken: '',
                    autoSkip: true,
                    showNotifications: true,
                    hideContent: true,
                    syncInterval: 5
                };

                this.stats = {
                    blockedArtists: 0,
                    tracksSkipped: 0,
                    contentHidden: 0,
                    lastSync: null
                };

                this.updateUI();
                this.showStatus('All data cleared successfully!', 'success');
            } catch (error) {
                this.showStatus('Failed to clear data', 'error');
            }
        }
    }

    showStatus(message, type) {
        const statusElement = document.getElementById('status-message');
        statusElement.textContent = message;
        statusElement.className = `status ${type}`;
        statusElement.classList.remove('hidden');

        setTimeout(() => {
            statusElement.classList.add('hidden');
        }, 5000);
    }
}

// Initialize settings when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new SettingsController();
});