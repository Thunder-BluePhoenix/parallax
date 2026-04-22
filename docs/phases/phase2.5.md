# Phase 2.5 — Git Collaboration & Team Chat

**Status:** 🔲 Planned
**Depends on:** Phase 2 complete
**Goal:** Make Parallax the first git-native, GitHub-integrated API workspace. Every workspace is a git repo. Teams are GitHub collaborators. API docs publish to GitHub Pages. Real-time chat runs through the Go sidecar — local-first, no Parallax cloud ever required.

---

## Objectives

1. Turn every Parallax workspace into a **git repository** — commit, push, pull, stash, branch from within the app.
2. Add **GitHub OAuth** as the universal identity layer — one login, used for git authorship, team management, and chat identity.
3. Let users **publish API documentation** to GitHub Pages from any collection in one click.
4. Enable **team workspaces** — invite teammates by GitHub username; each workspace has its own independent team backed by GitHub repo collaborators.
5. Add **real-time team chat** powered by the Go sidecar — P2P on the same network, git-relay fallback for remote teams, fully opt-in per user.

---

## Deliverables

### 1. Git-Native Workspace (`git2` Rust crate)

Every `.parallax/` directory is initialized as a git repository on workspace creation. All collections, environments, history, and chat logs are plain files — already git-ready from Phase 1.

**In-app git operations (Rust commands via `git2`):**

```
commit   → stage all .parallax/ changes + write commit message
push     → push current branch to configured remote
pull     → fetch + fast-forward or merge; surface conflicts in UI
stash    → stash uncommitted changes
stash pop → restore stashed changes
branch   → create / switch / delete branches
log      → list commits with author, timestamp, message
diff     → show uncommitted file diffs (feeds Environment diff view)
```

**UI surfaces:**
- **Sidebar git status badge** — count of uncommitted changes next to workspace name
- **Branch chip** (already in sidebar ✅) — extended to allow create/switch/delete
- **Commit panel** — triggered from sidebar or keyboard shortcut; shows staged files + message input
- **Conflict resolver** — on pull conflicts, shows a side-by-side diff for each conflicting file; user accepts left/right/both
- **Commit history panel** — scrollable log of past commits with author avatar, message, timestamp; click to inspect diff

**Workspace config (`.parallax/workspace.yaml`):**
```yaml
name: My API Workspace
remote: https://github.com/username/my-api-workspace.git
visibility: private   # or public
git_author_name: ""   # filled from GitHub identity on login
git_author_email: ""  # filled from GitHub identity on login
```

### 2. GitHub OAuth Identity

A single GitHub login gives Parallax a verified user identity used everywhere in Phase 2.5 — no separate Parallax account ever.

**Flow:**
1. User clicks "Sign in with GitHub" in settings or first-run prompt
2. Tauri opens the system browser to GitHub OAuth2 PKCE authorization URL
3. GitHub redirects to `parallax://oauth/github` (custom URL scheme registered in Tauri)
4. Tauri intercepts the redirect, extracts the auth code, exchanges for access token
5. Token + `{ login, id, avatar_url, email }` stored in OS keychain via `tauri-plugin-keychain`

**Identity used for:**
- Git commit author name + email
- `presence.json` peer entries (GitHub login as ID)
- Chat message sender identity
- Team invite / collaborator API calls
- Titlebar avatar display

**UI:**
- Titlebar shows GitHub avatar + `@username` when signed in
- Settings → Account: shows connected account, sign out, revoke token button

### 3. Publish API Docs to GitHub Pages

One-click: generate static HTML documentation from a collection and push it live to GitHub Pages.

**Generation:**
- Walks collection tree (folders → requests)
- For each request: method badge, URL, description, params table, headers table, auth info, body example, sample responses
- Renders to a single-page HTML site (or multi-page with sidebar nav)
- Includes collection name, description, last-updated timestamp
- Output committed to `gh-pages` branch of the workspace repo

**Publish flow:**
```
Collection → Doc Generator → HTML/CSS bundle
    → git checkout gh-pages (or orphan branch)
    → write files
    → git commit "Update API docs — {timestamp}"
    → git push origin gh-pages
    → GitHub Pages serves at https://{user}.github.io/{repo}/
```

**Settings per workspace:**
- Public or private repo toggle (private = GitHub Pages requires GitHub Pro/Teams — surfaced as a warning)
- Custom doc site title + tagline
- Include/exclude specific collections or folders
- "View live docs" button — opens `https://{user}.github.io/{repo}/` in browser

### 4. Team Workspaces

Teams are GitHub repo collaborators. No separate Parallax user database — GitHub enforces access.

**Invite flow:**
1. Workspace owner types a GitHub username in "Invite teammate" field
2. Parallax calls `PUT /repos/{owner}/{repo}/collaborators/{username}` via GitHub API
3. Invitee gets a GitHub email notification with accept link
4. Once accepted, their name appears in the workspace team panel with their avatar

**Team panel (in workspace sidebar):**
- List of collaborators with GitHub avatar, username, role (Admin / Write / Read)
- Online indicator (green dot) — from chat `presence.json`; only visible when chat is enabled
- "Remove" button (owner only) — calls `DELETE /repos/{owner}/{repo}/collaborators/{username}`

**Joining an existing workspace:**
- Teammate accepts GitHub invite → clones the repo into their local `.parallax/` path
- On next Parallax launch, workspace is detected and opened automatically
- Their GitHub identity is used from first launch

