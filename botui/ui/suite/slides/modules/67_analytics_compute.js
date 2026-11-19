"use strict";
/* Slides analytics (#2489) — layer 2, pure DOM-free computation over the
 * simulated event stream. No network, no persistence, no user-facing literals:
 * every label arrives through the locale catalogues via A.tr(). */
(function (window) {
  var A = window.GBSlidesAnalytics = window.GBSlidesAnalytics || {};

  function countBy(items, keyOf, buckets) {
    var out = {};
    (buckets || []).forEach(function (name) { out[name] = 0; });
    items.forEach(function (item) {
      var name = keyOf(item);
      out[name] = (out[name] || 0) + 1;
    });
    return out;
  }

  A.compute = function (state) {
    var data = state || A.fresh();
    var events = Array.isArray(data.events) ? data.events : [];
    var viewers = Array.isArray(data.viewers) ? data.viewers : [];
    var opens = events.filter(function (event) { return event.type === "open"; });
    var timestamps = opens.map(function (event) { return A.num(event.ts, 0); });
    var first = timestamps.length ? Math.min.apply(null, timestamps) : 0;
    var last = timestamps.length ? Math.max.apply(null, timestamps) : 0;
    var slideCount = Math.max(1, A.num(data.slideCount, A.SLIDE_COUNT));
    var perSlide = [];
    for (var index = 0; index < slideCount; index += 1) {
      var entered = events.filter(function (event) {
        return event.type === "enter" && A.num(event.slide, -1) === index;
      });
      var dwellRows = events.filter(function (event) {
        return event.type === "dwell" && A.num(event.slide, -1) === index;
      });
      var dwellMs = dwellRows.reduce(function (sum, event) {
        return sum + A.num(event.duration, 0);
      }, 0);
      var exits = events.filter(function (event) {
        return event.type === "leave" && A.num(event.slide, -1) === index;
      }).length;
      perSlide.push({
        slide: index,
        entered: entered.length,
        dwellMs: dwellMs,
        avgDwellMs: entered.length ? Math.round(dwellMs / entered.length) : 0,
        exits: exits
      });
    }
    /* Reaching the final slide is completion, not a drop-off: only earlier
     * slides are candidates so the report points at a real attention loss. */
    var dropOffSlide = 0;
    var dropOffCount = 0;
    perSlide.slice(0, -1).forEach(function (row) {
      if (row.exits > dropOffCount) { dropOffCount = row.exits; dropOffSlide = row.slide; }
    });
    var completed = viewers.filter(function (viewer) { return !!viewer.completed; }).length;
    var unique = viewers.length || opens.reduce(function (ids, event) {
      if (ids.indexOf(event.viewerId) < 0) ids.push(event.viewerId);
      return ids;
    }, []).length;
    return {
      views: opens.length,
      uniqueViewers: unique,
      firstView: first ? A.dateLabel(first) : "",
      lastView: last ? A.dateLabel(last) : "",
      slideCount: slideCount,
      perSlide: perSlide,
      completedViewers: completed,
      completionRate: unique ? Math.round((completed / unique) * 10000) / 100 : 0,
      dropOffSlide: dropOffCount ? dropOffSlide : 0,
      dropOffCount: dropOffCount,
      deviceBuckets: countBy(viewers, function (viewer) {
        return A.bucketDevice(viewer.deviceRaw);
      }, A.DEVICE_BUCKETS),
      geoBuckets: countBy(viewers, function (viewer) {
        return A.bucketGeo(viewer.geoRaw);
      }, A.GEO_BUCKETS)
    };
  };

  function cell(value) {
    var text = value === null || value === undefined ? "" : String(value);
    return "\"" + text.split("\"").join("\"\"") + "\"";
  }

  function row(values) {
    return values.map(cell).join(",") + "\n";
  }

  A.csv = {};

  A.csv.summary = function (report) {
    var data = report || A.compute(A.state || A.fresh());
    var out = row(["slide", "entered", "dwell_ms", "avg_dwell_ms", "exits"]);
    data.perSlide.forEach(function (item) {
      out += row([item.slide + 1, item.entered, item.dwellMs, item.avgDwellMs, item.exits]);
    });
    return out;
  };

  A.csv.viewers = function (state, anonymise) {
    var data = state || A.fresh();
    var strip = anonymise === undefined ? !!(data.options && data.options.anonymise) : !!anonymise;
    var viewers = Array.isArray(data.viewers) ? data.viewers : [];
    var out;
    if (strip) {
      out = row(["pseudonym", "device_bucket", "geo_bucket", "slides_seen", "dwell_ms", "completed"]);
      viewers.forEach(function (viewer) {
        out += row([A.pseudonym(viewer.id), A.bucketDevice(viewer.deviceRaw),
          A.bucketGeo(viewer.geoRaw), viewer.slidesSeen, viewer.dwellMs, !!viewer.completed]);
      });
    } else {
      out = row(["viewer_id", "email", "name", "device", "geo", "slides_seen", "dwell_ms", "completed"]);
      viewers.forEach(function (viewer) {
        out += row([viewer.id, viewer.email, viewer.name, viewer.deviceRaw, viewer.geoRaw,
          viewer.slidesSeen, viewer.dwellMs, !!viewer.completed]);
      });
    }
    return out;
  };

  A.prune = function (state, now) {
    var cutoff = A.num(now, Date.now()) - A.num(state.options && state.options.retentionDays, 90) * 86400000;
    var before = state.events.length;
    state.events = state.events.filter(function (event) { return A.num(event.ts, 0) >= cutoff; });
    return before - state.events.length;
  };
})(typeof window !== "undefined" ? window : globalThis);
