# User Awareness and Project Sync Plan

## Goal

When Asset Forge launches on any machine, it should recognize the signed-in user, load that user's Asset Forge project list, and show whether each project has a local external project folder linked on the current machine.

This slice intentionally does not try to solve every future sync provider. It creates the identity, entitlement, project index, local machine binding, and provider-link shape that GitHub, Steam, Tabletop Simulator, The Game Crafter, and future services can plug into.

## Reference Constraints

- Keep credentials and private endpoint mappings out of the app repo.
- Public app docs should stay vendor-neutral where possible; exact hosted domain mappings belong in private references.
- Use environment-managed secrets for hosted services.
- Prefer compact launcher/index UI with current state visible at a glance.
- Treat local machine paths as machine-specific state, not portable project metadata.

## Opinionated Recommendation

Use a hosted Asset Forge account service as the source of truth for users, entitlements, and the project list. For the fastest reliable implementation, use Firebase Auth plus Firestore first, because the account already exists in the TWS reference inventory and it gives us identity, sync, security rules, and low-ops hosting.

Keep a small VPS-backed API as a later migration point only when we need custom license logic, webhooks, or provider integrations that should not run from the desktop app. Stripe/WooCommerce direct-sale webhooks and Steam partner-only checks should go through this backend, not the Tauri client.

Do not put GitHub, Steam, or store credentials in `project.json`. Store only provider links and IDs there or in the cloud project record. Store user/provider tokens per machine in the OS credential store, or avoid storing them entirely by relying on Git Credential Manager and Steam client session state.

## Data Model

Cloud user profile:

```json
{
  "userId": "auth uid",
  "displayName": "User name",
  "email": "user@example.com",
  "createdAt": "ISO timestamp",
  "updatedAt": "ISO timestamp"
}
```

Cloud project index:

```json
{
  "projectId": "proj_...",
  "ownerUserId": "auth uid",
  "name": "Project name",
  "githubRepositoryUrl": "https://github.com/owner/repo",
  "steamAppId": "optional",
  "steamPackageId": "optional",
  "providerLinks": {
    "github": { "repositoryUrl": "https://github.com/owner/repo" },
    "steam": { "appId": "optional", "packageId": "optional" }
  },
  "createdAt": "ISO timestamp",
  "updatedAt": "ISO timestamp"
}
```

Local machine registry:

```json
{
  "machineId": "generated uuid stored locally",
  "machineName": "human-readable device name",
  "projectBindings": {
    "proj_...": {
      "localFolder": "C:\\Projects\\Game",
      "lastSeenAt": "ISO timestamp"
    }
  }
}
```

Portable project folder metadata:

```json
{
  "schemaVersion": "0.1.0",
  "projectId": "proj_...",
  "name": "Project name",
  "githubRepositoryUrl": "https://github.com/owner/repo",
  "providerLinks": {
    "steam": { "appId": "optional", "packageId": "optional" }
  },
  "createdAt": "ISO timestamp",
  "updatedAt": "ISO timestamp"
}
```

## Narrow Slice Implementation

1. Add account sign-in state to the launcher.
   - First pass: Firebase Auth with email magic link or Google sign-in.
   - Store refresh/session state using the platform credential store, not a plaintext app config.
   - Display signed-in user identity in the launcher header.

2. Add cloud project-list sync.
   - On launch, fetch projects for the signed-in user from Firestore.
   - Merge with locally discovered project folders by `projectId`.
   - Show cloud projects even if no local folder is linked on this machine.

3. Keep local folder links local.
   - Continue storing `projectId -> localFolder` in app data on each machine.
   - Add a stable local `machineId` file.
   - The launcher should show one of three states per project: linked on this machine, not linked on this machine, or missing folder.

4. Add link and relink actions.
   - Link an existing local folder.
   - Clone from GitHub into a selected local folder.
   - Relink when a folder moved.
   - Unlink this machine only.

5. Add GitHub connection status.
   - Use GitHub OAuth device flow or loopback OAuth for account-level GitHub API access.
   - Store GitHub API tokens in OS credential storage.
   - Prefer Git Credential Manager or SSH keys for actual `git pull` and `git push`; the app should not become a Git credential helper.

6. Add basic sync actions.
   - Project list sync: cloud to launcher.
   - Git status: local only.
   - Git pull: local Git repo from origin.
   - Git push: local Git repo to origin.
   - Record last sync result locally and optionally in cloud as non-secret status metadata.

## Steam and Sales Recommendation

Use one Asset Forge account model and multiple entitlement providers.

Direct sale:

- Use Stripe or WooCommerce for checkout.
- Webhooks update the Asset Forge backend entitlement table.
- The desktop app signs into Asset Forge and asks the backend what the user owns.

Steam sale:

- The Steam build should use Steamworks identity/ownership as an entitlement provider.
- On launch from Steam, the app gets Steam identity through Steamworks and verifies it with the Asset Forge backend.
- The backend links the Steam account to the Asset Forge user and grants the Steam entitlement.
- Do not attempt to bypass Steam purchases inside the Steam build. Steam in-app purchases should use Steam Wallet-backed flows when needed.

Practical rollout:

- Phase 1: Direct-sale entitlement only, with manual/admin entitlement grants if needed.
- Phase 2: Steam build recognizes Steam launch and links Steam ID to Asset Forge account.
- Phase 3: Steam ownership verification grants entitlement automatically.
- Phase 4: Optional Steam package/app link per project for developer workflow metadata.

Steam project links should be metadata, not credentials. A project can store `steamAppId` and optional `steamPackageId`, but any Steam partner API keys belong only on the backend.

## GitHub Credential Recommendation

Do not ask the user to paste a GitHub password. Do not store a PAT in `project.json`.

Use this order:

1. For local Git operations, rely on the user's configured Git authentication: Git Credential Manager, browser login, or SSH keys.
2. For app-level GitHub features, use GitHub OAuth device flow or loopback OAuth and store the resulting token in OS credential storage.
3. For organization/team workflows later, consider a GitHub App installation instead of user PATs.
4. Keep fine-grained PAT paste as an advanced fallback only, scoped to selected repositories and stored in OS credential storage.

## Future Provider Shape

Every sync provider should follow the same structure:

```json
{
  "provider": "github | steam | tabletopSimulator | gameCrafter",
  "portableLink": "repo url, app id, package id, product id",
  "localBinding": "machine-specific local path or local app install state",
  "credentialRef": "OS keychain/backend reference, never the secret itself",
  "lastSync": "status metadata"
}
```

## Acceptance Criteria

- Launching Asset Forge shows the signed-in user.
- Project list appears after sign-in on a new machine.
- Projects without a local folder are visible and marked as not linked on this machine.
- Linking a local folder affects only the current machine.
- Git status/pull/push only enable for projects linked to an existing local Git folder.
- GitHub and Steam secrets are not stored in `project.json` or repo-tracked files.
- Direct-sale and Steam ownership can both map into one entitlement model.

## External References Checked

- GitHub OAuth app authorization and device/loopback flows: https://docs.github.com/apps/building-oauth-apps/authorizing-oauth-apps
- GitHub personal access token guidance: https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens
- Steam user authentication and ownership: https://partner.steamgames.com/doc/features/auth
- Steam microtransaction guidance: https://partner.steamgames.com/doc/features/microtransactions
