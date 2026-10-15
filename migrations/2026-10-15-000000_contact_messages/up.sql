CREATE TABLE contact_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    host_id INTEGER NOT NULL REFERENCES hosts(id),
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    read_at TIMESTAMP
);
CREATE INDEX contact_messages_host_created ON contact_messages(host_id, created_at, id);
CREATE INDEX contact_messages_unread ON contact_messages(host_id) WHERE read_at IS NULL;

-- Subscriber rows retain only the latest message. Import all recoverable messages
-- as unread; their original submission times are not available separately.
INSERT INTO contact_messages (host_id, name, email, message, created_at)
SELECT host_id, name, email, message, created_at
FROM mailing_list_subscribers
WHERE message IS NOT NULL AND length(trim(message)) > 0;

CREATE TABLE contact_message_replies (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    message_id INTEGER NOT NULL REFERENCES contact_messages(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id),
    request_id TEXT NOT NULL,
    from_email TEXT NOT NULL,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('sending', 'sent', 'failed')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sent_at TIMESTAMP,
    UNIQUE(message_id, request_id)
);
CREATE INDEX contact_message_replies_message ON contact_message_replies(message_id, id);
