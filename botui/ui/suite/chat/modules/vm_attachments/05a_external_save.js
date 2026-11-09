"use strict";

/* #2047 — VM output save with a target menu.
 *
 * The attachment cards and the output folder used to have a single "Save"
 * button (General Bots Drive). This adds the same menu the artifact panel
 * uses: internal Drive plus one entry per connected provider that reports
 * `can_write`. Saving to a provider streams the workspace file through the
 * signed VM endpoint, encodes it, and posts to the upload route; the returned
 * `web_view_link` is what the confirmation names.
 *
 * Split from 04_bind.js/05_folder.js so those stay small; globals only.
 */

function vmSaveFilesToExternal(provider, conversationId, files) {
  var list = (files || []).filter(function (name) {
    return !!name;
  });
  if (!window.GBExternalSave) {
    return Promise.reject(new Error("external drive saving is unavailable in this build"));
  }
  if (!list.length) {
    return Promise.reject(new Error("no file selected"));
  }
  return list
    .reduce(function (chain, relative) {
      return chain.then(function (saved) {
        var path = relative.indexOf("/") === -1 ? "output/" + relative : relative;
        var name = path.split("/").pop();
        return GBExternalSave.fetchWorkspaceFile(conversationId, path)
          .then(function (buffer) {
            return GBExternalSave.upload(provider, {
              name: name,
              mimeType: GBExternalSave.mimeFor(name),
              base64: GBExternalSave.base64(buffer),
            });
          })
          .then(function (result) {
            saved.push(result);
            return saved;
          });
      });
    }, Promise.resolve([]))
    .then(function (saved) {
      GBExternalSave.report(provider, saved);
      return saved;
    });
}

/* Resolves the destination and performs the save. `files` are workspace paths
 * (`output/x.xlsx`) or bare names; an empty list means "the whole folder" and
 * is only valid with an explicit list from the caller. */
function vmSaveWithMenu(anchor, conversationId, files) {
  var chooser =
    window.GBExternalSave && GBExternalSave.choose
      ? GBExternalSave.choose(anchor)
      : Promise.resolve("internal");
  return chooser.then(function (target) {
    if (!target) return null;
    if (target === "internal") {
      return vmSaveToDrive(conversationId, files).then(vmReportSaved);
    }
    return vmSaveFilesToExternal(target, conversationId, files);
  });
}

/* Names behind the output folder, used by the folder-level Save control. */
function vmFolderFileNames(folder) {
  var names = [];
  if (!folder) return names;
  var cards = folder.querySelectorAll(".gb-attachment[data-path]");
  for (var i = 0; i < cards.length; i++) {
    var path = cards[i].dataset.path || "";
    var name = path.split("/").pop();
    if (name) names.push("output/" + name);
  }
  return names;
}
