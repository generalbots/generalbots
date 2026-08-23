
    (function () {
        "use strict";
if (window.GBAppLifecycle) GBAppLifecycle.begin("social");

        function showNewPostModal() {
            document.getElementById("newPostModal").classList.add("active");
            document.querySelector(".post-textarea").focus();
        }

        function closeNewPostModal() {
            document.getElementById("newPostModal").classList.remove("active");
            document.getElementById("newPostForm").reset();
        }

        function handlePostCreated(event) {
            if (event.detail.successful) {
                closeNewPostModal();
                htmx.trigger("#social-content", "refresh");
                if (window.GBAlerts) {
                    window.GBAlerts.info(
                        "Social",
                        "Post created successfully!",
                    );
                }
            }
        }

        function togglePollCreator() {
            var attachments = document.getElementById("postAttachments");
            var existing = attachments.querySelector(".poll-creator");
            if (existing) {
                existing.remove();
                return;
            }
            var pollHtml =
                '<div class="poll-creator">' +
                '<input type="text" name="poll_question" placeholder="Ask a question..." class="poll-question" />' +
                '<div class="poll-options">' +
                '<input type="text" name="poll_option_1" placeholder="Option 1" />' +
                '<input type="text" name="poll_option_2" placeholder="Option 2" />' +
                '<button type="button" class="btn-add-option" onclick="addPollOption()">+ Add option</button>' +
                "</div>" +
                "</div>";
            attachments.innerHTML = pollHtml;
        }

        function addPollOption() {
            var options = document.querySelector(".poll-options");
            var count = options.querySelectorAll("input").length + 1;
            var input = document.createElement("input");
            input.type = "text";
            input.name = "poll_option_" + count;
            input.placeholder = "Option " + count;
            options.insertBefore(
                input,
                options.querySelector(".btn-add-option"),
            );
        }

        document.querySelectorAll(".social-tab").forEach(function (tab) {
            tab.addEventListener("click", function () {
                document.querySelectorAll(".social-tab").forEach(function (t) {
                    t.classList.remove("active");
                });
                this.classList.add("active");
            });
        });

        window.showNewPostModal = showNewPostModal;
        window.closeNewPostModal = closeNewPostModal;
        window.handlePostCreated = handlePostCreated;
        window.togglePollCreator = togglePollCreator;
        window.addPollOption = addPollOption;

        window.showFeed = function() {
            var content = document.getElementById('social-content');
            content.innerHTML = '<div style="text-align:center;padding:40px;color:var(--text-secondary)">Loading feed...</div>';
            fetch('/api/social/feed').then(function(r){return r.json()}).then(function(d){
                if (!d || !d.posts || !d.posts.length) {
                    content.innerHTML = '<div class="empty-feed"><p>No posts yet. Be the first to share something!</p></div>';
                    return;
                }
                content.innerHTML = d.posts.map(function(p) {
                    var authorName = p.author ? (p.author.name || p.author.display_name || 'User') : (p.author_name || 'User');
                    return '<div class="social-post"><div class="post-header"><strong>' + authorName + '</strong></div><div class="post-body">' + (p.content || '') + '</div></div>';
                }).join('');
            }).catch(function(){ content.innerHTML = '<div class="empty-feed"><p>Could not load feed.</p></div>'; });
        };

        window.showCommunities = function() {
            var content = document.getElementById('social-content');
            content.innerHTML = '<div style="text-align:center;padding:40px;color:var(--text-secondary)">Loading communities...</div>';
            fetch('/api/social/communities').then(function(r){return r.json()}).then(function(d){
                if (!d || !d.length) {
                    content.innerHTML = '<div class="empty-feed"><p>No communities yet.</p></div>';
                    return;
                }
                content.innerHTML = d.map(function(c) {
                    return '<div class="social-post"><div class="post-header"><strong>' + (c.name || 'Community') + '</strong></div><div class="post-body">' + (c.description || '') + '</div></div>';
                }).join('');
            }).catch(function(){ content.innerHTML = '<div class="empty-feed"><p>Could not load communities.</p></div>'; });
        };

        window.showAnnouncements = function() {
            var content = document.getElementById('social-content');
            content.innerHTML = '<div style="text-align:center;padding:40px;color:var(--text-secondary)">No announcements yet.</div>';
        };

        window.showAnalytics = function() {
            var content = document.getElementById('social-content');
            content.innerHTML = '<div class="analytics-dashboard">'
                + '<h3 style="margin-bottom:16px">Social Analytics</h3>'
                + '<div class="analytics-grid">'
                + '<div class="analytics-card"><div class="analytics-value" id="analytics-posts">--</div><div class="analytics-label">Posts</div></div>'
                + '<div class="analytics-card"><div class="analytics-value" id="analytics-engagement">--</div><div class="analytics-label">Engagement</div></div>'
                + '<div class="analytics-card"><div class="analytics-value" id="analytics-reach">--</div><div class="analytics-label">Reach</div></div>'
                + '<div class="analytics-card"><div class="analytics-value" id="analytics-followers">--</div><div class="analytics-label">Followers</div></div>'
                + '</div>'
                + '<div class="analytics-chart" style="margin-top:20px;padding:20px;background:var(--surface);border:1px solid var(--border);border-radius:8px">'
                + '<h4 style="margin-bottom:12px">Engagement Over Time</h4>'
                + '<div id="engagement-chart" style="height:200px;display:flex;align-items:end;gap:4px"></div>'
                + '</div>'
                + '<div class="analytics-chart" style="margin-top:16px;padding:20px;background:var(--surface);border:1px solid var(--border);border-radius:8px">'
                + '<h4 style="margin-bottom:12px">Best Posting Times</h4>'
                + '<div id="best-times" style="color:var(--text-secondary)">Analyzing your post data...</div>'
                + '</div>'
                + '</div>';
            fetch('/api/social/analytics').then(function(r){return r.json()}).then(function(d){
                document.getElementById('analytics-posts').textContent = d.total_posts || 0;
                document.getElementById('analytics-engagement').textContent = d.engagement_rate || '0%';
                document.getElementById('analytics-reach').textContent = d.reach || '0';
                document.getElementById('analytics-followers').textContent = d.followers || 0;
                var chart = document.getElementById('engagement-chart');
                if (d.engagement_history && chart) {
                    chart.innerHTML = d.engagement_history.map(function(v) {
                        return '<div style="flex:1;background:var(--accent, #3b82f6);height:' + (v.value || 10) + '%;border-radius:2px 2px 0 0" title="' + v.label + ': ' + v.value + '"></div>';
                    }).join('');
                }
                var bt = document.getElementById('best-times');
                if (d.best_times && bt) bt.textContent = d.best_times;
            }).catch(function(){});
        };

        window.likePost = function(postId) {
            fetch('/api/social/posts/' + postId + '/react', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({reaction_type: 'like'})
            }).then(function(){
                htmx.trigger('#social-content', 'refresh');
            });
        };

        window.commentPost = function(postId) {
            var input = document.getElementById('comment-input-' + postId);
            if (!input || !input.value.trim()) return;
            fetch('/api/social/posts/' + postId + '/comments', {
                method: 'POST',
                headers: {'Content-Type':'application/json'},
                body: JSON.stringify({content: input.value.trim()})
            }).then(function(){
                input.value = '';
                htmx.trigger('#social-content', 'refresh');
            });
        };

        window.sharePost = function(postId) {
            fetch('/api/social/posts/' + postId + '/react', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({reaction_type: 'share'})
            }).then(function(){
                if (window.GBAlerts) window.GBAlerts.info('Social', 'Post shared!');
            });
        };

        window.deletePost = function(postId) {
            if (!confirm('Delete this post?')) return;
            fetch('/api/social/posts/' + postId, {method:'DELETE'}).then(function(){
                htmx.trigger('#social-content', 'refresh');
            });
        };
    })();


// Auto-load the feed once the app is injected into a window.
(function () {
  "use strict";
  var tries = 0;
  var timer = setInterval(function () {
    tries++;
    var activeTab = document.querySelector('.social-tab.active');
    if (typeof window.showFeed === "function" && document.getElementById("social-content") && activeTab) {
      clearInterval(timer);
      if (activeTab.dataset.tab === "feed") window.showFeed();
      return;
    }
    if (tries > 50) clearInterval(timer);
  }, 150);
})();
