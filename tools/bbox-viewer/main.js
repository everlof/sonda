import * as pdfjsLib from "https://cdn.jsdelivr.net/npm/pdfjs-dist@4.6.82/build/pdf.mjs";

pdfjsLib.GlobalWorkerOptions.workerSrc =
  "https://cdn.jsdelivr.net/npm/pdfjs-dist@4.6.82/build/pdf.worker.mjs";

const state = {
  pdfDoc: null,
  result: null,
  entries: [],
  substanceDecisions: [],
  contributorEntryIds: new Set(),
  contributorReasonsByEntryId: new Map(),
  filteredEntries: [],
  selectedEntryId: null,
  currentPage: 1,
  currentPageProxy: null,
  currentViewport: null,
  renderTask: null,
  renderNonce: 0,
  scale: 1.35,
};

const els = {
  pdfInput: document.getElementById("pdfInput"),
  jsonInput: document.getElementById("jsonInput"),
  searchInput: document.getElementById("searchInput"),
  topLeftToggle: document.getElementById("topLeftToggle"),
  prevPageBtn: document.getElementById("prevPageBtn"),
  nextPageBtn: document.getElementById("nextPageBtn"),
  pageLabel: document.getElementById("pageLabel"),
  entryLabel: document.getElementById("entryLabel"),
  stats: document.getElementById("stats"),
  entriesList: document.getElementById("entriesList"),
  canvas: document.getElementById("pdfCanvas"),
  overlay: document.getElementById("overlay"),
  hoverInfo: document.getElementById("hoverInfo"),
  entryDetails: document.getElementById("entryDetails"),
};

els.pdfInput.addEventListener("change", onPdfChange);
els.jsonInput.addEventListener("change", onJsonChange);
els.searchInput.addEventListener("input", renderEntries);
els.topLeftToggle.addEventListener("change", () => redrawHighlightsOnly());
els.prevPageBtn.addEventListener("click", () => renderPage(state.currentPage - 1));
els.nextPageBtn.addEventListener("click", () => renderPage(state.currentPage + 1));

async function onPdfChange(e) {
  const file = e.target.files?.[0];
  if (!file) return;
  const buffer = await file.arrayBuffer();
  const task = pdfjsLib.getDocument({ data: buffer });
  state.pdfDoc = await task.promise;
  state.currentPage = 1;
  await renderPage(state.currentPage);
}

async function onJsonChange(e) {
  const file = e.target.files?.[0];
  if (!file) return;
  const text = await file.text();
  const parsed = JSON.parse(text);
  let entries = parsed?.trace?.entries;
  let usedFallback = false;
  if (!Array.isArray(entries)) {
    entries = buildFallbackEntries(parsed);
    usedFallback = true;
  }

  state.result = parsed;
  state.entries = entries;
  state.substanceDecisions = Array.isArray(parsed?.trace?.decisions)
    ? parsed.trace.decisions.filter((d) => d?.target === "substance")
    : [];
  const contributionMeta = buildContributionMeta(parsed, entries);
  state.contributorEntryIds = contributionMeta.ids;
  state.contributorReasonsByEntryId = contributionMeta.reasonsByEntryId;
  state.filteredEntries = [...entries];
  state.selectedEntryId = null;
  renderEntries();
  els.entryDetails.textContent = "";
  if (usedFallback) {
    els.stats.textContent =
      "trace.entries missing; using fallback from samples (no bbox highlighting)";
  }

  // If a PDF is already open, refresh overlays immediately so hover works
  // even before any left-pane selection.
  if (state.pdfDoc) {
    await renderPage(state.currentPage || 1);
  }
}

