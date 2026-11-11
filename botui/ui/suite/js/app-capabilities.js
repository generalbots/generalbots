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
  paper:  { ops:["append_block","update_block","delete_block","format_runs","set_table_rows","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["paper.save"], fidelity:"full" },
  canvas: { ops:["add_shape","update_shape","delete_shape","add_text","add_image","set_layer","set_style","group","bring_to_front","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"partial" },
  designer:{ ops:["add_shape","update_shape","delete_shape","add_text","add_image","add_layer_group","apply_effect","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"partial" },
  database:{ ops:["open_query","set_schema","set_rows","set_query_stats","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["database.query","database.export"], fidelity:"partial" },
  notepad:{ ops:["append_text","patch_text","finalize","error","ack","clear"], steps:["open","click","fill"], api:["notepad.save"], fidelity:"full" },
  notes:  { ops:["add_note","patch_note","delete_note","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  lists:  { ops:["add_item","patch_item","reorder","check","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  minutes:{ ops:["append_section","set_action_item","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  goals:  { ops:["upsert_record","patch_field","delete_record","link","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["goals.save"], fidelity:"full" },
  plan:   { ops:["upsert_record","patch_field","delete_record","reorder","link","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["plan.save"], fidelity:"full" },
  project:{ ops:["upsert_record","patch_field","delete_record","link","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:[], fidelity:"full" },
  tasks:  { ops:["upsert_record","patch_field","delete_record","reorder","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["tasks.save"], fidelity:"full" },
  crm:    { ops:["upsert_record","patch_field","delete_record","link","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["crm.save"], fidelity:"full" },
  people: { ops:["upsert_record","patch_field","delete_record","finalize","error","ack","clear"], steps:["open","click","fill"], api:["people.save"], fidelity:"full" },
  tickets:{ ops:["upsert_record","patch_field","delete_record","link","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["tickets.save"], fidelity:"full" },
  hr:     { ops:["upsert_record","patch_field","delete_record","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  billing:{ ops:["upsert_record","set_rows","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["billing.save"], fidelity:"full" },
  products:{ ops:["upsert_record","patch_field","delete_record","finalize","error","ack","clear"], steps:["open","click","fill"], api:["products.save"], fidelity:"full" },
  tax:    { ops:["set_rows","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["tax.save"], fidelity:"partial" },
  calendar:{ ops:["upsert_record","patch_field","delete_record","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["calendar.save"], fidelity:"full" },
  campaigns:{ ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  social: { ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  mail:   { ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill"], api:["mail.save"], fidelity:"full" },
  templates:{ ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill"], api:[], fidelity:"full" },
  video:  { ops:["set_progress","set_stage","finalize","error","ack"], steps:["open","click"], api:[], fidelity:"progress" },
  jukebox:{ ops:["set_progress","set_stage","finalize","error","ack"], steps:["open","click"], api:[], fidelity:"progress" },
  photos: { ops:["add_image","finalize","error","ack"], steps:["open","click","menu"], api:[], fidelity:"partial" },
  automations:{ ops:["upsert_record","patch_field","finalize","error","ack","clear"], steps:["open","click","fill","menu"], api:["automations.save"], fidelity:"full" },
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
