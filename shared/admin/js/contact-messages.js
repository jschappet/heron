/* Shared inbox behavior; keep this file identical in both sites. */
(() => {
  const base = '/api/admin/messages';
  async function request(path = '', options = {}) {
    const response = await fetch(base + path, {
      credentials: 'same-origin', cache: 'no-store',
      ...options, headers: { 'Content-Type': 'application/json', ...options.headers }
    });
    if (!response.ok) {
      if (response.status === 401) throw new Error('Please log in to view messages.');
      if (response.status === 403) throw new Error('You need admin access for this site to view messages.');
      if (response.status === 404) throw new Error('This message was not found for this site.');
      throw new Error('The request could not be completed. Please try again.');
    }
    return response.json();
  }
  function updateCount(count) {
    window.dispatchEvent(new CustomEvent('contact-message-count', { detail: count }));
  }
  window.contactMessageBadge = () => ({
    count: null, allowed: false, timer: null, onCount: null, onFocus: null,
    init() {
      this.onCount = event => { this.count = event.detail; this.allowed = true; };
      this.onFocus = () => this.refresh();
      window.addEventListener('contact-message-count', this.onCount);
      window.addEventListener('focus', this.onFocus);
      this.$watch('$store.auth.loaded', loaded => { if (loaded) this.refresh(); });
      this.$watch('$store.auth.authenticated', () => this.refresh());
      this.refresh();
      this.timer = setInterval(() => { if (!document.hidden) this.refresh(); }, 60000);
    },
    async refresh() {
      if (!this.$store.auth?.authenticated) { this.allowed = false; this.count = null; return; }
      try {
        const result = await request('/count');
        this.count = result.count; this.allowed = true;
      } catch (_) { this.allowed = false; this.count = null; }
    },
    destroy() {
      clearInterval(this.timer);
      window.removeEventListener('contact-message-count', this.onCount);
      window.removeEventListener('focus', this.onFocus);
    }
  });
  window.contactMessageInbox = () => ({
    messages: [], selected: null, replies: [], replyFrom: null,
    unreadOnly: true, total: 0, offset: 0, limit: 30,
    loading: false, opening: false, sending: false, error: '', notice: '',
    subject: '', body: '', requestId: null, requestPayload: null, uncertain: false,
    init() { this.load(); },
    async load(reset = false) {
      if (reset) this.offset = 0;
      this.loading = true; this.error = '';
      try {
        const result = await request(`?unread=${this.unreadOnly}&offset=${this.offset}`);
        this.messages = result.messages; this.total = result.total; this.limit = result.limit;
        updateCount(result.unread_count);
        if (this.offset > 0 && !this.messages.length) { this.offset = Math.max(0, this.offset - this.limit); await this.load(); }
      } catch (error) { this.error = error.message; }
      finally { this.loading = false; }
    },
    async open(message) {
      if (this.opening || this.sending) return;
      if (this.body.trim() && !window.confirm('Discard the unsent reply?')) return;
      this.opening = true; this.error = ''; this.notice = '';
      try {
        const result = await request(`/${message.id}`);
        this.selected = result.message; this.replies = result.replies; this.replyFrom = result.reply_from;
        this.subject = 'Re: Your message'; this.body = ''; this.requestId = null; this.requestPayload = null; this.uncertain = false;
        if (!this.selected.read_at) {
          const read = await request(`/${message.id}/read`, { method: 'PUT', body: JSON.stringify({read: true}) });
          this.selected = read.message; updateCount(read.unread_count);
          await this.load();
        }
      } catch (error) { this.error = error.message; }
      finally { this.opening = false; }
    },
    async markUnread() {
      this.error = '';
      try {
        const result = await request(`/${this.selected.id}/read`, { method: 'PUT', body: JSON.stringify({read: false}) });
        this.selected = result.message; updateCount(result.unread_count); await this.load();
      } catch (error) { this.error = error.message; }
    },
    async send() {
      if (this.sending || !this.selected || !this.replyFrom) return;
      const payload = JSON.stringify({subject: this.subject.trim(), body: this.body.trim()});
      if (!this.subject.trim() || !this.body.trim()) return;
      if (this.uncertain && payload !== this.requestPayload) {
        this.error = 'Check the previous send result before changing this reply. Reload the message to review its history.'; return;
      }
      if (!this.requestId || payload !== this.requestPayload) this.requestId = crypto.randomUUID();
      this.requestPayload = payload;
      this.sending = true; this.error = ''; this.notice = '';
      try {
        const result = await request(`/${this.selected.id}/replies`, {
          method: 'POST', body: JSON.stringify({...JSON.parse(payload), request_id: this.requestId})
        });
        const index = this.replies.findIndex(reply => reply.id === result.id);
        if (index < 0) this.replies.push(result); else this.replies[index] = result;
        this.uncertain = false;
        if (result.status === 'sent') {
          this.notice = 'Reply sent.'; this.body = ''; this.requestId = null; this.requestPayload = null;
        } else if (result.status === 'failed') {
          this.error = 'The mail server did not confirm delivery. Check your mailbox before retrying; the attempt is saved below.';
          this.requestId = null;
        } else {
          this.uncertain = true;
          this.error = 'This reply is still sending, or its outcome is unknown. Check its history before sending another reply.';
        }
      } catch (error) {
        this.uncertain = true;
        this.error = error.message + ' Delivery is not confirmed. Use “Check send result” to retry the same request without sending it twice.';
      } finally { this.sending = false; }
    },
    date(value) { return value ? new Date(value + 'Z').toLocaleString() : ''; },
    status(reply) { return reply.status === 'sent' ? 'Sent' : reply.status === 'failed' ? 'Delivery not confirmed' : 'Sending / outcome unknown'; }
  });
})();
