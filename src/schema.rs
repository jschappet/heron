// @generated automatically by Diesel CLI.

diesel::table! {
    completed_offers (id) {
        id -> Integer,
        offer_id -> Integer,
        reviewer_id -> Integer,
        rating -> Nullable<Integer>,
        review -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    contribution_events (id) {
        id -> Integer,
        context_id -> Integer,
        contributor_id -> Integer,
        effort_date -> Nullable<Timestamp>,
        hours -> Nullable<Float>,
        work_done -> Text,
        details -> Text,
        appreciation_message -> Text,
        public_flag -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    contributors (id) {
        id -> Integer,
        name -> Nullable<Text>,
        email -> Nullable<Text>,
        user_id -> Nullable<Integer>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    drafts (id) {
        id -> Integer,
        doc_type -> Text,
        title -> Text,
        description -> Nullable<Text>,
        tags -> Nullable<Text>,
        author -> Nullable<Text>,
        meta -> Nullable<Text>,
        body_md -> Text,
        status -> Text,
        submitted_by -> Integer,
        submitted_at -> Nullable<Timestamp>,
        reviewed_by -> Nullable<Integer>,
        reviewed_at -> Nullable<Timestamp>,
        review_notes -> Nullable<Text>,
        details -> Nullable<Text>,
    }
}

diesel::table! {
    effort_contexts (id) {
        id -> Integer,
        context_type -> Text,
        short_code -> Text,
        name -> Text,
        description -> Text,
        created_at -> Timestamp,
        active_flag -> Bool,
    }
}

diesel::table! {
    events (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        start_time -> Timestamp,
        end_time -> Timestamp,
        location -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    hosts (id) {
        id -> Integer,
        slug -> Text,
        host_name -> Text,
        display_name -> Text,
        base_url -> Text,
        created_at -> Timestamp,
        active -> Bool,
    }
}

diesel::table! {
    mailing_list_subscribers (id) {
        id -> Integer,
        host_id -> Integer,
        name -> Text,
        email -> Text,
        confirmed -> Bool,
        confirmation_token -> Nullable<Text>,
        unsubscribed -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    memberships (id) {
        id -> Integer,
        user_id -> Integer,
        role_id -> Integer,
        host_id -> Integer,
        active -> Bool,
        created_at -> Timestamp,
        ended_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    offers (id) {
        id -> Integer,
        user_id -> Integer,
        title -> Text,
        offer -> Text,
        request -> Text,
        location -> Nullable<Text>,
        contact_link -> Nullable<Text>,
        start_date -> Timestamp,
        end_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
        details -> Text,
    }
}

diesel::table! {
    question_summaries (id) {
        id -> Nullable<Integer>,
        question_uuid -> Text,
        answers_count -> Integer,
        question_text -> Text,
        summary -> Text,
        prompt -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    rating_events (id) {
        id -> Integer,
        rating_type -> Text,
        target_id -> Text,
        user_id -> Nullable<Integer>,
        rating -> Integer,
        review -> Nullable<Text>,
        rating_details -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    rating_summary (rating_type, target_id) {
        rating_type -> Text,
        target_id -> Text,
        rating_sum -> Integer,
        rating_count -> Integer,
        average_rating -> Float,
        last_updated -> Timestamp,
    }
}

diesel::table! {
    recipe_drafts (id) {
        id -> Integer,
        title -> Text,
        description -> Text,
        tags -> Text,
        author -> Text,
        prep_time -> Nullable<Integer>,
        cook_time -> Nullable<Integer>,
        total_time -> Nullable<Integer>,
        servings -> Nullable<Integer>,
        difficulty -> Nullable<Text>,
        source -> Nullable<Text>,
        dietary -> Nullable<Text>,
        body_md -> Text,
        status -> Text,
        submitted_by -> Integer,
        submitted_at -> Nullable<Timestamp>,
        reviewed_by -> Nullable<Integer>,
        reviewed_at -> Nullable<Timestamp>,
        review_notes -> Nullable<Text>,
        details -> Nullable<Text>,
    }
}

diesel::table! {
    registration (id) {
        id -> Integer,
        event_id -> Text,
        user_id -> Integer,
        name -> Text,
        email -> Text,
        phone -> Text,
        attend -> Bool,
        notification -> Bool,
        source -> Nullable<Text>,
        comments -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    roles (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        show_in_directory -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    sms_replies (id) {
        id -> Integer,
        registration_id -> Nullable<Integer>,
        to_number -> Text,
        from_number -> Text,
        body -> Text,
        received_at -> Timestamp,
        parsed_response -> Nullable<Text>,
        raw_payload -> Nullable<Text>,
    }
}

diesel::table! {
    ticket (id) {
        id -> Text,
        user_id -> Integer,
        event_id -> Text,
        registration_id -> Nullable<Integer>,
        checked_in -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    user_tokens (id) {
        id -> Integer,
        user_id -> Integer,
        token_hash -> Text,
        purpose -> Text,
        expires_at -> Timestamp,
        used_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
        profile_picture -> Nullable<Text>,
        user_details -> Text,
        is_active -> Bool,
    }
}

diesel::table! {
    wants_to_contribute (id) {
        id -> Nullable<Integer>,
        offer_id -> Integer,
        helper_user_id -> Integer,
        who -> Nullable<Text>,
        how_helping -> Nullable<Text>,
        availability_days -> Nullable<Text>,
        availability_times -> Nullable<Text>,
        notes -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    weekly_answers (id) {
        id -> Nullable<Integer>,
        name -> Text,
        email -> Text,
        question_uuid -> Text,
        answer -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(completed_offers -> offers (offer_id));
diesel::joinable!(completed_offers -> users (reviewer_id));
diesel::joinable!(contribution_events -> contributors (contributor_id));
diesel::joinable!(contribution_events -> effort_contexts (context_id));
diesel::joinable!(contributors -> users (user_id));
diesel::joinable!(mailing_list_subscribers -> hosts (host_id));
diesel::joinable!(memberships -> hosts (host_id));
diesel::joinable!(memberships -> roles (role_id));
diesel::joinable!(memberships -> users (user_id));
diesel::joinable!(offers -> users (user_id));
diesel::joinable!(registration -> events (event_id));
diesel::joinable!(registration -> users (user_id));
diesel::joinable!(sms_replies -> registration (registration_id));
diesel::joinable!(ticket -> events (event_id));
diesel::joinable!(ticket -> registration (registration_id));
diesel::joinable!(ticket -> users (user_id));
diesel::joinable!(user_tokens -> users (user_id));
diesel::joinable!(wants_to_contribute -> offers (offer_id));
diesel::joinable!(wants_to_contribute -> users (helper_user_id));

diesel::allow_tables_to_appear_in_same_query!(
    completed_offers,
    contribution_events,
    contributors,
    drafts,
    effort_contexts,
    events,
    hosts,
    mailing_list_subscribers,
    memberships,
    offers,
    question_summaries,
    rating_events,
    rating_summary,
    recipe_drafts,
    registration,
    roles,
    sms_replies,
    ticket,
    user_tokens,
    users,
    wants_to_contribute,
    weekly_answers,
);
