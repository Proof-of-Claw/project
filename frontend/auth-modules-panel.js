/**
 * Shared auth modules markup for dashboard embed and auth-modules.html.
 * Depends on auth-modules-panel.css (injected on first mount).
 */
(function (global) {
  'use strict';

  var PANEL_HTML =
    '<header class="page-intro">' +
    '<h1>Authentication modules</h1>' +
    '<p>Reference layout for user and agent sign-in surfaces: OAuth providers, chat platforms, and machine-first credentials. ' +
    'Vault-style APIs (for example the <a href="https://github.com/1clawAI/1claw-openapi-spec" rel="noopener noreferrer">1Claw OpenAPI spec</a>) ' +
    'document JWT exchange, Google OAuth, API keys, and agent auth methods—map each module below to the flows your backend exposes.</p>' +
    '</header>' +
    '<div class="module-grid">' +
    '<article class="auth-card" style="--accent:#f0f6fc;">' +
    '<div class="auth-card-header">' +
    '<div class="auth-icon" aria-hidden="true">' +
    '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/></svg>' +
    '</div><div><h2>GitHub</h2><p class="tagline">OAuth App · GitHub App</p></div></div>' +
    '<p class="desc">Delegate identity to GitHub accounts. Use for developer tooling, repo-scoped agents, and org membership checks.</p>' +
    '<div class="chips"><span class="chip">OAuth2</span><span class="chip">Device flow</span><span class="chip">Fine-grained tokens</span></div>' +
    '<ul class="flow-list">' +
    '<li>Redirect to GitHub authorize URL with <code>client_id</code> and scopes</li>' +
    '<li>Exchange <code>code</code> at token endpoint; store refresh token if offline access</li>' +
    '<li>Map <code>id</code> / login to your user record; issue your own session or JWT</li>' +
    '</ul></article>' +
    '<article class="auth-card" style="--accent:#4285f4;">' +
    '<div class="auth-card-header">' +
    '<div class="auth-icon" aria-hidden="true">' +
    '<svg viewBox="0 0 24 24" width="26" height="26">' +
    '<path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>' +
    '<path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>' +
    '<path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>' +
    '<path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>' +
    '</svg></div><div><h2>Google</h2><p class="tagline">OpenID Connect</p></div></div>' +
    '<p class="desc">Sign in with Google for consumer-friendly onboarding. OIDC <code>id_token</code> carries verified email and subject (<code>sub</code>).</p>' +
    '<div class="chips"><span class="chip">OIDC</span><span class="chip">email_verified</span><span class="chip">refresh token</span></div>' +
    '<ul class="flow-list">' +
    '<li>Use Google’s discovery document for JWKS and token endpoint URLs</li>' +
    '<li>Validate <code>id_token</code> audience, issuer, and expiry server-side</li>' +
    '<li>Aligns with “Google OAuth” style flows in vault APIs such as 1Claw</li>' +
    '</ul></article>' +
    '<article class="auth-card" style="--accent:#5865f2;">' +
    '<div class="auth-card-header">' +
    '<div class="auth-icon" aria-hidden="true">' +
    '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/></svg>' +
    '</div><div><h2>Discord</h2><p class="tagline">OAuth2 · Bot</p></div></div>' +
    '<p class="desc">Guild-aware bots and “Login with Discord” for communities. Combine user OAuth with bot tokens for channel actions.</p>' +
    '<div class="chips"><span class="chip">guilds</span><span class="chip">bot scope</span><span class="chip">webhook</span></div>' +
    '<ul class="flow-list">' +
    '<li>Register application; set redirect URIs for your web or deep link</li>' +
    '<li>Request <code>identify</code> / <code>guilds</code> as needed; add bot install URL separately</li>' +
    '<li>Store Discord user id; respect rate limits on the REST API</li>' +
    '</ul></article>' +
    '<article class="auth-card" style="--accent:#2aabee;">' +
    '<div class="auth-card-header">' +
    '<div class="auth-icon" aria-hidden="true">' +
    '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z"/></svg>' +
    '</div><div><h2>Telegram</h2><p class="tagline">Bot API · Login Widget</p></div></div>' +
    '<p class="desc">Bots for messaging and the Login Widget for lightweight web auth without a full OAuth server in some setups.</p>' +
    '<div class="chips"><span class="chip">Bot token</span><span class="chip">HMAC verify</span><span class="chip">Mini Apps</span></div>' +
    '<ul class="flow-list">' +
    '<li>Create bot via BotFather; keep token server-side only</li>' +
    '<li>For Login Widget: verify hash of auth payload with bot token</li>' +
    '<li>Use deep links (<code>t.me/</code>) to start conversations from your site</li>' +
    '</ul></article>' +
    '<article class="auth-card" style="--accent:#e01e5a;">' +
    '<div class="auth-card-header">' +
    '<div class="auth-icon" aria-hidden="true">' +
    '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M5.042 15.165a2.528 2.528 0 0 1-2.52 2.523A2.528 2.528 0 0 1 0 15.165a2.527 2.527 0 0 1 2.522-2.52h2.52v2.52zM6.313 15.165a2.527 2.527 0 0 1 2.521-2.52 2.527 2.527 0 0 1 2.521 2.52v6.313A2.528 2.528 0 0 1 8.834 24a2.528 2.528 0 0 1-2.521-2.522v-6.313zM8.834 5.042a2.528 2.528 0 0 1-2.521-2.52A2.528 2.528 0 0 1 8.834 0a2.528 2.528 0 0 1 2.521 2.522v2.52H8.834V5.042zm0 1.27a2.528 2.528 0 0 1 2.521 2.521 2.528 2.528 0 0 1-2.521 2.521H2.522A2.528 2.528 0 0 1 0 8.834a2.528 2.528 0 0 1 2.522-2.521h6.312zm11.126 2.521a2.528 2.528 0 0 1 2.522-2.521A2.528 2.528 0 0 1 24 8.834a2.528 2.528 0 0 1-2.522 2.521h-2.522V8.834zm-1.27 0a2.528 2.528 0 0 1-2.523 2.521 2.527 2.527 0 0 1-2.52-2.521V2.522A2.527 2.527 0 0 1 15.165 0a2.528 2.528 0 0 1 2.523 2.522v6.312zm-2.523 11.126a2.527 2.527 0 0 1 2.52 2.522A2.527 2.527 0 0 1 15.165 24a2.528 2.528 0 0 1-2.52-2.522v-2.522h2.52zm0-1.27a2.527 2.527 0 0 1-2.52-2.523 2.526 2.526 0 0 1 2.52-2.52h6.313A2.528 2.528 0 0 1 24 15.165a2.528 2.528 0 0 1-2.522 2.523h-6.313z"/></svg>' +
    '</div><div><h2>Slack</h2><p class="tagline">OAuth v2 · workspace</p></div></div>' +
    '<p class="desc">Install apps into customer workspaces; user tokens vs bot tokens determine whether you act as a user or the app.</p>' +
    '<div class="chips"><span class="chip">user_scope</span><span class="chip">bot token</span><span class="chip">signing secret</span></div>' +
    '<ul class="flow-list">' +
    '<li>Start OAuth with client id, scopes, and redirect URI registered in Slack app</li>' +
    '<li>Exchange code for access token; store team id and token type</li>' +
    '<li>Verify incoming requests with signing secret for Events API</li>' +
    '</ul></article>' +
    '<article class="auth-card" style="--accent:#ffb300;">' +
    '<div class="auth-card-header">' +
    '<div class="auth-icon" aria-hidden="true">' +
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">' +
    '<rect x="3" y="11" width="18" height="11" rx="2"/>' +
    '<path d="M7 11V7a5 5 0 0 1 10 0v4"/>' +
    '<circle cx="12" cy="16" r="1.25" fill="currentColor" stroke="none"/>' +
    '</svg></div><div><h2>Custom</h2><p class="tagline">API key · mTLS · OIDC</p></div></div>' +
    '<p class="desc">First-party credentials and enterprise patterns: static API keys, mutual TLS, or OIDC client credentials for service-style agents.</p>' +
    '<div class="chips"><span class="chip">api_key</span><span class="chip">mtls</span><span class="chip">oidc_client_credentials</span></div>' +
    '<ul class="flow-list">' +
    '<li>Issue scoped secrets; rotate and audit like vault “agent” auth methods</li>' +
    '<li>mTLS: terminate at gateway; forward client cert fingerprint to policy layer</li>' +
    '<li>OIDC client credentials: machine token → your API JWT (e.g. agent-token exchange)</li>' +
    '</ul></article>' +
    '</div>' +
    '<aside class="spec-callout">' +
    '<strong>API alignment.</strong> When you wire a backend to <a href="https://github.com/1clawAI/1claw-openapi-spec">@1claw/openapi-spec</a>, ' +
    'reuse the same vocabulary: user-facing OAuth where documented, plus agent <code>auth_method</code> values (API key, mTLS, OIDC) for non-interactive clients.' +
    '</aside>';

  function ensureStylesheet() {
    if (document.getElementById('poc-auth-modules-stylesheet')) return;
    var link = document.createElement('link');
    link.id = 'poc-auth-modules-stylesheet';
    link.rel = 'stylesheet';
    link.href = 'auth-modules-panel.css';
    document.head.appendChild(link);
  }

  global.PocAuthModules = {
    mount: function (container) {
      if (!container) return;
      ensureStylesheet();
      container.classList.add('poc-auth-panel');
      container.innerHTML = PANEL_HTML;
    }
  };
})(typeof window !== 'undefined' ? window : globalThis);
