function showMentionDropdown() {
  var dropdown = document.getElementById("mentionDropdown");
  if (dropdown) dropdown.classList.add("visible");
}

function hideMentionDropdown() {
  var dropdown = document.getElementById("mentionDropdown");
  if (dropdown) dropdown.classList.remove("visible");
  ChatState.mentionState.active = false;
  ChatState.mentionState.query = "";
  ChatState.mentionState.startPos = -1;
  ChatState.mentionState.selectedIndex = 0;
  ChatState.mentionState.results = [];
}

function searchEntities(query) {
  if (!query || query.length < 1) {
    var defaultResults = Object.keys(EntityTypes).map(function (type) {
      return { type: type, name: EntityTypes[type].label, icon: EntityTypes[type].icon, isTypeHint: true };
    });
    renderMentionResults(defaultResults);
    return;
  }

  var colonIndex = query.indexOf(":");
  if (colonIndex > 0) {
    var entityType = query.substring(0, colonIndex).toLowerCase();
    var searchTerm = query.substring(colonIndex + 1);
    if (EntityTypes[entityType]) {
      fetchEntitiesOfType(entityType, searchTerm);
      return;
    }
  }

  var filteredTypes = Object.keys(EntityTypes)
    .filter(function (type) {
      return type.toLowerCase().indexOf(query.toLowerCase()) === 0 ||
        EntityTypes[type].label.toLowerCase().indexOf(query.toLowerCase()) === 0;
    })
    .map(function (type) {
      return { type: type, name: EntityTypes[type].label, icon: EntityTypes[type].icon, isTypeHint: true };
    });

  // A bare `@calculator` is a project reference, not only an entity type.
  // Search the user's Vibe projects alongside the type hints so the selected
  // project id travels with the message and the LLM never guesses a workspace.
  fetchProjectMentions(query, filteredTypes);
}

function normalizeListResponse(data) {
  if (Array.isArray(data)) return data;
  if (data && Array.isArray(data.results)) return data.results;
  if (data && Array.isArray(data.items)) return data.items;
  return [];
}

function fetchEntitiesOfType(type, searchTerm) {
  if (type === "integration") {
    fetchIntegrationMentions(searchTerm);
    return;
  }
  if (type === "project") {
    fetchProjectMentions(searchTerm, []);
    return;
  }
  fetch("/api/search/entities?type=" + encodeURIComponent(type) + "&q=" + encodeURIComponent(searchTerm || ""))
    .then(function (r) { return r.json(); })
    .then(function (data) {
      var results = normalizeListResponse(data).map(function (item) {
        return {
          type: type, name: item.name || item.title || item.number,
          id: item.id, icon: EntityTypes[type].icon,
          subtitle: item.subtitle || item.status || "", isTypeHint: false,
        };
      });
      if (results.length === 0) {
        results = [{ type: type, name: "No results for '" + searchTerm + "'", icon: "\u274C", isTypeHint: false, disabled: true }];
      }
      renderMentionResults(results);
    })
    .catch(function () {
      renderMentionResults([{ type: type, name: "Search unavailable", icon: "\u26A0\uFE0F", isTypeHint: false, disabled: true }]);
    });
}

function fetchProjectMentions(searchTerm, leadingResults) {
  var term = String(searchTerm || "").toLowerCase();
  fetch("/api/vibe/projects?limit=50")
    .then(function (r) { return r.json(); })
    .then(function (data) {
      var projects = normalizeListResponse(data && data.projects ? data.projects : data);
      ChatState.projectCatalog = projects;
      var results = projects
        .filter(function (project) {
          var name = String(project.name || "").toLowerCase();
          return !term || name.indexOf(term) !== -1;
        })
        .map(function (project) {
          return {
            type: "project",
            name: project.name || "Unnamed project",
            id: project.id || project.project_id,
            icon: EntityTypes.project.icon,
            subtitle: project.status || "Vibe project",
            isTypeHint: false,
          };
        });
      renderMentionResults((leadingResults || []).concat(results));
    })
    .catch(function () {
      renderMentionResults(leadingResults || []);
    });
}

function fetchIntegrationMentions(searchTerm) {
  var term = searchTerm || "";
  fetch("/api/apps/integrations/mentions?q=" + encodeURIComponent(term))
    .then(function (r) { return r.json(); })
    .then(function (data) {
      var results = normalizeListResponse(data).map(function (item) {
        return {
          type: "integration",
          name: item.label || item.name || item.provider,
          id: item.id,
          icon: EntityTypes.integration.icon,
          subtitle: item.provider || "",
          details: item.provider,
          isTypeHint: false,
        };
      });
      if (results.length === 0) {
        results = [{ type: "integration", name: "No connected integrations for '" + term + "'", icon: "\u274C", isTypeHint: false, disabled: true }];
      }
      renderMentionResults(results);
    })
    .catch(function () {
      renderMentionResults([{ type: "integration", name: "Search unavailable", icon: "\u26A0\uFE0F", isTypeHint: false, disabled: true }]);
    });
}

