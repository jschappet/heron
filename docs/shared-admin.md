# Shared administration and contributor content

Heron owns the canonical administration source in `shared/admin/`, plus the API
client in `alpine_custom.js`. Both site repositories receive byte-identical copies
of common components, scripts, and admin pages. Their branding/layouts and content
remain independent. ReVillage's existing source directory is retained.

Run from Heron:

```sh
node utils/sync-admin.mjs
node utils/sync-admin.mjs --check
node --test tests/shared-admin.test.mjs tests/alpine-client.test.mjs
cargo test --lib
```

The synchronizer expects the existing sibling `regen-skagit` and
`revillage-society` checkouts. Commit the source and generated changes together
across all three repositories. Do not edit generated copies independently.
Each site's `npm run build` checks its copy against a checked-in SHA-256 manifest;
this works in isolated hosting builds without a neighboring Heron checkout.
`--check` additionally compares both sites against the current canonical source.
This arrangement can later become a versioned build-time include/package without
rewriting the shared editor or list logic.

## User-facing behavior

- `/my/content/`: own content, schema-derived content types, drafts, search,
  filters, and pagination. `/my/content/new/` opens the content-type chooser;
  `/my/content/edit/?doc_type=post&id=123` edits an item using static-compatible URLs.
- `/admin/content/`: the same list logic, with all accessible site content,
  mobile filter panel/cards, desktop table option, and review/approval controls.
- `/admin/roles/` (also `/admin/users/`): current-site assignments, grant/revoke,
  exact-email lookup of an existing account, confirmations, and errors.
- Draft saving stays in the editor. Submission saves first, then submits the
  returned ID; failed submission retains that saved draft for retry.
- Filters, sorting, page, and desktop view are remembered in session storage per
  site and workspace. Browser navigation warns about unsaved editor changes.
- Approval remains distinct from deployment. The Markdown/static publishing
  pipeline and content identifiers are preserved.

## Configuration and preserved differences

`src/_data/contentManagement.js` in each site declares optional specialized
admin editor URLs. Regenerate Skagit retains its existing experiment/result
editors and research work. Personal content uses the common schema editor.
Specialized site-only administration pages remain local. The shared dashboard,
messages, API workbench, ledger pages, content tools, and API client are tracked
by the synchronization manifest. Site layouts and public navigation are local.

The inspected backend has one `doc_schema.json`, not separate per-host doctype
settings. The UI now gets configured types from that API instead of hardcoding
a list; both hosts currently share those configured types. No new host-selector
or doctype configuration system was introduced.

## Permissions and compatibility

- Existing `member` grants contributor access; reviewer and admin also qualify.
- Direct reads, exports, edits, and submissions enforce the request's host and
  ownership. Reviewers/admins can access site content; their roles on another
  site do not grant access here. Contributors can edit drafts or content with
  changes requested, and can view their submitted/approved content.
- Ordinary saves cannot change owner, host, approval status, or review metadata.
  Required configured fields are checked again before submission. Workflow
  checks and mutations use write transactions where needed.
- New `content_*` capabilities use `/api/drafts/...`, outside the old admin-only
  scope. Existing admin draft URLs and `draft_create_public` remain available.
- Role assignment uses the server host, ignores supplied host IDs, is idempotent,
  and reactivates existing assignments. Revocation retains membership records.
  A write transaction prevents removing the last active site administrator.
- Email lookup returns only an exact matching active account. It exposes no role
  assignments from other sites, and allows granting the first role here without
  manual SQL or introducing account invitations.
- Role definitions are global. Their mutation endpoints now reject requests;
  site administrators manage assignments, not global definitions that could
  change another site's access or bypass the last-administrator safeguard.
- Profile roles and advertised capabilities are scoped to the current host.
- Optional ledger logging cannot panic after a draft was successfully saved if
  the site's IdeaBank identity is missing; the failure is logged for operators.

## Verification

Automated coverage includes ownership, cross-site access, submitted-content
editing restrictions, forged owner/status/host payloads, role assignment and
revocation isolation, last-administrator protection, save/submit ordering,
failed-save recovery, concurrent cross-site draft creation, pagination/filter persistence, and drift detection.
Existing research export tests cover stable identifiers and Markdown metadata.
Both Eleventy production builds were checked. Browser verification uses an
isolated local API fixture with sample content; it does not modify real accounts,
content, or role assignments. Production authentication/deployment remains a
release smoke-test step.