function renderEntries() {
  const filter = els.searchInput.value.trim().toLowerCase();
  state.filteredEntries = state.entries.filter((entry) => {
    const text = `${entry.raw_name} ${entry.normalized_name} ${entry.sample_id}`.toLowerCase();
    return !filter || text.includes(filter);
  });

  els.stats.textContent = `${state.filteredEntries.length} / ${state.entries.length} entries`;
  els.entriesList.innerHTML = "";

  state.filteredEntries.forEach((entry, idx) => {
    const li = document.createElement("li");
    li.dataset.entryId = entry.entry_id;
    if (state.contributorEntryIds.has(entry.entry_id)) {
      li.classList.add("contributor");
    }
    if (entry.entry_id === state.selectedEntryId) {
      li.classList.add("active");
    }
    const contributorTag = state.contributorEntryIds.has(entry.entry_id)
      ? '<span class="tag">Contributed</span>'
      : "";
    const contributorWhy = state.contributorReasonsByEntryId.get(entry.entry_id) || [];
    const contributorWhyLine =
      contributorWhy.length > 0
        ? `<div class="row-why">${escapeHtml(contributorWhy[0])}</div>`
        : "";
    li.innerHTML = `
      <div class="row-main">${escapeHtml(entry.raw_name)} ${contributorTag}</div>
      <div class="row-sub">${escapeHtml(entry.sample_id)} · spans: ${entry.evidence_spans?.length ?? 0}</div>
      ${contributorWhyLine}
    `;
    li.addEventListener("click", async () => {
      state.selectedEntryId = entry.entry_id;
      const firstSpan = entry.evidence_spans?.[0];
      if (firstSpan?.page_number) {
        if (firstSpan.page_number !== state.currentPage) {
          await renderPage(firstSpan.page_number);
        } else {
          redrawHighlightsOnly();
        }
      } else {
        redrawHighlightsOnly();
      }
      renderEntries();
      scrollEntryIntoView(entry.entry_id);
      renderEntryDetails(entry);
    });
    els.entriesList.appendChild(li);
  });
}

async function renderPage(pageNumber) {
  if (!state.pdfDoc) {
    els.pageLabel.textContent = "Page 0 / 0";
    return;
  }

  if (pageNumber < 1 || pageNumber > state.pdfDoc.numPages) {
    return;
  }

  const nonce = ++state.renderNonce;
  state.currentPage = pageNumber;
  const page = await state.pdfDoc.getPage(pageNumber);
  if (nonce !== state.renderNonce) return;

  const viewport = page.getViewport({ scale: state.scale });

  const canvas = els.canvas;
  const ctx = canvas.getContext("2d");
  canvas.width = viewport.width;
  canvas.height = viewport.height;

  if (state.renderTask) {
    try {
      state.renderTask.cancel();
    } catch (_e) {
      // ignore
    }
  }

  const renderTask = page.render({ canvasContext: ctx, viewport });
  state.renderTask = renderTask;

  try {
    await renderTask.promise;
  } catch (_e) {
    return;
  }
  if (nonce !== state.renderNonce) return;

  state.currentPageProxy = page;
  state.currentViewport = viewport;
  els.overlay.style.width = `${viewport.width}px`;
  els.overlay.style.height = `${viewport.height}px`;
  drawHighlights(pageNumber, page);
  els.pageLabel.textContent = `Page ${pageNumber} / ${state.pdfDoc.numPages}`;
}

function drawHighlights(pageNumber, page) {
  els.overlay.innerHTML = "";
  hideSpanInfo();
  const selectedEntry = getSelectedEntry();
  const hotspots = buildPageHotspots(pageNumber);

  const [x1, y1, x2, y2] = page.view;
  const pdfHeight = y2 - y1;
  const topLeftMode = els.topLeftToggle.checked;

  const selectedKeys = new Set(
    (selectedEntry?.evidence_spans || [])
      .filter((s) => s.page_number === pageNumber)
      .map((s) => spanKey(s))
  );

  if (selectedEntry) {
    els.entryLabel.textContent = `${selectedEntry.raw_name} (${selectedKeys.size} spans selected on page)`;
  } else {
    els.entryLabel.textContent = `${hotspots.length} hotspots on page`;
  }

  hotspots.forEach((spot) => {
    const span = spot.span;
    const box = document.createElement("div");
    box.className = selectedKeys.has(spot.key) ? "hotspot selected" : "hotspot";
    const x = span.x_min * state.scale;
    const width = Math.max((span.x_max - span.x_min) * state.scale, 1);
    const height = Math.max((span.y_max - span.y_min) * state.scale, 1);
    const y = topLeftMode
      ? span.y_min * state.scale
      : (pdfHeight - span.y_max) * state.scale;

    box.style.left = `${x}px`;
    box.style.top = `${y}px`;
    box.style.width = `${width}px`;
    box.style.height = `${height}px`;
    box.addEventListener("click", async (evt) => {
      evt.preventDefault();
      evt.stopPropagation();
      const pick =
        spot.entries.find((e) => e.entry_id === state.selectedEntryId) || spot.entries[0];
      if (!pick) return;
      state.selectedEntryId = pick.entry_id;
      renderEntries();
      renderEntryDetails(pick);
      redrawHighlightsOnly();
      scrollEntryIntoView(pick.entry_id);
    });
    box.addEventListener("mouseenter", (evt) => {
      showSpotInfo(spot, evt, selectedEntry);
    });
    box.addEventListener("mousemove", (evt) => {
      moveSpanInfo(evt);
    });
    box.addEventListener("mouseleave", () => {
      hideSpanInfo();
    });
    els.overlay.appendChild(box);
  });
}