function renderMentionResults(results) {
  var container = document.getElementById("mentionResults");
  if (!container) return;

  ChatState.mentionState.results = results;
  ChatState.mentionState.selectedIndex = 0;

  container.innerHTML = results.map(function (item, index) {
    var classes = "mention-item";
    if (index === ChatState.mentionState.selectedIndex) classes += " selected";
    if (item.disabled) classes += " disabled";
    var subtitle = item.subtitle ? '<span class="mention-item-subtitle">' + escapeHtml(item.subtitle) + "</span>" : "";
    var hint = item.isTypeHint ? '<span class="mention-item-hint">Type : to search</span>' : "";
    return '<div class="' + classes + '" data-index="' + index + '" data-type="' + item.type +
      '" data-name="' + escapeHtml(item.name) + '" data-is-type="' + item.isTypeHint + '">' +
      '<span class="mention-item-icon">' + item.icon + "</span>" +
      '<span class="mention-item-content">' +
      '<span class="mention-item-name">' + escapeHtml(item.name) + "</span>" +
      subtitle + hint + "</span></div>";
  }).join("");

  container.querySelectorAll(".mention-item:not(.disabled)").forEach(function (item) {
    item.addEventListener("click", function () {
      selectMentionItem(parseInt(this.getAttribute("data-index")));
    });
  });
}

function selectMentionItem(index) {
  var item = ChatState.mentionState.results[index];
  if (!item || item.disabled) return;

  var input = document.getElementById("messageInput");
  if (!input) return;

  var value = input.value;
  var beforeMention = value.substring(0, ChatState.mentionState.startPos);
  var afterMention = value.substring(input.selectionStart);

  var insertText;
  if (item.isTypeHint) {
    insertText = "@" + item.type + ":";
    ChatState.mentionState.query = item.type + ":";
    ChatState.mentionState.startPos = beforeMention.length;
    input.value = beforeMention + insertText + afterMention;
    input.setSelectionRange(beforeMention.length + insertText.length, beforeMention.length + insertText.length);
    searchEntities(ChatState.mentionState.query);
    return;
  } else {
    // Projects use the concise `@calculator` form requested for chat. The
    // selectedMentions record remains canonical and carries the UUID.
    insertText = item.type === "project"
      ? "@" + item.name + " "
      : "@" + item.type + ":" + item.name + " ";
    input.value = beforeMention + insertText + afterMention;
    input.setSelectionRange(beforeMention.length + insertText.length, beforeMention.length + insertText.length);
    hideMentionDropdown();
    // Remember the selection so sendMessage can attach structured mentions
    // (#939): only integrations carry the connection id - other entity kinds
    // keep an empty id to avoid changing their behavior.
    if (!ChatState.selectedMentions) ChatState.selectedMentions = [];
    ChatState.selectedMentions.push({
      kind: item.type,
      id: item.id ? String(item.id) : "",
      name: item.name,
      project_id: item.type === "project" && item.id ? String(item.id) : null,
    });
  }
  input.focus();
}

function updateMentionSelection(direction) {
  var enabledResults = ChatState.mentionState.results.filter(function (r) { return !r.disabled; });
  if (enabledResults.length === 0) return;

  var currentEnabled = 0;
  for (var i = 0; i < ChatState.mentionState.selectedIndex; i++) {
    if (!ChatState.mentionState.results[i].disabled) currentEnabled++;
  }

  currentEnabled += direction;
  if (currentEnabled < 0) currentEnabled = enabledResults.length - 1;
  if (currentEnabled >= enabledResults.length) currentEnabled = 0;

  var newIndex = 0;
  var count = 0;
  for (var j = 0; j < ChatState.mentionState.results.length; j++) {
    if (!ChatState.mentionState.results[j].disabled) {
      if (count === currentEnabled) { newIndex = j; break; }
      count++;
    }
  }

  ChatState.mentionState.selectedIndex = newIndex;
  var items = document.querySelectorAll("#mentionResults .mention-item");
  items.forEach(function (item, idx) { item.classList.toggle("selected", idx === newIndex); });

  var selectedItem = document.querySelector("#mentionResults .mention-item.selected");
  if (selectedItem) selectedItem.scrollIntoView({ block: "nearest" });
}

