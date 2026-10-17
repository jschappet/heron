/* Shared by both sites. Update through Heron's sync-admin script. */
function contentWorkspace(personal = false) {
  return {
    personal, signedIn: false, drafts: [], types: [], error: '', loading: true, busy: false,
    filtersOpen: false, view: 'list', page: 1, pageSize: 20, selected: [],
    filters: {search: '', docType: '', status: '', author: '', dateFrom: '', dateTo: '', sort: personal ? 'drafts' : 'recent'},
    get stateKey() { return this.personal ? 'my-content-state' : 'admin-content-state'; },
    async init() {
      try {
        if (!await requireContentLogin()) return;
        this.signedIn = true;
        const saved = JSON.parse(sessionStorage.getItem(this.stateKey) || 'null');
        if (saved) { Object.assign(this.filters, saved.filters); this.page = saved.page || 1; this.view = saved.view || 'list'; }
        const schema = await Alpine.store('api').call('content_schema');
        this.types = Object.entries(schema.types).map(([value, type]) => ({value, label: type.label}));
        await this.load();
        this.$watch('filters', () => { this.page = 1; this.persist(); });
        this.$watch('page', () => this.persist());
        this.$watch('view', () => this.persist());
      } catch (error) { this.error = error.message; }
      finally { this.loading = false; }
    },
    persist() { sessionStorage.setItem(this.stateKey, JSON.stringify({filters: this.filters, page: this.page, view: this.view})); },
    async load() {
      this.error = '';
      try {
        const response = await Alpine.store('api').call('content_list', {mine: this.personal});
        this.drafts = response.drafts || [];
        this.selected = [];
        this.page = Math.min(this.page, this.pages);
      } catch (error) { this.error = error.message; }
    },
    get filtered() {
      const f = this.filters, search = f.search.toLowerCase(), author = f.author.toLowerCase();
      const rows = this.drafts.filter(d => (!f.docType || d.doc_type === f.docType)
        && (!f.status || d.status === f.status)
        && (!author || (d.author || '').toLowerCase().includes(author))
        && (!search || `${d.title} ${d.description || ''}`.toLowerCase().includes(search))
        && (!f.dateFrom || (d.submitted_at || '').slice(0, 10) >= f.dateFrom)
        && (!f.dateTo || (!!d.submitted_at && d.submitted_at.slice(0, 10) <= f.dateTo)));
      return rows.sort((a, b) => {
        if (f.sort === 'recent') return b.id - a.id;
        if (f.sort === 'drafts') return Number(b.status === 'draft') - Number(a.status === 'draft') || b.id - a.id;
        return String(a[f.sort] || '').localeCompare(String(b[f.sort] || ''));
      });
    },
    get pages() { return Math.max(1, Math.ceil(this.filtered.length / this.pageSize)); },
    get visible() { return this.filtered.slice((this.page - 1) * this.pageSize, this.page * this.pageSize); },
    editorUrl(type, id) {
      const base = this.personal ? '/my/content/edit/' : (window.contentEditorRoutes?.[type] || '/admin/content/edit/');
      const params = new URLSearchParams({doc_type: type});
      if (id) params.set('id', id);
      return `${base}?${params}`;
    },
    async approve(draft) {
      if (this.busy || !confirm(`Approve “${draft.title}” for the next site build?`)) return;
      this.busy = true;
      try { await Alpine.store('api').call('content_approve', {id: draft.id}, {}); await this.load(); }
      catch (error) { this.error = error.message; }
      finally { this.busy = false; }
    },
    async bulk(action) {
      if (this.busy || !this.selected.length || !confirm(`${action === 'approve' ? 'Approve' : 'Delete'} ${this.selected.length} selected items?`)) return;
      this.busy = true;
      try {
        await Alpine.store('api').call(action === 'approve' ? 'draft_bulk_approve' : 'delete_draft_api', {}, {ids: this.selected.map(Number)});
        await this.load();
      } catch (error) { this.error = error.message; }
      finally { this.busy = false; }
    }
  };
}

function siteRoles() {
  return {
    users: [], roles: [], assignments: [], search: '', lookupEmail: '', userId: '', roleId: '', error: '', message: '', busy: false,
    async init() { await this.load(); },
    async load() {
      this.error = '';
      try {
        [this.users, this.roles, this.assignments] = await Promise.all([
          Alpine.store('api').call('site_users'), Alpine.store('api').call('roles_list'), Alpine.store('api').call('get_memberships')
        ]);
        this.roles = this.roles.filter(r => ['admin', 'reviewer', 'member', 'organizer', 'volunteer', 'guest'].includes(r.name));
      } catch (error) { this.error = error.message; }
    },
    get filteredUsers() { const q = this.search.toLowerCase(); return this.users.filter(u => `${u.username} ${u.email}`.toLowerCase().includes(q)); },
    userRoles(id) { return this.assignments.filter(m => m.user_id === id); },
    roleName(id) { return this.roles.find(r => r.id === id)?.name || 'Unknown role'; },
    async lookup() {
      this.error = ''; this.message = '';
      if (!this.lookupEmail.trim()) return;
      try {
        const matches = await Alpine.store('api').call('site_users', {email: this.lookupEmail.trim()});
        if (!matches.length) { this.message = 'No active account matches that email.'; return; }
        const user = matches[0];
        if (!this.users.some(u => u.id === user.id)) this.users.push(user);
        this.userId = String(user.id);
        this.message = 'Account found. Choose a role to grant access to this site.';
      } catch (error) { this.error = error.message; }
    },
    async assign() {
      if (this.busy || !this.userId || !this.roleId) return;
      this.busy = true; this.error = ''; this.message = '';
      try {
        await Alpine.store('api').call('assign_site_role', {}, {user_id: Number(this.userId), role_id: Number(this.roleId)});
        await this.load(); this.message = 'Role assigned.';
      } catch (error) { this.error = error.message; }
      finally { this.busy = false; }
    },
    async revoke(assignment, user) {
      if (this.busy || !confirm(`Revoke ${this.roleName(assignment.role_id)} access for ${user.username}?`)) return;
      this.busy = true; this.error = ''; this.message = '';
      try {
        await Alpine.store('api').call('revoke_site_role', {id: assignment.id});
        await this.load(); this.message = 'Role revoked.';
      } catch (error) { this.error = error.message; }
      finally { this.busy = false; }
    }
  };
}
