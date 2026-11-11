"use strict";
/* #2174/#2175/#2176/#2177 — per-app action fidelity matrix.
   Single source of truth for which channels each app supports:
     ops   = gb-doc-delta/v1 ops the app renders (VM streaming)
     steps = __ui_plan__ step types the app honors (ui hints)
     api   = command-catalog entries (or [])
     fidelity = full | partial | progress | finalize-only | none
   Prompt builders and the plan executor read this; the LLM cardapio is
   generated from `api` + the granted VM docstrings, never from raw ops. */
window.APP_CAPABILITIES = {
  docs:   { ops:["append_block","update_block","delete_block","format_runs","set_table_rows","set_title","set_page_layout","finalize","error","ack","clear"], steps:["open","click","fill","menu","download"], api:["docs.save","docs.export"], fidelity:"full" },
  sheet:  { ops:["set_sheet_props","set_rows","clear_rows","set_col_widths","set_formats","set_formula","add_sheet","rename_sheet","finalize","error","ack","clear"], steps:["open","click","fill","cell","menu","download"], api:["sheet.save","sheet.export"], fidelity:"full" },
  slides: { ops:["add_slide","update_slide","delete_slide","set_text","add_shape","add_image","set_style","finalize","error","ack","clear"], steps:["open","click","fill","menu","download"], api:["slides.save","slides.export"], fidelity:"full" },
  paper:  { ops:["append_block","update_block","delete_block","format_runs","set_table_rows","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  canvas: { ops:["add_shape","update_shape","delete_shape","add_text","add_image","set_layer","set_style","group","bring_to_front","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"partial" },
  designer:{ ops:["add_shape","update_shape","delete_shape","add_text","add_image","add_layer_group","apply_effect","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"partial" },
  database:{ ops:["open_query","set_schema","set_rows","set_query_stats","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["database.query"], fidelity:"partial" },
  notepad:{ ops:["append_text","patch_text","finalize","error","ack","clear"], steps:["open","click","fill"], api:["notepad.save"], fidelity:"full" },
  notes:  { ops:["add_note","patch_note","delete_note","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  lists:  { ops:["add_item","patch_item","reorder","check","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  minutes:{ ops:["append_section","set_action_item","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  goals:  { ops:["upsert_record","patch_field","delete_record","link","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  plan:   { ops:["upsert_record","patch_field","delete_record","reorder","link","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  project:{ ops:["upsert_record","patch_field","delete_record","link","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  tasks:  { ops:["upsert_record","patch_field","delete_record","reorder","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  crm:    { ops:["upsert_record","patch_field","delete_record","link","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  people: { ops:["upsert_record","patch_field","delete_record","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  tickets:{ ops:["upsert_record","patch_field","delete_record","link","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  hr:     { ops:["upsert_record","patch_field","delete_record","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  billing:{ ops:["upsert_record","set_rows","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  products:{ ops:["upsert_record","patch_field","delete_record","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  tax:    { ops:["set_rows","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"partial" },
  calendar:{ ops:["upsert_record","patch_field","delete_record","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  campaigns:{ ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  social: { ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  mail:   { ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  templates:{ ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  video:  { ops:[], steps:["open","click"], api:[], fidelity:"none" },
  jukebox:{ ops:[], steps:["open","click"], api:[], fidelity:"none" },
  photos: { ops:[], steps:["open","click","menu"], api:[], fidelity:"none" },
  automations:{ ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  workspace:{ ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  vibe:   { ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  preview:{ ops:["finalize"], steps:["open"], api:[], fidelity:"finalize-only" },
  drive:  { ops:["finalize"], steps:["open"], api:[], fidelity:"finalize-only" },
  browser:{ ops:[], steps:["open","click"], api:[], fidelity:"none" },
  settings:{ ops:[], steps:["open","click","fill"], api:[], fidelity:"none" },
  chat:   { ops:[], steps:[], api:[], fidelity:"none" },
  terminal:{ ops:[], steps:[], api:[], fidelity:"none" }
};
window.appCapability = function (id) { return (window.APP_CAPABILITIES || {})[id] || null; };
