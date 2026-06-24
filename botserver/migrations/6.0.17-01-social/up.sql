CREATE TABLE social_communities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    description TEXT,
    cover_image TEXT,
    icon TEXT,
    visibility VARCHAR(50) NOT NULL DEFAULT 'public',
    join_policy VARCHAR(50) NOT NULL DEFAULT 'open',
    owner_id UUID NOT NULL,
    member_count INTEGER NOT NULL DEFAULT 0,
    post_count INTEGER NOT NULL DEFAULT 0,
    is_official BOOLEAN NOT NULL DEFAULT FALSE,
    is_featured BOOLEAN NOT NULL DEFAULT FALSE,
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ
);

CREATE TABLE social_community_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    community_id UUID NOT NULL REFERENCES social_communities(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    notifications_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ,
    UNIQUE(community_id, user_id)
);

CREATE TABLE social_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    author_id UUID NOT NULL,
    community_id UUID REFERENCES social_communities(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES social_posts(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    content_type VARCHAR(50) NOT NULL DEFAULT 'text',
    attachments JSONB NOT NULL DEFAULT '[]',
    mentions JSONB NOT NULL DEFAULT '[]',
    hashtags TEXT[] NOT NULL DEFAULT '{}',
    visibility VARCHAR(50) NOT NULL DEFAULT 'public',
    is_announcement BOOLEAN NOT NULL DEFAULT FALSE,
    is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    poll_id UUID,
    reaction_counts JSONB NOT NULL DEFAULT '{}',
    comment_count INTEGER NOT NULL DEFAULT 0,
    share_count INTEGER NOT NULL DEFAULT 0,
    view_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE social_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES social_posts(id) ON DELETE CASCADE,
    parent_comment_id UUID REFERENCES social_comments(id) ON DELETE CASCADE,
    author_id UUID NOT NULL,
    content TEXT NOT NULL,
    mentions JSONB NOT NULL DEFAULT '[]',
    reaction_counts JSONB NOT NULL DEFAULT '{}',
    reply_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE social_reactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID REFERENCES social_posts(id) ON DELETE CASCADE,
    comment_id UUID REFERENCES social_comments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    reaction_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT social_reactions_target_check CHECK (
        (post_id IS NOT NULL AND comment_id IS NULL) OR
        (post_id IS NULL AND comment_id IS NOT NULL)
    ),
    UNIQUE(post_id, user_id, reaction_type),
    UNIQUE(comment_id, user_id, reaction_type)
);

CREATE TABLE social_polls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES social_posts(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    allow_multiple BOOLEAN NOT NULL DEFAULT FALSE,
    allow_add_options BOOLEAN NOT NULL DEFAULT FALSE,
    anonymous BOOLEAN NOT NULL DEFAULT FALSE,
    total_votes INTEGER NOT NULL DEFAULT 0,
    ends_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE social_poll_options (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID NOT NULL REFERENCES social_polls(id) ON DELETE CASCADE,
    text VARCHAR(500) NOT NULL,
    vote_count INTEGER NOT NULL DEFAULT 0,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE social_poll_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id UUID NOT NULL REFERENCES social_polls(id) ON DELETE CASCADE,
    option_id UUID NOT NULL REFERENCES social_poll_options(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(poll_id, option_id, user_id)
);

CREATE TABLE social_announcements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    author_id UUID NOT NULL,
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    priority VARCHAR(50) NOT NULL DEFAULT 'normal',
    target_audience JSONB NOT NULL DEFAULT '{}',
    is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    requires_acknowledgment BOOLEAN NOT NULL DEFAULT FALSE,
    acknowledged_by JSONB NOT NULL DEFAULT '[]',
    starts_at TIMESTAMPTZ,
    ends_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE social_praises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    from_user_id UUID NOT NULL,
    to_user_id UUID NOT NULL,
    badge_type VARCHAR(50) NOT NULL,
    message TEXT,
    is_public BOOLEAN NOT NULL DEFAULT TRUE,
    post_id UUID REFERENCES social_posts(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE social_bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    post_id UUID NOT NULL REFERENCES social_posts(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, post_id)
);

CREATE TABLE social_hashtags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    tag VARCHAR(100) NOT NULL,
    post_count INTEGER NOT NULL DEFAULT 0,
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, bot_id, tag)
);

CREATE INDEX idx_social_communities_org_bot ON social_communities(org_id, bot_id);
CREATE INDEX idx_social_communities_slug ON social_communities(slug);
CREATE INDEX idx_social_communities_visibility ON social_communities(visibility);
CREATE INDEX idx_social_communities_featured ON social_communities(is_featured) WHERE is_featured = TRUE;
CREATE INDEX idx_social_communities_owner ON social_communities(owner_id);

CREATE INDEX idx_social_community_members_community ON social_community_members(community_id);
CREATE INDEX idx_social_community_members_user ON social_community_members(user_id);
CREATE INDEX idx_social_community_members_role ON social_community_members(community_id, role);

CREATE INDEX idx_social_posts_org_bot ON social_posts(org_id, bot_id);
CREATE INDEX idx_social_posts_author ON social_posts(author_id);
CREATE INDEX idx_social_posts_community ON social_posts(community_id) WHERE community_id IS NOT NULL;
CREATE INDEX idx_social_posts_parent ON social_posts(parent_id) WHERE parent_id IS NOT NULL;
CREATE INDEX idx_social_posts_visibility ON social_posts(visibility);
CREATE INDEX idx_social_posts_pinned ON social_posts(community_id, is_pinned) WHERE is_pinned = TRUE;
CREATE INDEX idx_social_posts_announcement ON social_posts(is_announcement) WHERE is_announcement = TRUE;
CREATE INDEX idx_social_posts_created ON social_posts(created_at DESC);
CREATE INDEX idx_social_posts_hashtags ON social_posts USING GIN(hashtags);

CREATE INDEX idx_social_comments_post ON social_comments(post_id);
CREATE INDEX idx_social_comments_parent ON social_comments(parent_comment_id) WHERE parent_comment_id IS NOT NULL;
CREATE INDEX idx_social_comments_author ON social_comments(author_id);
CREATE INDEX idx_social_comments_created ON social_comments(created_at DESC);

CREATE INDEX idx_social_reactions_post ON social_reactions(post_id) WHERE post_id IS NOT NULL;
CREATE INDEX idx_social_reactions_comment ON social_reactions(comment_id) WHERE comment_id IS NOT NULL;
CREATE INDEX idx_social_reactions_user ON social_reactions(user_id);

CREATE INDEX idx_social_polls_post ON social_polls(post_id);
CREATE INDEX idx_social_poll_options_poll ON social_poll_options(poll_id);
CREATE INDEX idx_social_poll_votes_poll ON social_poll_votes(poll_id);
CREATE INDEX idx_social_poll_votes_user ON social_poll_votes(user_id);

CREATE INDEX idx_social_announcements_org_bot ON social_announcements(org_id, bot_id);
CREATE INDEX idx_social_announcements_active ON social_announcements(starts_at, ends_at);
CREATE INDEX idx_social_announcements_priority ON social_announcements(priority);
CREATE INDEX idx_social_announcements_pinned ON social_announcements(is_pinned) WHERE is_pinned = TRUE;

CREATE INDEX idx_social_praises_org_bot ON social_praises(org_id, bot_id);
CREATE INDEX idx_social_praises_from ON social_praises(from_user_id);
CREATE INDEX idx_social_praises_to ON social_praises(to_user_id);
CREATE INDEX idx_social_praises_created ON social_praises(created_at DESC);

CREATE INDEX idx_social_bookmarks_user ON social_bookmarks(user_id);
CREATE INDEX idx_social_bookmarks_post ON social_bookmarks(post_id);

CREATE INDEX idx_social_hashtags_org_bot ON social_hashtags(org_id, bot_id);
CREATE INDEX idx_social_hashtags_tag ON social_hashtags(tag);
CREATE INDEX idx_social_hashtags_popular ON social_hashtags(org_id, bot_id, post_count DESC);
