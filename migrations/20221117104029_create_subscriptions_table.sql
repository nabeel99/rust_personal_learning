-- Add migration script here
CREATE TABLE subscriptions(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email TEXT NOT NULL UNIQUE,
    name  TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL 
);



--
-- Did you know you can embed your migrations in your application binary?
-- On startup, after creating your database connection or pool, add:

-- sqlx::migrate!().run(<&your_pool OR &mut your_connection>).await?;