function renderEntryDetails(entry) {
  els.entryDetails.textContent = JSON.stringify(entry, null, 2);
}

function getSelectedEntry() {
  if (!state.selectedEntryId) return null;
  return state.entries.find((e) => e.entry_id === state.selectedEntryId) || null;
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function showSpotInfo(spot, evt, selectedEntry) {
  const span = spot.span;
  const entry =
    (selectedEntry && spot.entries.find((e) => e.entry_id === selectedEntry.entry_id)) ||
    spot.entries[0];
  if (!entry) return;

  const decisions = state.substanceDecisions.filter(
    (d) => d.sample_id === entry.sample_id && d.substance === entry.normalized_name
  );

  const decisionLines =
    decisions.length > 0
      ? decisions
          .map((d) => `${d.ruleset_name}: ${d.category}`)
          .slice(0, 3)
          .join("\n")
      : "No decision trace";
  const compareText =
    decisions[0]?.reason || entry.reason || "No comparison details in trace";
  const contribReasons = state.contributorReasonsByEntryId.get(entry.entry_id) || [];
  const contribText =
    contribReasons.length > 0 ? contribReasons.slice(0, 3).join("\n") : "Not a determining entry";

  const info = [
    `Substance: ${entry.raw_name}`,
    `Value: ${entry.raw_value} ${entry.unit || ""}`.trim(),
    `Matched line: ${span.matched_text}`,
    `Classification:`,
    decisionLines,
    `Contribution:`,
    contribText,
    `Compared against: ${compareText}`,
    `Click box to select in left panel`,
  ].join("\n");

  els.hoverInfo.textContent = info;
  els.hoverInfo.hidden = false;
  moveSpanInfo(evt);
}

function moveSpanInfo(evt) {
  if (els.hoverInfo.hidden) return;
  const wrap = document.getElementById("canvasWrap");
  const rect = wrap.getBoundingClientRect();
  const left = Math.min(evt.clientX - rect.left + 14, rect.width - 360);
  const top = Math.min(evt.clientY - rect.top + 14, rect.height - 120);
  els.hoverInfo.style.left = `${Math.max(left, 8)}px`;
  els.hoverInfo.style.top = `${Math.max(top, 8)}px`;
}

function hideSpanInfo() {
  els.hoverInfo.hidden = true;
}

function redrawHighlightsOnly() {
  if (!state.currentPageProxy) return;
  drawHighlights(state.currentPage, state.currentPageProxy);
}

function scrollEntryIntoView(entryId) {
  const el = els.entriesList.querySelector(`li[data-entry-id="${CSS.escape(entryId)}"]`);
  if (!el) return;
  el.scrollIntoView({ block: "nearest", behavior: "smooth" });
}

function spanKey(span) {
  return [
    span.page_number,
    span.line_index,
    span.x_min,
    span.y_min,
    span.x_max,
    span.y_max,
  ].join(":");
}

function buildPageHotspots(pageNumber) {
  const byKey = new Map();

  for (const entry of state.entries) {
    const spans = Array.isArray(entry.evidence_spans) ? entry.evidence_spans : [];
    for (const span of spans) {
      if (span.page_number !== pageNumber) continue;
      const key = spanKey(span);
      if (!byKey.has(key)) {
        byKey.set(key, { key, span, entries: [] });
      }
      byKey.get(key).entries.push(entry);
    }
  }

  return Array.from(byKey.values());
}

function buildFallbackEntries(parsed) {
  const out = [];
  const samples = parsed?.samples;
  if (!Array.isArray(samples)) return out;

  for (const sample of samples) {
    const sampleId = sample?.sample_id || "unknown";
    const rulesets = Array.isArray(sample?.ruleset_results)
      ? sample.ruleset_results
      : [];

    for (const rs of rulesets) {
      const subs = Array.isArray(rs?.substance_results) ? rs.substance_results : [];
      for (const sr of subs) {
        out.push({
          sample_id: sampleId,
          raw_name: sr.raw_name || sr.substance || "unknown",
          normalized_name: sr.substance || "unknown",
          evidence_spans: [],
          reason: sr.reason || "",
        });
      }
    }
  }
  return out;
}

function buildContributionMeta(parsed, entries) {
  const ids = new Set();
  const reasonsByEntryId = new Map();
  for (const entry of entries) {
    reasonsByEntryId.set(entry.entry_id, []);
  }
  const samples = Array.isArray(parsed?.samples) ? parsed.samples : [];

  for (const sample of samples) {
    const sampleId = sample?.sample_id;
    const rsList = Array.isArray(sample?.ruleset_results) ? sample.ruleset_results : [];

    for (const rs of rsList) {
      const hp = rs?.hp_details;
      const isHpRuleset = !!hp;

      if (!isHpRuleset) {
        const overall = String(rs?.overall_category || "");
        const lowest = String(rs?.lowest_category || "");
        const worsened =
          overall.startsWith("> ") || (lowest.length > 0 && overall !== lowest);

        // If overall class is at the cleanest level, entries are evaluated but not contributing.
        if (!worsened) {
          continue;
        }

        const determining = new Set(
          (rs?.determining_substances || []).map((d) => String(d).toLowerCase())
        );
        const substanceResults = Array.isArray(rs?.substance_results) ? rs.substance_results : [];
        for (const sr of substanceResults) {
          const raw = String(sr?.raw_name || "").toLowerCase();
          const norm = String(sr?.substance || "").toLowerCase();
          if (!determining.has(raw) && !determining.has(norm)) {
            continue;
          }
          const reasonText = `${rs.ruleset_name}: drove overall ${overall} — ${shorten(sr.reason, 140)}`;
          addContributionReason(entries, reasonsByEntryId, sampleId, raw, norm, reasonText);
        }
      }

      // Also mark explicit triggered HP contributors if present.
      if (hp?.is_hazardous && hp?.criteria_results) {
        for (const cr of hp.criteria_results) {
          if (!cr?.triggered) continue;
          for (const c of cr?.contributions || []) {
            if (c?.triggers && c?.substance) {
              const norm = String(c.substance).toLowerCase();
              const threshold = c.threshold_pct ? ` >= ${c.threshold_pct}%` : "";
              const reasonText = `${rs.ruleset_name}/${cr.hp_id}: ${c.compound} ${c.h_code} ${c.concentration_pct}%${threshold}`;
              addContributionReason(entries, reasonsByEntryId, sampleId, null, norm, reasonText);
            }
          }
        }
      }
    }
  }

  for (const [entryId, reasons] of reasonsByEntryId.entries()) {
    if (reasons.length > 0) {
      ids.add(entryId);
      reasonsByEntryId.set(entryId, [...new Set(reasons)]);
    }
  }

  return { ids, reasonsByEntryId };
}

function addContributionReason(entries, reasonsByEntryId, sampleId, rawNameLower, normNameLower, reasonText) {
  for (const entry of entries) {
    if (entry.sample_id !== sampleId) continue;
    const eraw = String(entry.raw_name || "").toLowerCase();
    const enorm = String(entry.normalized_name || "").toLowerCase();
    const rawMatch = rawNameLower ? eraw === rawNameLower : false;
    const normMatch = normNameLower ? enorm === normNameLower : false;
    if (!rawMatch && !normMatch) continue;
    reasonsByEntryId.get(entry.entry_id).push(reasonText);
  }
}

function shorten(s, maxLen) {
  if (!s) return "";
  if (s.length <= maxLen) return s;
  return `${s.slice(0, maxLen - 1)}…`;
}
