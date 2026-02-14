SELECT r.*, u.username, u.email
FROM registration r
JOIN users u ON u.id = r.user_id
WHERE r.event_id = 'AR_JUN1' AND r.attend = 1
  AND NOT EXISTS (
    SELECT 1 FROM ticket t
    WHERE t.user_id = r.user_id AND t.event_id = r.event_id
);



SELECT 
            r.id registration_id, 
            r.event_id event_id,
            u.id user_id,
            u.username, 
            e.name event_name,
            r.email as email,
            r.source as source,
            r.phone as phone,
            r.comments as comments,
            e.created_at event_created_at
        FROM registration r
        JOIN users u ON u.id = r.user_id
        JOIN events e ON e.id = r.event_id
        WHERE r.event_id = 'AR_JUN1' AND r.attend = 1
        AND NOT EXISTS (
            SELECT 1 FROM ticket t
            WHERE t.user_id = r.user_id AND t.event_id = r.event_id
        )