**Multi-workspace / multi-team:**
- Each workspace is its own git repo with its own collaborator list
- A user can be on different teams in different workspaces — naturally, since each is a separate GitHub repo
- No concept of an "org-level" team in Parallax — GitHub orgs handle that at the repo level

### 5. Real-Time Chat (Go Sidecar)

Chat is opt-in per user. When enabled, the Go sidecar spins up a WebSocket listener and the user becomes discoverable as a peer. When disabled, the listener never starts and the user never writes to `presence.json` — teammates see them as offline.

**New gRPC service in `src-go/` (`chat.go`):**

```proto
service Chat {
  rpc ConnectPeer  (PeerInfo)       returns (stream ChatMessage);
  rpc SendMessage  (ChatMessage)    returns (Ack);
  rpc GetHistory   (WorkspaceId)    returns (stream ChatMessage);
  rpc SetPresence  (PresenceUpdate) returns (Ack);
  rpc ListPeers    (WorkspaceId)    returns (PeerList);
}

message ChatMessage {
  string  id           = 1;
  string  workspace_id = 2;
  string  sender_login = 3;  // GitHub login
  string  sender_name  = 4;
  string  body         = 5;
  int64   timestamp    = 6;
}

message PeerInfo {
  string github_login = 1;
  string ip           = 2;
  int32  port         = 3;
  string workspace_id = 4;
}
```

**Peer discovery via git repo:**

When chat is enabled and user opens a workspace:
1. Sidecar starts WebSocket listener on a random available port
2. Writes `{ "login": "alice", "ip": "192.168.1.5", "port": 54321, "ts": 1714000000 }` into `.parallax/team/presence.json`
3. Commits + pushes `presence.json` to the workspace repo
4. Other teammates pull → their sidecars read `presence.json` → attempt direct WebSocket connections to each peer

`presence.json` entries older than 5 minutes are treated as stale (peer went offline without cleanup).

**Connection modes:**

| Mode | Trigger | How |
|---|---|---|
| **Direct P2P** | Peers on same network or VPN | Sidecar → sidecar WebSocket; real-time |
| **Git-relay fallback** | P2P connection fails | Messages appended to `.parallax/chat/{workspace-id}/messages.jsonl`; each client polls git pull every 15s |
| **Custom relay** | User configures relay URL in workspace settings | Sidecar connects to user-provided WebSocket relay; real-time over internet |

**Persistence:**

All messages (regardless of mode) are appended to:
```
.parallax/chat/{workspace-id}/messages.jsonl
```
One JSON object per line:
```json
{"id":"uuid","sender":"alice","body":"Hey, the /users endpoint returns 422 now","ts":1714000123}
```

This file is git-tracked. Full chat history is versioned with the workspace — a pull brings in messages from teammates even if you were offline.

**Offline queue:**

Messages sent while offline are written to a local queue file (`.parallax/chat/{workspace-id}/outbox.jsonl`, not committed). On next successful push, queued messages are merged into `messages.jsonl` and pushed.

**Chat UI (Svelte 5):**

A collapsible chat panel in the workspace sidebar / Dashboard view:
- Message list with GitHub avatar, username, timestamp, body
- Input field + send button (Enter to send)
- Online presence indicators next to teammate names in team panel
- Unread message badge on the chat panel toggle button
- Chat panel remembers collapsed/expanded state per workspace

**The disable toggle:**

In Settings → Workspace → Chat:
- Toggle: "Enable chat for this workspace" (default: off)
- When off: sidecar skips WebSocket listener, never writes to `presence.json`, never reads or polls chat files
- Teammates have no indication the user disabled it — user simply appears as permanently offline

---

## Architecture Diagram

```
  [Parallax App — Alice]                    [Parallax App — Bob]
  ┌─────────────────────┐                  ┌─────────────────────┐
  │  Svelte 5 UI        │                  │  Svelte 5 UI        │
  │  Chat Panel         │                  │  Chat Panel         │
  └────────┬────────────┘                  └────────┬────────────┘
           │ gRPC (localhost)                        │ gRPC (localhost)
  ┌────────▼────────────┐                  ┌────────▼────────────┐
  │  Go Sidecar :50151  │◄─── WebSocket ──►│  Go Sidecar :50151  │
  │  Chat service       │   (P2P direct)   │  Chat service       │
  └────────┬────────────┘                  └────────┬────────────┘
           │                                        │
           └──────────────── git push/pull ─────────┘
                         .parallax/
                           team/presence.json
                           chat/{ws}/messages.jsonl
```

---

## Success Criteria

- [ ] `cargo tauri dev` — workspace initializes as a git repo on creation
- [ ] User can commit, push, pull, stash from within the app; changes reflect in `.parallax/`
- [ ] GitHub OAuth login completes; avatar + username appear in titlebar
- [ ] Git commits use GitHub name + email as author
- [ ] Team invite by GitHub username sends a GitHub collaborator invite
- [ ] Invited teammate can clone workspace and open it in their Parallax
- [ ] API docs generated and pushed to `gh-pages`; GitHub Pages URL opens in browser
- [ ] Chat works P2P between two instances on the same network with no external server
- [ ] Chat falls back to git-relay (15s polling) when direct P2P fails
- [ ] Disabling chat per-user stops all sidecar listeners and `presence.json` writes with no errors
- [ ] Chat history survives app restart (loaded from `messages.jsonl`)
- [ ] Messages sent offline are queued and flushed on next git push

---

## Next Phase

Once the above criteria are met → **Phase 3: AI Integration & MCP Server**
