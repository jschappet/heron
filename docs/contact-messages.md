# Contact message inbox

ReVillage and Regen include **Message (new count)** in the admin menu. Access requires an active admin membership for the resolved host. Opening a message marks it read for all admins of that site. The inbox defaults to new messages; clear the filter to see read messages. **Mark as new** restores the unread state.

The migration imports nonempty `mailing_list_subscribers.message` values as unread. Their dates come from subscriber creation dates, since original message dates were not stored. Previously overwritten submissions cannot be recovered. Future contact submissions are saved separately, including repeat messages from the same email. Mailing-list subscriptions without a message do not create inbox entries.

Replies use Heron's configured SMTP server and `email.smtp_from_email` (falling back to the SMTP username when empty), with the resolved site's display name. The page shows the sender address. Replies are plain text and sent only to the original submitter. The saved history records the admin, sender, subject, body, timestamps, and delivery status. Incoming email replies are handled in that mailbox; they are not imported into this inbox.

A reply request UUID prevents duplicate sends when a browser retries after losing a response. Failed attempts remain in history. An interrupted send can remain in “sending / outcome unknown”; check the mailbox before deliberately sending another reply. “Sent” means the SMTP server accepted the message, not that delivery to the recipient is guaranteed.

## Setup and checks

Restart Heron to apply its normal startup migrations and load the new routes. Build both static sites using `npm run build`. No new mail credentials are required. Back up the production database before applying migrations as usual. The migration does not modify existing subscriber records; rolling it back removes inbox and reply history.

Backend checks: `cargo test`. Frontend behavior checks: `node --test tests/contact-messages.test.cjs` in either site. These tests mock mail delivery and do not send real email.

## API

All routes require the current site's admin membership:

- `GET /api/admin/messages?unread=true&offset=0`: messages, total, unread count; pages of 30.
- `GET /api/admin/messages/count`: unread count.
- `GET /api/admin/messages/{id}`: message, reply history, sender address.
- `PUT /api/admin/messages/{id}/read` with `{"read":true}`: update shared read state.
- `POST /api/admin/messages/{id}/replies` with `request_id`, `subject`, `body`: record and send a reply; inspect the returned status.
