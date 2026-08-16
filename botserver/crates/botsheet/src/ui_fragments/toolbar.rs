use super::{html_escape, Lang, t, tf};
use axum::extract::{Form, Query};
use axum::http::HeaderMap;
use axum::response::Html;
use std::collections::HashMap;

pub async fn handle_share_form(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Form<HashMap<String, String>>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let mut params = query;
    if let Some(Form(form)) = body {
        params.extend(form);
    }
    let sheet_id = params.get("id").cloned().unwrap_or_default();
    let action = params.get("action").cloned().unwrap_or_else(|| "share".to_string());
    if action == "delete_named" {
        let name = params.get("name").cloned().unwrap_or_default();
        return Html(format!(
            r##"<div class="ss-toast" style="padding:12px;background:#7f1d1d;color:#fecaca;border-radius:6px;">{}</div>"##,
            tf(lang, "form.share.removed", &[("name", &html_escape(name))])
        ));
    }
    Html(format!(
        r##"<form class="ss-form" id="share-form" hx-post="/api/sheet/share" hx-vals='{{"id":"{id}","emails":""}}' hx-target="#share-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">{}
<input type="email" name="email" placeholder="usuario@empresa.com" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">{}
<select name="permission" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="view">{view}</option>
<option value="comment">{comment}</option>
<option value="edit" selected>{edit}</option>
</select>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{submit}</button>
</form>"##,
        t(lang, "form.share.email"),
        t(lang, "form.share.permission"),
        view = t(lang, "form.share.view"),
        comment = t(lang, "form.share.comment"),
        edit = t(lang, "form.share.edit"),
        submit = t(lang, "form.share.submit"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_find_replace_form(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Form<HashMap<String, String>>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let mut params = query;
    if let Some(Form(form)) = body {
        params.extend(form);
    }
    let sheet_id = params.get("id").cloned().unwrap_or_default();
    Html(format!(
        r##"<form class="ss-form" id="find-replace-form" hx-post="/api/sheet/load" hx-vals='{{"id":"{id}","action":"find_replace"}}' hx-target="#find-replace-results" hx-swap="innerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">{find}
<input type="text" name="find" placeholder="{find_ph}" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">{replace}
<input type="text" name="replace" placeholder="{replace_ph}" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<div id="find-replace-results"></div>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{apply}</button>
</form>"##,
        find = t(lang, "form.find.label"),
        find_ph = t(lang, "form.find.placeholder"),
        replace = t(lang, "form.replace.label"),
        replace_ph = t(lang, "form.replace.placeholder"),
        apply = t(lang, "form.apply"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_conditional_format_form(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Form<HashMap<String, String>>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let mut params = query;
    if let Some(Form(form)) = body {
        params.extend(form);
    }
    let sheet_id = params.get("id").cloned().unwrap_or_default();
    Html(format!(
        r##"<form class="ss-form" id="cf-form" hx-post="/api/sheet/conditional-format" hx-vals='{{"id":"{id}"}}' hx-target="#conditional-format-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">{range_label}
<input type="text" name="range" placeholder="A1:D10" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">{rule}
<select name="rule_type" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="greater_than">{gt}</option>
<option value="less_than">{lt}</option>
<option value="equal_to">{eq}</option>
<option value="between">{between}</option>
<option value="text_contains">{contains}</option>
<option value="duplicate">{duplicates}</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">{value}
<input type="text" name="value" placeholder="100" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">{bg}
<input type="color" name="bg_color" value="#3b82f6" style="width:100%;height:36px;background:#0f172a;border:1px solid #334155;border-radius:4px;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{submit}</button>
</form>"##,
        range_label = t(lang, "form.cf.range"),
        rule = t(lang, "form.cf.rule"),
        gt = t(lang, "form.cf.gt"),
        lt = t(lang, "form.cf.lt"),
        eq = t(lang, "form.cf.eq"),
        between = t(lang, "form.cf.between"),
        contains = t(lang, "form.cf.contains"),
        duplicates = t(lang, "form.cf.duplicates"),
        value = t(lang, "form.value"),
        bg = t(lang, "form.cf.bg"),
        submit = t(lang, "form.cf.submit"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_data_validation_form(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Form<HashMap<String, String>>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let mut params = query;
    if let Some(Form(form)) = body {
        params.extend(form);
    }
    let sheet_id = params.get("id").cloned().unwrap_or_default();
    Html(format!(
        r##"<form class="ss-form" id="dv-form" hx-post="/api/sheet/data-validation" hx-vals='{{"id":"{id}"}}' hx-target="#data-validation-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">{cell}
<input type="text" name="range" placeholder="A1" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">{kind}
<select name="kind" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="list">{list}</option>
<option value="number">{number}</option>
<option value="date">{date}</option>
<option value="text_length">{text_length}</option>
<option value="custom">{custom}</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">{values}
<input type="text" name="values" placeholder="Sim,Não,Talvez" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">{error_msg}
<input type="text" name="error_message" value="{invalid}" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{submit}</button>
</form>"##,
        cell = t(lang, "form.dv.cell"),
        kind = t(lang, "form.dv.kind"),
        list = t(lang, "form.dv.list"),
        number = t(lang, "form.dv.number"),
        date = t(lang, "form.dv.date"),
        text_length = t(lang, "form.dv.text_length"),
        custom = t(lang, "form.dv.custom"),
        values = t(lang, "form.dv.values"),
        error_msg = t(lang, "form.dv.error_msg"),
        invalid = t(lang, "toast.invalid_value"),
        submit = t(lang, "form.dv.submit"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_custom_format_form(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Form<HashMap<String, String>>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let mut params = query;
    if let Some(Form(form)) = body {
        params.extend(form);
    }
    let sheet_id = params.get("id").cloned().unwrap_or_default();
    Html(format!(
        r##"<form class="ss-form" id="cf-custom-form" hx-post="/api/sheet/format" hx-vals='{{"id":"{id}","format_type":"custom"}}' hx-target="#custom-format-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">{range}
<input type="text" name="range" placeholder="A1:A100" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">{code}
<input type="text" name="format" placeholder="#,##0.00_);[Red](#,##0.00)" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;font-family:monospace;" />
</label>
<div style="color:#94a3b8;font-size:11px;background:#0f172a;padding:8px;border-radius:4px;">
<strong>{examples}</strong><br/>
<code>0.00</code> — 1234.56 → 1,234.56<br/>
<code>0.00%</code> — 0.123 → 12.30%<br/>
<code>R$ #,##0.00</code> — 1234.5 → R$ 1,234.50<br/>
<code>dd/mm/yyyy</code> — 45292 → 15/01/2024
</div>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{submit}</button>
</form>"##,
        range = t(lang, "form.range"),
        code = t(lang, "form.fmt.code"),
        examples = t(lang, "form.fmt.examples"),
        submit = t(lang, "form.fmt.submit"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_insert_image_form(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Form<HashMap<String, String>>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let mut params = query;
    if let Some(Form(form)) = body {
        params.extend(form);
    }
    let sheet_id = params.get("id").cloned().unwrap_or_default();
    Html(format!(
        r##"<form class="ss-form" id="img-form" hx-post="/api/sheet/format" hx-vals='{{"id":"{id}","format_type":"image"}}' hx-target="#insert-image-modal" hx-swap="outerHTML" enctype="multipart/form-data" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">{file}
<input type="file" name="image" accept="image/*" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">{anchor}
<input type="text" name="anchor" placeholder="B5" value="B5" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{submit}</button>
</form>"##,
        file = t(lang, "form.img.file"),
        anchor = t(lang, "form.img.anchor"),
        submit = t(lang, "form.img.submit"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_print_preview_form(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Form<HashMap<String, String>>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let mut params = query;
    if let Some(Form(form)) = body {
        params.extend(form);
    }
    let sheet_id = params.get("id").cloned().unwrap_or_default();
    Html(format!(
        r##"<form class="ss-form" id="print-form" hx-post="/api/sheet/export" hx-vals='{{"id":"{id}","format":"pdf"}}' hx-target="#print-preview-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<div class="print-preview-frame" style="background:#0f172a;border:1px solid #334155;border-radius:4px;padding:24px;min-height:300px;">
<h3 style="color:#f8fafc;margin-top:0;">{title}</h3>
<p style="color:#94a3b8;">{description}</p>
</div>
<label style="color:#f8fafc;font-size:13px;">{orientation}
<select name="orientation" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="portrait">{portrait}</option>
<option value="landscape" selected>{landscape}</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">{scale}
<input type="number" name="scale" value="100" min="10" max="200" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{submit}</button>
</form>"##,
        title = t(lang, "form.print.title"),
        description = t(lang, "form.print.description"),
        orientation = t(lang, "form.print.orientation"),
        portrait = t(lang, "form.print.portrait"),
        landscape = t(lang, "form.print.landscape"),
        scale = t(lang, "form.print.scale"),
        submit = t(lang, "form.print.submit"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_chart_form(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Form<HashMap<String, String>>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let mut params = query;
    if let Some(Form(form)) = body {
        params.extend(form);
    }
    let sheet_id = params.get("id").cloned().unwrap_or_default();
    Html(format!(
        r##"<form class="ss-form" id="chart-form" hx-post="/api/sheet/chart" hx-vals='{{"id":"{id}"}}' hx-target="#chart-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">{chart_type}
<select name="chart_type" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="bar">{bar}</option>
<option value="line">{line}</option>
<option value="pie">{pie}</option>
<option value="scatter">{scatter}</option>
<option value="area">{area}</option>
<option value="column">{column}</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">{data_range}
<input type="text" name="data_range" placeholder="A1:D10" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">{title}
<input type="text" name="title" placeholder="Vendas por Trimestre" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">{anchor}
<input type="text" name="anchor" value="F2" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{submit}</button>
</form>"##,
        chart_type = t(lang, "form.chart.type"),
        bar = t(lang, "form.chart.bar"),
        line = t(lang, "form.chart.line"),
        pie = t(lang, "form.chart.pie"),
        scatter = t(lang, "form.chart.scatter"),
        area = t(lang, "form.chart.area"),
        column = t(lang, "form.chart.column"),
        data_range = t(lang, "form.chart.data_range"),
        title = t(lang, "form.chart.title"),
        anchor = t(lang, "form.chart.anchor"),
        submit = t(lang, "form.chart.submit"),
        id = html_escape(sheet_id)
    ))
}
