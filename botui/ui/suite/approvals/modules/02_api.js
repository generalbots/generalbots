"use strict";
(function () {
  var A = window.GBApprovals;

  function authHeaders() {
    if (window.GBShared && window.GBShared.auth && window.GBShared.auth.headers) {
      return window.GBShared.auth.headers(null, { management: true });
    }
    var token =
      (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
      localStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      sessionStorage.getItem("gb-access-token") ||
      "";
    return token ? { Authorization: "Bearer " + token } : {};
  }

  function request(path, method, body) {
    var headers = { "Content-Type": "application/json" };
    var auth = authHeaders();
    Object.keys(auth).forEach(function (key) { headers[key] = auth[key]; });
    return fetch(path, {
      method: method || "GET",
      headers: headers,
      body: body === undefined ? undefined : JSON.stringify(body)
    }).then(function (response) {
      return response.text().then(function (text) {
        var payload = {};
        try { payload = text ? JSON.parse(text) : {}; } catch (err) { payload = { error: text }; }
        if (!response.ok) {
          var message = payload.error || payload.message || ("HTTP " + response.status);
          return Promise.reject(new Error(String(message)));
        }
        return payload;
      });
    });
  }

  function createApproval(input) { return request("/api/approvals", "POST", input); }
  function listApprovals() { return request("/api/approvals", "GET"); }
  function getApproval(id) { return request("/api/approvals/" + encodeURIComponent(id), "GET"); }
  function routeApproval(id, input) { return request("/api/approvals/" + id + "/route", "POST", input); }
  function delegate(id, input) { return request("/api/approvals/" + id + "/delegate", "POST", input); }
  function decide(id, input) { return request("/api/approvals/" + id + "/decide", "POST", input); }
  function sign(id, input) { return request("/api/approvals/" + id + "/sign", "POST", input); }
  function updateContent(id, input) { return request("/api/approvals/" + id + "/update-content", "POST", input); }
  function checkSla(id, input) { return request("/api/approvals/" + id + "/check-sla", "POST", input); }
  function slaSweep(input) { return request("/api/approvals/sla-sweep", "POST", input); }
  function publish(id) { return request("/api/approvals/" + id + "/publish", "POST", {}); }
  function verify(id) { return request("/api/approvals/" + id + "/verify", "GET"); }

  A.api = {
    createApproval: createApproval,
    listApprovals: listApprovals,
    getApproval: getApproval,
    routeApproval: routeApproval,
    delegate: delegate,
    decide: decide,
    sign: sign,
    updateContent: updateContent,
    checkSla: checkSla,
    slaSweep: slaSweep,
    publish: publish,
    verify: verify
  };
})();