function handleMentionInput(e) {
  var input = e.target;
  var value = input.value;
  var cursorPos = input.selectionStart;
  var textBeforeCursor = value.substring(0, cursorPos);
  var atIndex = textBeforeCursor.lastIndexOf("@");

  if (atIndex >= 0) {
    var charBeforeAt = atIndex > 0 ? textBeforeCursor[atIndex - 1] : " ";
    if (charBeforeAt === " " || atIndex === 0) {
      var query = textBeforeCursor.substring(atIndex + 1);
      if (!query.includes(" ")) {
        ChatState.mentionState.active = true;
        ChatState.mentionState.startPos = atIndex;
        ChatState.mentionState.query = query;
        showMentionDropdown();
        searchEntities(query);
        return;
      }
    }
  }
  if (ChatState.mentionState.active) hideMentionDropdown();
}

function handleMentionKeydown(e) {
  if (!ChatState.mentionState.active) return false;
  if (e.key === "ArrowDown") { e.preventDefault(); updateMentionSelection(1); return true; }
  if (e.key === "ArrowUp") { e.preventDefault(); updateMentionSelection(-1); return true; }
  if (e.key === "Enter" || e.key === "Tab") { e.preventDefault(); selectMentionItem(ChatState.mentionState.selectedIndex); return true; }
  if (e.key === "Escape") { e.preventDefault(); hideMentionDropdown(); return true; }
  return false;
}

function setupMentionClickHandlers(container) {
  var mentions = container.querySelectorAll(".mention-tag");
  mentions.forEach(function (mention) {
    mention.addEventListener("click", function (e) {
      e.preventDefault();
      var type = this.getAttribute("data-type");
      var name = this.getAttribute("data-name");
      navigateToEntity(type, name);
    });
    mention.addEventListener("mouseenter", function () {
      var type = this.getAttribute("data-type");
      var name = this.getAttribute("data-name");
      showEntityCard(type, name, e.target);
    });
    mention.addEventListener("mouseleave", function () { hideEntityCard(); });
  });
}

function navigateToEntity(type, name) {
  var entityType = EntityTypes[type.toLowerCase()];
  if (entityType) {
    var route = entityType.route;
    window.location.hash = "#" + route;
    var htmxLink = document.querySelector('a[data-section="' + route + '"]');
    if (htmxLink) htmx.trigger(htmxLink, "click");
  }
}

function showEntityCard(type, name, targetEl) {
  var card = document.getElementById("entityCardTooltip");
  var entityType = EntityTypes[type.toLowerCase()];
  if (!card || !entityType) return;

  card.querySelector(".entity-card-type").textContent = entityType.label;
  card.querySelector(".entity-card-type").style.background = entityType.color;
  card.querySelector(".entity-card-title").textContent = entityType.icon + " " + name;
  card.querySelector(".entity-card-status").textContent = "";
  card.querySelector(".entity-card-details").textContent = "Loading...";
  card.dataset.entityType = type;
  card.dataset.entityName = name;

  var rect = targetEl.getBoundingClientRect();
  card.style.left = rect.left + "px";
  card.style.top = rect.top - card.offsetHeight - 8 + "px";
  card.classList.add("visible");

  fetchEntityDetails(type, name).then(function (details) {
    if (card.classList.contains("visible")) {
      card.querySelector(".entity-card-details").innerHTML = details;
    }
  });
}

function hideEntityCard() {
  var card = document.getElementById("entityCardTooltip");
  if (card) card.classList.remove("visible");
}

function fetchEntityDetails(type, name) {
return fetch("/api/search/entity?type=" + encodeURIComponent(type) + "&name=" + encodeURIComponent(name))
.then(function (r) { return r.json(); })
.then(function (data) { return data && data.details ? data.details : "No additional details available"; })
.catch(function () { return "Unable to load details"; });
}

// Bind the entity-card "View" button to navigate to the referenced entity.
if (typeof window !== 'undefined' && !window.__entityCardViewBound) {
  window.__entityCardViewBound = true;
  document.addEventListener("click", function (e) {
    var btn = e.target.closest && e.target.closest(".entity-card-btn[data-action='view']");
    if (!btn) return;
    var card = document.getElementById("entityCardTooltip");
    if (!card || !card.classList.contains("visible")) return;
    var type = card.dataset.entityType;
    var name = card.dataset.entityName;
    if (type && name) navigateToEntity(type, name);
    hideEntityCard();
  });
}

// Export functions to global scope for use in chat-init.js
if (typeof window !== 'undefined') {
window.handleMentionInput = handleMentionInput;
window.handleMentionKeydown = handleMentionKeydown;
window.hideMentionDropdown = hideMentionDropdown;
}
