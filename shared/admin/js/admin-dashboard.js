/* Shared admin dashboard; keep identical in both sites. */
window.adminDashboard = () => ({
  loading: true, error: '', messages: null, drafts: null, workbench: null, ledger: null,
  failures: {},
  async get(url) {
    const response = await fetch(url, {credentials: 'same-origin', cache: 'no-store'});
    if (!response.ok) throw new Error(response.status === 401 ? 'Please log in to view the dashboard.' : response.status === 403 ? 'Admin access for this site is required.' : 'This summary is currently unavailable.');
    return response.json();
  },
  init() { return this.refresh(); },
  async refresh() {
    this.loading = true; this.error = ''; this.failures = {};
    this.messages = this.drafts = this.workbench = this.ledger = null;
    // Confirm admin membership for this host before requesting any summaries.
    try { this.messages = await this.get('/api/admin/messages/count'); }
    catch (error) { this.error = error.message; this.loading = false; return; }
    await Promise.all([
      ['drafts', '/api/admin/drafts/list', data => ({
        total: data.meta.total,
        draft: data.drafts.filter(d => d.status === 'draft').length,
        submitted: data.drafts.filter(d => d.status === 'submitted').length,
        changes: data.drafts.filter(d => d.status === 'changes_requested').length
      })],
      ['workbench', '/api/config/capabilities', data => ({total: data.length, reads: data.filter(r => r.method === 'GET').length})],
      ['ledger', '/api/ledger/ledger.json?limit=100', data => ({total: data.total_events})]
    ].map(async ([key, url, summarize]) => {
      try { this[key] = summarize(await this.get(url)); }
      catch (_) { this.failures[key] = 'Summary unavailable. Open this section to try again.'; }
    }));
    this.loading = false;
  }
});
