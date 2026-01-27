DROP INDEX IF EXISTS idx_social_hashtags_popular;
DROP INDEX IF EXISTS idx_social_hashtags_tag;
DROP INDEX IF EXISTS idx_social_hashtags_org_bot;

DROP INDEX IF EXISTS idx_social_bookmarks_post;
DROP INDEX IF EXISTS idx_social_bookmarks_user;

DROP INDEX IF EXISTS idx_social_praises_created;
DROP INDEX IF EXISTS idx_social_praises_to;
DROP INDEX IF EXISTS idx_social_praises_from;
DROP INDEX IF EXISTS idx_social_praises_org_bot;

DROP INDEX IF EXISTS idx_social_announcements_pinned;
DROP INDEX IF EXISTS idx_social_announcements_priority;
DROP INDEX IF EXISTS idx_social_announcements_active;
DROP INDEX IF EXISTS idx_social_announcements_org_bot;

DROP INDEX IF EXISTS idx_social_poll_votes_user;
DROP INDEX IF EXISTS idx_social_poll_votes_poll;
DROP INDEX IF EXISTS idx_social_poll_options_poll;
DROP INDEX IF EXISTS idx_social_polls_post;

DROP INDEX IF EXISTS idx_social_reactions_user;
DROP INDEX IF EXISTS idx_social_reactions_comment;
DROP INDEX IF EXISTS idx_social_reactions_post;

DROP INDEX IF EXISTS idx_social_comments_created;
DROP INDEX IF EXISTS idx_social_comments_author;
DROP INDEX IF EXISTS idx_social_comments_parent;
DROP INDEX IF EXISTS idx_social_comments_post;

DROP INDEX IF EXISTS idx_social_posts_hashtags;
DROP INDEX IF EXISTS idx_social_posts_created;
DROP INDEX IF EXISTS idx_social_posts_announcement;
DROP INDEX IF EXISTS idx_social_posts_pinned;
DROP INDEX IF EXISTS idx_social_posts_visibility;
DROP INDEX IF EXISTS idx_social_posts_parent;
DROP INDEX IF EXISTS idx_social_posts_community;
DROP INDEX IF EXISTS idx_social_posts_author;
DROP INDEX IF EXISTS idx_social_posts_org_bot;

DROP INDEX IF EXISTS idx_social_community_members_role;
DROP INDEX IF EXISTS idx_social_community_members_user;
DROP INDEX IF EXISTS idx_social_community_members_community;

DROP INDEX IF EXISTS idx_social_communities_owner;
DROP INDEX IF EXISTS idx_social_communities_featured;
DROP INDEX IF EXISTS idx_social_communities_visibility;
DROP INDEX IF EXISTS idx_social_communities_slug;
DROP INDEX IF EXISTS idx_social_communities_org_bot;

DROP TABLE IF EXISTS social_hashtags;
DROP TABLE IF EXISTS social_bookmarks;
DROP TABLE IF EXISTS social_praises;
DROP TABLE IF EXISTS social_announcements;
DROP TABLE IF EXISTS social_poll_votes;
DROP TABLE IF EXISTS social_poll_options;
DROP TABLE IF EXISTS social_polls;
DROP TABLE IF EXISTS social_reactions;
DROP TABLE IF EXISTS social_comments;
DROP TABLE IF EXISTS social_posts;
DROP TABLE IF EXISTS social_community_members;
DROP TABLE IF EXISTS social_communities;
