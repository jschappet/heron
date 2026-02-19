// AUTO-GENERATED — do not edit by hand
export const ROUTES = {
  "register_new_user": {
    "url": "/api/auth/register",
    "method": "POST",
    "auth": false
  },
  "verify_account": {
    "url": "/api/auth/token/{token}",
    "method": "GET",
    "auth": false
  },
  "register_new_user_off": {
    "url": "/api/auth/register1",
    "method": "POST",
    "auth": false
  },
  "login": {
    "url": "/api/auth/login",
    "method": "POST",
    "auth": false
  },
  "logout": {
    "url": "/api/auth/logout",
    "method": "POST",
    "auth": false
  },
  "create_user_api": {
    "url": "/api/users/",
    "method": "POST",
    "auth": false
  },
  "get_users_page": {
    "url": "/api/users/users/page",
    "method": "GET",
    "auth": false
  },
  "get_users_api": {
    "url": "/api/users/",
    "method": "GET",
    "auth": true
  },
  "get_public_users_api": {
    "url": "/api/users/public_profile",
    "method": "GET",
    "auth": true
  },
  "get_public_user_api": {
    "url": "/api/users/public_profile/user/{user_id}",
    "method": "GET",
    "auth": false
  },
  "get_user_api": {
    "url": "/api/users/{user_id}",
    "method": "GET",
    "auth": false
  },
  "update_user_api": {
    "url": "/api/users/{user_id}",
    "method": "PUT",
    "auth": false
  },
  "delete_user_api": {
    "url": "/api/users/{user_id}",
    "method": "DELETE",
    "auth": true
  },
  "update_user_details_api": {
    "url": "/api/users/details",
    "method": "POST",
    "auth": true
  },
  "get_reset_password_page": {
    "url": "/api/users/reset-password/{token}",
    "method": "GET",
    "auth": false
  },
  "reset_password_token": {
    "url": "/api/users/reset-password",
    "method": "POST",
    "auth": false
  },
  "set_password_new": {
    "url": "/api/users/set-password",
    "method": "POST",
    "auth": false
  },
  "get_config_api": {
    "url": "/api/config/",
    "method": "GET",
    "auth": false
  },
  "online": {
    "url": "/api/config/ONLINE",
    "method": "GET",
    "auth": false
  },
  "ping": {
    "url": "/api/config/ping",
    "method": "GET",
    "auth": false
  },
  "create_role_api": {
    "url": "/api/roles/",
    "method": "POST",
    "auth": false
  },
  "get_roles_api": {
    "url": "/api/roles/",
    "method": "GET",
    "auth": false
  },
  "get_role_api": {
    "url": "/api/roles/{id}",
    "method": "GET",
    "auth": false
  },
  "update_role_api": {
    "url": "/api/roles/{id}",
    "method": "PUT",
    "auth": false
  },
  "delete_role_api": {
    "url": "/api/roles/{id}",
    "method": "DELETE",
    "auth": false
  },
  "profile_json": {
    "url": "/api/profile/",
    "method": "GET",
    "auth": false
  },
  "profile_offers_json": {
    "url": "/api/profile/offers",
    "method": "GET",
    "auth": true
  },
  "profile_completed_json": {
    "url": "/api/profile/completed",
    "method": "GET",
    "auth": true
  },
  "create_membership_api": {
    "url": "/api/memberships/",
    "method": "POST",
    "auth": true
  },
  "get_memberships_api": {
    "url": "/api/memberships/",
    "method": "GET",
    "auth": true
  },
  "get_memberships_for_user_api": {
    "url": "/api/memberships/user/{id}",
    "method": "GET",
    "auth": true
  },
  "delete_membership_api": {
    "url": "/api/memberships/{id}",
    "method": "DELETE",
    "auth": true
  },
  "create_offer_api": {
    "url": "/api/offers/",
    "method": "POST",
    "auth": true
  },
  "get_offer_api": {
    "url": "/api/offers/{id}",
    "method": "GET",
    "auth": false
  },
  "get_offers_api": {
    "url": "/api/offers/",
    "method": "GET",
    "auth": false
  },
  "get_user_offers_api": {
    "url": "/api/offers/user/{input_uid}/offers",
    "method": "GET",
    "auth": false
  },
  "update_offer_api": {
    "url": "/api/offers/{id}",
    "method": "PUT",
    "auth": false
  },
  "delete_offer_api": {
    "url": "/api/offers/{id}",
    "method": "DELETE",
    "auth": false
  },
  "create_wants_to_contribute": {
    "url": "/api/offers/wants_to_help",
    "method": "POST",
    "auth": true
  },
  "create_contribute_event": {
    "url": "/api/celebrate/efforts",
    "method": "POST",
    "auth": true
  },
  "get_ratings": {
    "url": "/api/ratings/all",
    "method": "GET",
    "auth": false
  },
  "save_ratings": {
    "url": "/api/ratings/save",
    "method": "POST",
    "auth": true
  },
  "get_summaries_array": {
    "url": "/api/ratings/summary/array",
    "method": "GET",
    "auth": false
  },
  "rebuild_summary_route": {
    "url": "/api/ratings/rebuild-summary",
    "method": "POST",
    "auth": false
  },
  "get_summaries": {
    "url": "/api/ratings/summary/all",
    "method": "GET",
    "auth": false
  },
  "create_event_api": {
    "url": "/api/events/events",
    "method": "POST",
    "auth": false
  },
  "get_events_api": {
    "url": "/api/events/events",
    "method": "GET",
    "auth": false
  },
  "get_event_api": {
    "url": "/api/events/event/{event_id}",
    "method": "GET",
    "auth": false
  },
  "update_event_api": {
    "url": "/api/events/event/{event_id}",
    "method": "PUT",
    "auth": false
  },
  "delete_event_api": {
    "url": "/api/events/event/{event_id}",
    "method": "DELETE",
    "auth": false
  },
  "get_pending_registrations_html": {
    "url": "/api/events/events/{event_id}/pending-registrations",
    "method": "GET",
    "auth": false
  },
  "upload": {
    "url": "/api/upload/",
    "method": "POST",
    "auth": true
  },
  "submit_weekly_answers": {
    "url": "/api/weekly-answers/",
    "method": "POST",
    "auth": false
  },
  "get_answers": {
    "url": "/api/weekly-answers/question/{uuid}/answers",
    "method": "GET",
    "auth": false
  },
  "submit_summary": {
    "url": "/api/weekly-answers/question/{uuid}/summary",
    "method": "POST",
    "auth": false
  },
  "get_summary": {
    "url": "/api/weekly-answers/question/{uuid}/summary",
    "method": "GET",
    "auth": false
  },
  "get_all_summaries": {
    "url": "/api/weekly-answers/question-summaries/all",
    "method": "GET",
    "auth": false
  },
  "get_all_answers": {
    "url": "/api/weekly-answers/all",
    "method": "GET",
    "auth": false
  },
  "upload_questions": {
    "url": "/api/weekly-answers/questions/upload",
    "method": "POST",
    "auth": false
  },
  "post_question_summary": {
    "url": "/api/weekly-answers/question-summaries/update",
    "method": "POST",
    "auth": false
  },
  "create_ticket_api": {
    "url": "/api/ticket/",
    "method": "POST",
    "auth": false
  },
  "get_tickets_for_event_api": {
    "url": "/api/ticket/event/{new_event_id}",
    "method": "GET",
    "auth": false
  },
  "get_tickets_api": {
    "url": "/api/ticket/",
    "method": "GET",
    "auth": false
  },
  "get_ticket_api": {
    "url": "/api/ticket/{ticket_id}",
    "method": "GET",
    "auth": false
  },
  "update_ticket_api": {
    "url": "/api/ticket/{ticket_id}",
    "method": "PUT",
    "auth": false
  },
  "generate_qr_code": {
    "url": "/api/ticket/{ticket_id}/qr",
    "method": "GET",
    "auth": false
  },
  "delete_ticket_api": {
    "url": "/api/ticket/{ticket_id}",
    "method": "DELETE",
    "auth": false
  },
  "assign_ticket": {
    "url": "/api/ticket/assign-ticket",
    "method": "POST",
    "auth": false
  },
  "subscribe": {
    "url": "/api/mail/subscribe",
    "method": "POST",
    "auth": false
  },
  "confirm": {
    "url": "/api/mail/confirm/{token}",
    "method": "GET",
    "auth": false
  },
  "unsubscribe": {
    "url": "/api/mail/unsubscribe/{token}",
    "method": "GET",
    "auth": false
  },
  "list_subscribers": {
    "url": "/api/mail/mailing_list",
    "method": "GET",
    "auth": true
  },
  "list_all_effort_context": {
    "url": "/api/celebrate/",
    "method": "GET",
    "auth": true
  },
  "_create_contribute_event": {
    "url": "/api/celebrate/efforts_off",
    "method": "POST",
    "auth": false
  },
  "get_sms_replies": {
    "url": "/api/twillio-admin/sms_replies",
    "method": "GET",
    "auth": false
  },
  "receive_sms_reply": {
    "url": "/api/twillio/webhook",
    "method": "POST",
    "auth": false
  },
  "send_sms_api": {
    "url": "/api/twillio/send_sms",
    "method": "POST",
    "auth": false
  },
  "get_doc_schema": {
    "url": "/api/drafts/doc_schema",
    "method": "GET",
    "auth": true
  },
  "create_draft_api": {
    "url": "/api/drafts/",
    "method": "POST",
    "auth": true
  },
  "get_drafts_api": {
    "url": "/api/drafts/",
    "method": "GET",
    "auth": true
  },
  "get_draft_api": {
    "url": "/api/drafts/{id}",
    "method": "GET",
    "auth": true
  },
  "update_draft_api": {
    "url": "/api/drafts/{id}",
    "method": "POST",
    "auth": true
  },
  "delete_draft_api": {
    "url": "/api/drafts/{id}",
    "method": "DELETE",
    "auth": false
  },
  "submit_draft_api": {
    "url": "/api/drafts/{id}/submit",
    "method": "POST",
    "auth": false
  },
  "request_changes_api": {
    "url": "/api/drafts/{id}/request_changes",
    "method": "POST",
    "auth": false
  },
  "approve_draft_api": {
    "url": "/api/drafts/{id}/approve",
    "method": "POST",
    "auth": true
  },
  "deploy_draft_api": {
    "url": "/api/drafts/{id}/deploy",
    "method": "POST",
    "auth": true
  },
  "get_draft_md_api": {
    "url": "/api/drafts/{id}/md",
    "method": "GET",
    "auth": false
  },
  "bulk_approve": {
    "url": "/api/drafts/bulk/approve",
    "method": "POST",
    "auth": true
  }
};
