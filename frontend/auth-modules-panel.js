/**
 * Proof of Claw — Auth Modules Panel
 * OAuth and agent auth modules: GitHub, Google, Discord, Telegram, Slack, and custom credentials.
 */

'use strict';

const PocAuthModules = (function() {
  // --- State ---
  let rootEl = null;
  let config = {
    onConnect: null,
    onDisconnect: null,
    compact: false
  };

  // --- Auth Module Definitions ---
  const AUTH_MODULES = [
    {
      id: 'github',
      name: 'GitHub',
      description: 'Access repositories, issues, and pull requests',
      icon: '<svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>',
      color: '#24292e',
      scopes: ['repo', 'read:user', 'read:org'],
      connected: false
    },
    {
      id: 'google',
      name: 'Google',
      description: 'Gmail, Calendar, and Drive access',
      icon: '<svg viewBox="0 0 24 24" width="24" height="24"><path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/><path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/><path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/><path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/></svg>',
      scopes: ['email', 'profile', 'https://www.googleapis.com/auth/gmail.readonly'],
      connected: false
    },
    {
      id: 'discord',
      name: 'Discord',
      description: 'Send messages and read channels',
      icon: '<svg viewBox="0 0 24 24" width="24" height="24" fill="#5865F2"><path d="M20.317 4.37a19.791 19.791 0 00-4.885-1.515.074.074 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 00-5.487 0 12.64 12.64 0 00-.617-1.25.077.077 0 00-.079-.037A19.736 19.736 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 00.031.057 19.9 19.9 0 005.993 3.03.078.078 0 00.084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 00-.041-.106 13.107 13.107 0 01-1.872-.892.077.077 0 01-.008-.128 10.2 10.2 0 00.372-.292.074.074 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 01.078.01c.12.098.246.198.373.292a.077.077 0 01-.006.127 12.299 12.299 0 01-1.873.892.077.077 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 00.084.028 19.839 19.839 0 006.002-3.03.077.077 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 00-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/></svg>',
      scopes: ['identify', 'email', 'bot'],
      connected: false
    },
    {
      id: 'telegram',
      name: 'Telegram',
      description: 'Bot API for messages and notifications',
      icon: '<svg viewBox="0 0 24 24" width="24" height="24" fill="#0088cc"><path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z"/></svg>',
      scopes: ['bot'],
      connected: false
    },
    {
      id: 'slack',
      name: 'Slack',
      description: 'Post messages and read channels',
      icon: '<svg viewBox="0 0 24 24" width="24" height="24"><path fill="#E01E5A" d="M5.042 15.165a2.528 2.528 0 0 1-2.52 2.523A2.528 2.528 0 0 1 0 15.165a2.527 2.527 0 0 1 2.522-2.52h2.52v2.52zM6.313 15.165a2.527 2.527 0 0 1 2.521-2.52 2.527 2.527 0 0 1 2.521 2.52v6.313A2.528 2.528 0 0 1 8.834 24a2.528 2.528 0 0 1-2.521-2.522v-6.313zM8.834 5.042a2.528 2.528 0 0 1-2.521-2.52A2.528 2.528 0 0 1 8.834 0a2.528 2.528 0 0 1 2.521 2.522v2.52H8.834zM8.834 6.313a2.528 2.528 0 0 1 2.521 2.521 2.528 2.528 0 0 1-2.521 2.521H2.522A2.528 2.528 0 0 1 0 8.834a2.528 2.528 0 0 1 2.522-2.521h6.312zM18.956 8.834a2.528 2.528 0 0 1 2.522-2.521A2.528 2.528 0 0 1 24 8.834a2.528 2.528 0 0 1-2.522 2.521h-2.522V8.834zM17.688 8.834a2.528 2.528 0 0 1-2.523 2.521 2.527 2.527 0 0 1-2.52-2.521V2.522A2.527 2.527 0 0 1 15.165 0a2.528 2.528 0 0 1 2.523 2.522v6.312zM15.165 18.956a2.528 2.528 0 0 1 2.523 2.522A2.528 2.528 0 0 1 15.165 24a2.527 2.527 0 0 1-2.52-2.522v-2.522h2.52zM15.165 17.688a2.527 2.527 0 0 1-2.52-2.523 2.526 2.526 0 0 1 2.52-2.52h6.313A2.527 2.527 0 0 1 24 15.165a2.528 2.528 0 0 1-2.522 2.523h-6.313z"/></svg>',
      scopes: ['chat:write', 'channels:read', 'users:read'],
      connected: false
    }
  ];

  // --- Custom Credentials ---
  const CUSTOM_CREDENTIAL_TYPES = [
    { id: 'api_key', name: 'API Key', placeholder: 'Enter API key' },
    { id: 'bearer_token', name: 'Bearer Token', placeholder: 'Enter bearer token' },
    { id: 'oauth_token', name: 'OAuth Token', placeholder: 'Enter OAuth token' },
    { id: 'webhook_url', name: 'Webhook URL', placeholder: 'https://...' }
  ];

  // --- Storage Helpers ---
  function getStoredConnections() {
    try {
      const stored = localStorage.getItem('poc_auth_connections');
      return stored ? JSON.parse(stored) : {};
    } catch (e) {
      return {};
    }
  }

  function setStoredConnections(connections) {
    try {
      localStorage.setItem('poc_auth_connections', JSON.stringify(connections));
    } catch (e) {}
  }

  function getStoredCustomCreds() {
    try {
      const stored = localStorage.getItem('poc_custom_credentials');
      return stored ? JSON.parse(stored) : [];
    } catch (e) {
      return [];
    }
  }

  function setStoredCustomCreds(creds) {
    try {
      localStorage.setItem('poc_custom_credentials', JSON.stringify(creds));
    } catch (e) {}
  }

  // --- UI Rendering ---
  function render() {
    if (!rootEl) return;

    const connections = getStoredConnections();
    const customCreds = getStoredCustomCreds();

    const html = `
      <div class="poc-auth-modules">
        <div class="auth-header">
          <h2 class="auth-title">Authentication Modules</h2>
          <p class="auth-subtitle">Connect OAuth providers and manage API credentials for your agents</p>
        </div>

        <div class="auth-section">
          <h3 class="section-title">OAuth Providers</h3>
          <div class="auth-grid">
            ${AUTH_MODULES.map(mod => renderModuleCard(mod, connections[mod.id])).join('')}
          </div>
        </div>

        <div class="auth-section">
          <div class="section-header">
            <h3 class="section-title">Custom Credentials</h3>
            <button class="btn-add-cred" onclick="PocAuthModules.showAddCredModal()">+ Add Credential</button>
          </div>
          <div class="custom-creds-list">
            ${customCreds.length === 0 ? 
              '<div class="empty-creds">No custom credentials added yet</div>' :
              customCreds.map((cred, idx) => renderCustomCred(cred, idx)).join('')
            }
          </div>
        </div>

        <div class="auth-tips">
          <h4 class="tips-title">Security Tips</h4>
          <ul class="tips-list">
            <li>Credentials are stored locally in your browser</li>
            <li>Agents can only access the scopes you authorize</li>
            <li>Revoke access anytime from provider settings</li>
            <li>Use separate bot accounts when possible</li>
          </ul>
        </div>
      </div>

      ${renderModal()}
    `;

    rootEl.innerHTML = html;
  }

  function renderModuleCard(mod, connection) {
    const isConnected = connection && connection.connected;
    const btnClass = isConnected ? 'btn-connected' : 'btn-connect';
    const btnText = isConnected ? 'Connected' : 'Connect';
    const cardClass = isConnected ? 'auth-card connected' : 'auth-card';

    return `
      <div class="${cardClass}" data-module="${mod.id}">
        <div class="auth-card-header">
          <div class="auth-icon">${mod.icon}</div>
          <div class="auth-info">
            <h4 class="auth-name">${esc(mod.name)}</h4>
            <p class="auth-desc">${esc(mod.description)}</p>
          </div>
        </div>
        <div class="auth-card-footer">
          <span class="auth-scopes">${mod.scopes.length} scopes</span>
          <button class="${btnClass}" onclick="PocAuthModules.toggleConnection('${mod.id}')">
            ${isConnected ? '✓ ' : ''}${btnText}
          </button>
        </div>
        ${isConnected ? `<div class="connection-badge">● Active</div>` : ''}
      </div>
    `;
  }

  function renderCustomCred(cred, idx) {
    return `
      <div class="custom-cred-item" data-idx="${idx}">
        <div class="cred-info">
          <span class="cred-type">${esc(cred.typeName)}</span>
          <span class="cred-name">${esc(cred.name)}</span>
          <span class="cred-preview">${esc(cred.value.substring(0, 20))}${cred.value.length > 20 ? '...' : ''}</span>
        </div>
        <div class="cred-actions">
          <button class="btn-cred-test" onclick="PocAuthModules.testCred(${idx})">Test</button>
          <button class="btn-cred-delete" onclick="PocAuthModules.deleteCred(${idx})">Delete</button>
        </div>
      </div>
    `;
  }

  function renderModal() {
    return `
      <div class="auth-modal" id="auth-modal" style="display:none;">
        <div class="modal-overlay" onclick="PocAuthModules.closeModal()"></div>
        <div class="modal-content">
          <div class="modal-header">
            <h3>Add Custom Credential</h3>
            <button class="modal-close" onclick="PocAuthModules.closeModal()">&times;</button>
          </div>
          <div class="modal-body">
            <div class="form-group">
              <label>Credential Type</label>
              <select id="cred-type" class="form-select" onchange="PocAuthModules.updateCredPlaceholder()">
                ${CUSTOM_CREDENTIAL_TYPES.map(t => `<option value="${t.id}">${esc(t.name)}</option>`).join('')}
              </select>
            </div>
            <div class="form-group">
              <label>Name (for reference)</label>
              <input type="text" id="cred-name" class="form-input" placeholder="e.g., OpenAI API Key">
            </div>
            <div class="form-group">
              <label id="cred-value-label">API Key</label>
              <input type="password" id="cred-value" class="form-input" placeholder="Enter credential value">
            </div>
          </div>
          <div class="modal-footer">
            <button class="btn-modal-secondary" onclick="PocAuthModules.closeModal()">Cancel</button>
            <button class="btn-modal-primary" onclick="PocAuthModules.saveCred()">Save Credential</button>
          </div>
        </div>
      </div>
    `;
  }

  function esc(str) {
    if (str == null) return '';
    return String(str).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  }

  // --- Actions ---
  function toggleConnection(moduleId) {
    const connections = getStoredConnections();
    const isConnected = connections[moduleId] && connections[moduleId].connected;

    if (isConnected) {
      // Disconnect
      connections[moduleId] = { connected: false, timestamp: null };
      setStoredConnections(connections);
      if (config.onDisconnect) config.onDisconnect(moduleId);
    } else {
      // Simulate OAuth flow
      simulateOAuth(moduleId).then(result => {
        if (result.success) {
          connections[moduleId] = {
            connected: true,
            timestamp: Date.now(),
            token: 'mock_token_' + Math.random().toString(36).substr(2)
          };
          setStoredConnections(connections);
          if (config.onConnect) config.onConnect(moduleId);
        }
      });
    }

    render();
  }

  function simulateOAuth(moduleId) {
    return new Promise(resolve => {
      const mod = AUTH_MODULES.find(m => m.id === moduleId);
      showToast(`Connecting to ${mod.name}...`);
      setTimeout(() => {
        showToast(`Connected to ${mod.name}!`, 'success');
        resolve({ success: true });
      }, 1500);
    });
  }

  function showAddCredModal() {
    const modal = document.getElementById('auth-modal');
    if (modal) {
      modal.style.display = 'block';
      document.getElementById('cred-name').value = '';
      document.getElementById('cred-value').value = '';
    }
  }

  function closeModal() {
    const modal = document.getElementById('auth-modal');
    if (modal) modal.style.display = 'none';
  }

  function updateCredPlaceholder() {
    const typeId = document.getElementById('cred-type').value;
    const type = CUSTOM_CREDENTIAL_TYPES.find(t => t.id === typeId);
    if (type) {
      document.getElementById('cred-value').placeholder = type.placeholder;
      document.getElementById('cred-value-label').textContent = type.name;
    }
  }

  function saveCred() {
    const typeId = document.getElementById('cred-type').value;
    const name = document.getElementById('cred-name').value.trim();
    const value = document.getElementById('cred-value').value.trim();

    if (!name || !value) {
      showToast('Please fill in all fields', 'error');
      return;
    }

    const type = CUSTOM_CREDENTIAL_TYPES.find(t => t.id === typeId);
    const creds = getStoredCustomCreds();
    creds.push({
      typeId,
      typeName: type.name,
      name,
      value,
      created: Date.now()
    });
    setStoredCustomCreds(creds);

    closeModal();
    render();
    showToast('Credential saved', 'success');
  }

  function deleteCred(idx) {
    if (!confirm('Delete this credential?')) return;
    const creds = getStoredCustomCreds();
    creds.splice(idx, 1);
    setStoredCustomCreds(creds);
    render();
    showToast('Credential deleted', 'success');
  }

  function testCred(idx) {
    const creds = getStoredCustomCreds();
    const cred = creds[idx];
    showToast(`Testing ${cred.name}...`);
    setTimeout(() => {
      showToast('Credential is valid', 'success');
    }, 1000);
  }

  function showToast(message, type = 'info') {
    // Simple toast notification
    const toast = document.createElement('div');
    toast.className = `auth-toast ${type}`;
    toast.textContent = message;
    toast.style.cssText = `
      position: fixed;
      bottom: 20px;
      right: 20px;
      padding: 12px 20px;
      background: ${type === 'success' ? 'rgba(0,230,118,0.9)' : type === 'error' ? 'rgba(255,61,90,0.9)' : 'rgba(0,229,255,0.9)'};
      color: #000;
      border-radius: 8px;
      font-weight: 600;
      z-index: 10000;
      animation: slideIn 0.3s ease;
    `;
    document.body.appendChild(toast);
    setTimeout(() => {
      toast.style.animation = 'slideOut 0.3s ease';
      setTimeout(() => toast.remove(), 300);
    }, 3000);
  }

  // --- Public API ---
  return {
    mount: function(element, options = {}) {
      rootEl = typeof element === 'string' ? document.getElementById(element) : element;
      config = { ...config, ...options };
      render();
    },

    toggleConnection,
    showAddCredModal,
    closeModal,
    updateCredPlaceholder,
    saveCred,
    deleteCred,
    testCred,

    getConnections: function() {
      return getStoredConnections();
    },

    getCustomCredentials: function() {
      return getStoredCustomCreds();
    },

    isConnected: function(moduleId) {
      const conns = getStoredConnections();
      return conns[moduleId] && conns[moduleId].connected;
    }
  };
})();

// Global access
window.PocAuthModules = PocAuthModules;
