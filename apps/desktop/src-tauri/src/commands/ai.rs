use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::{
    get_meta,
    models::{
        get_company, get_document, get_issue, get_ticket, list_comments, list_contacts, list_documents,
        list_issues, list_tickets, SearchResult,
    },
    DbError,
};
use crate::state::AppState;

fn map_err(e: DbError) -> String {
    e.to_string()
}

#[derive(Debug, Deserialize)]
pub struct AiChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct AiChatRequest {
    pub messages: Vec<AiChatMessage>,
    pub context_entity_type: Option<String>,
    pub context_entity_id: Option<String>,
    pub intent: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiDraft {
    pub kind: String,
    pub title: Option<String>,
    pub body: String,
    pub visibility: Option<String>,
    pub target_entity_type: Option<String>,
    pub target_entity_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AiChatResponse {
    pub answer: String,
    pub citations: Vec<SearchResult>,
    pub draft: Option<AiDraft>,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessageContent,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessageContent {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

fn build_context(
    state: &AppState,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    user_query: &str,
) -> Result<(String, Vec<SearchResult>), String> {
    let conn = state.conn.lock();
    let ws = get_meta(&conn, "workspace_id")
        .map_err(map_err)?
        .ok_or_else(|| "workspace not initialized".to_string())?;

    let mut citations: Vec<SearchResult> = Vec::new();
    let mut parts: Vec<String> = Vec::new();

    if let (Some(etype), Some(eid)) = (entity_type, entity_id) {
        match etype {
            "company" => {
                if let Some(c) = get_company(&conn, eid).map_err(map_err)? {
                    parts.push(format!(
                        "Company: {} | domain={} | industry={} | notes={}",
                        c.name,
                        c.domain.clone().unwrap_or_default(),
                        c.industry.clone().unwrap_or_default(),
                        c.notes.clone().unwrap_or_default()
                    ));
                    citations.push(SearchResult {
                        entity_type: "company".into(),
                        entity_id: c.id.clone(),
                        title: c.name.clone(),
                        snippet: c.notes.clone().unwrap_or_default(),
                        rank: 0.0,
                    });
                    let contacts = list_contacts(&conn, &ws, Some(&c.id)).map_err(map_err)?;
                    for contact in contacts.iter().take(5) {
                        parts.push(format!(
                            "Contact: {} <{}> ({})",
                            contact.name,
                            contact.email.clone().unwrap_or_default(),
                            contact.title.clone().unwrap_or_default()
                        ));
                        citations.push(SearchResult {
                            entity_type: "contact".into(),
                            entity_id: contact.id.clone(),
                            title: contact.name.clone(),
                            snippet: contact.email.clone().unwrap_or_default(),
                            rank: 0.0,
                        });
                    }
                    for t in list_tickets(&conn, &ws, Some(&c.id))
                        .map_err(map_err)?
                        .into_iter()
                        .take(5)
                    {
                        parts.push(format!(
                            "Ticket [{}]: {} | status={} priority={} | {}",
                            t.id,
                            t.title,
                            t.status,
                            t.priority,
                            t.description.clone().unwrap_or_default()
                        ));
                        citations.push(SearchResult {
                            entity_type: "ticket".into(),
                            entity_id: t.id.clone(),
                            title: t.title.clone(),
                            snippet: t.description.clone().unwrap_or_default(),
                            rank: 0.0,
                        });
                    }
                }
            }
            "ticket" => {
                if let Some(t) = get_ticket(&conn, eid).map_err(map_err)? {
                    parts.push(format!(
                        "Ticket: {} | status={} priority={} | {}",
                        t.title,
                        t.status,
                        t.priority,
                        t.description.clone().unwrap_or_default()
                    ));
                    citations.push(SearchResult {
                        entity_type: "ticket".into(),
                        entity_id: t.id.clone(),
                        title: t.title.clone(),
                        snippet: t.description.clone().unwrap_or_default(),
                        rank: 0.0,
                    });
                    if let Some(c) = get_company(&conn, &t.company_id).map_err(map_err)? {
                        parts.push(format!("Customer company: {}", c.name));
                        citations.push(SearchResult {
                            entity_type: "company".into(),
                            entity_id: c.id.clone(),
                            title: c.name.clone(),
                            snippet: String::new(),
                            rank: 0.0,
                        });
                    }
                    for i in list_issues(&conn, &ws, None)
                        .map_err(map_err)?
                        .into_iter()
                        .filter(|i| i.ticket_id.as_deref() == Some(t.id.as_str()))
                    {
                        parts.push(format!(
                            "Linked issue: {} | status={} | blocked={}",
                            i.title,
                            i.status,
                            i.blocked_reason.clone().unwrap_or_default()
                        ));
                        citations.push(SearchResult {
                            entity_type: "issue".into(),
                            entity_id: i.id.clone(),
                            title: i.title.clone(),
                            snippet: i.blocked_reason.clone().unwrap_or_default(),
                            rank: 0.0,
                        });
                    }
                    for d in list_documents(&conn, &ws)
                        .map_err(map_err)?
                        .into_iter()
                        .filter(|d| d.ticket_id.as_deref() == Some(t.id.as_str()))
                        .take(3)
                    {
                        parts.push(format!("Doc: {}\n{}", d.title, d.body));
                        citations.push(SearchResult {
                            entity_type: "document".into(),
                            entity_id: d.id.clone(),
                            title: d.title.clone(),
                            snippet: d.body.chars().take(120).collect(),
                            rank: 0.0,
                        });
                    }
                    for comment in list_comments(&conn, &ws, "ticket", &t.id)
                        .map_err(map_err)?
                        .into_iter()
                        .rev()
                        .take(4)
                    {
                        parts.push(format!(
                            "Comment ({}/{}): {}",
                            comment.visibility, comment.author, comment.body
                        ));
                    }
                    for asset in crate::db::extended::assets_for_ticket(&conn, &t.id)
                        .map_err(map_err)?
                    {
                        parts.push(format!(
                            "Asset: {} serial={} warranty={}",
                            asset.name,
                            asset.serial.clone().unwrap_or_default(),
                            asset.warranty_expires_at.clone().unwrap_or_default()
                        ));
                        citations.push(SearchResult {
                            entity_type: "asset".into(),
                            entity_id: asset.id.clone(),
                            title: asset.name.clone(),
                            snippet: asset.serial.clone().unwrap_or_default(),
                            rank: 0.0,
                        });
                    }
                    for rma in crate::db::extended::rmas_for_ticket(&conn, &t.id).map_err(map_err)? {
                        parts.push(format!(
                            "RMA: status={} reason={}",
                            rma.status,
                            rma.reason.clone().unwrap_or_default()
                        ));
                        citations.push(SearchResult {
                            entity_type: "rma".into(),
                            entity_id: rma.id.clone(),
                            title: "RMA".into(),
                            snippet: rma.reason.clone().unwrap_or_default(),
                            rank: 0.0,
                        });
                    }
                }
            }
            "issue" => {
                if let Some(i) = get_issue(&conn, eid).map_err(map_err)? {
                    parts.push(format!(
                        "Issue: {} | status={} priority={} | blocked={} | {}",
                        i.title,
                        i.status,
                        i.priority,
                        i.blocked_reason.clone().unwrap_or_default(),
                        i.description.clone().unwrap_or_default()
                    ));
                    citations.push(SearchResult {
                        entity_type: "issue".into(),
                        entity_id: i.id.clone(),
                        title: i.title.clone(),
                        snippet: i.blocked_reason.clone().unwrap_or_default(),
                        rank: 0.0,
                    });
                    if let Some(tid) = &i.ticket_id {
                        if let Some(t) = get_ticket(&conn, tid).map_err(map_err)? {
                            parts.push(format!(
                                "Originating ticket: {} | {}",
                                t.title,
                                t.description.clone().unwrap_or_default()
                            ));
                            citations.push(SearchResult {
                                entity_type: "ticket".into(),
                                entity_id: t.id.clone(),
                                title: t.title.clone(),
                                snippet: t.description.clone().unwrap_or_default(),
                                rank: 0.0,
                            });
                        }
                    }
                    for d in list_documents(&conn, &ws)
                        .map_err(map_err)?
                        .into_iter()
                        .filter(|d| d.issue_id.as_deref() == Some(i.id.as_str()))
                        .take(3)
                    {
                        parts.push(format!("Doc: {}\n{}", d.title, d.body));
                        citations.push(SearchResult {
                            entity_type: "document".into(),
                            entity_id: d.id.clone(),
                            title: d.title.clone(),
                            snippet: d.body.chars().take(120).collect(),
                            rank: 0.0,
                        });
                    }
                    for comment in list_comments(&conn, &ws, "issue", &i.id)
                        .map_err(map_err)?
                        .into_iter()
                        .rev()
                        .take(4)
                    {
                        parts.push(format!(
                            "Comment ({}/{}): {}",
                            comment.visibility, comment.author, comment.body
                        ));
                    }
                }
            }
            "document" => {
                if let Some(d) = get_document(&conn, eid).map_err(map_err)? {
                    parts.push(format!("Document: {}\n{}", d.title, d.body));
                    citations.push(SearchResult {
                        entity_type: "document".into(),
                        entity_id: d.id.clone(),
                        title: d.title.clone(),
                        snippet: d.body.chars().take(120).collect(),
                        rank: 0.0,
                    });
                }
            }
            _ => {}
        }
    }

    // Also pull FTS hits for the latest user question.
    let fts_query = user_query
        .split_whitespace()
        .take(8)
        .map(|t| format!("\"{}\"*", t.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" ");
    if !fts_query.is_empty() {
        if let Ok(mut stmt) = conn.prepare(
            "SELECT entity_type, entity_id, title, snippet(search_fts, 3, '', '', '…', 16), bm25(search_fts)
             FROM search_fts WHERE search_fts MATCH ?1 ORDER BY rank LIMIT 8",
        ) {
            if let Ok(rows) = stmt.query_map(rusqlite::params![fts_query], |row| {
                Ok(SearchResult {
                    entity_type: row.get(0)?,
                    entity_id: row.get(1)?,
                    title: row.get(2)?,
                    snippet: row.get(3)?,
                    rank: row.get(4)?,
                })
            }) {
                for row in rows.flatten() {
                    if !citations
                        .iter()
                        .any(|c| c.entity_id == row.entity_id && c.entity_type == row.entity_type)
                    {
                        parts.push(format!(
                            "Search hit [{}] {}: {}",
                            row.entity_type, row.title, row.snippet
                        ));
                        citations.push(row);
                    }
                }
            }
        }
    }

    Ok((parts.join("\n\n"), citations))
}

fn humanize_prose(text: &str) -> String {
    text.replace(" — ", ". ").replace('—', ". ")
}

fn infer_intent(explicit: Option<&str>, question: &str) -> String {
    if let Some(intent) = explicit {
        if !intent.trim().is_empty() && intent != "ask" {
            return intent.trim().to_string();
        }
    }
    let q = question.to_lowercase();
    if q.contains("draft") && (q.contains("article") || q.contains("doc") || q.contains("knowledge"))
    {
        "draft_article".into()
    } else if q.contains("draft") && (q.contains("reply") || q.contains("customer email")) {
        "draft_reply".into()
    } else if q.contains("draft") && (q.contains("update") || q.contains("follow-up") || q.contains("follow up"))
    {
        "draft_update".into()
    } else if q.contains("draft") && (q.contains("note") || q.contains("internal")) {
        "draft_note".into()
    } else if q.contains("rma") || q.contains("replacement") {
        "prepare_rma".into()
    } else if q.contains("update this ticket") || q.contains("update this issue") || q.contains("update the record")
    {
        "update_record".into()
    } else if q.contains("follow-up") || q.contains("follow up") || q.contains("create ticket") {
        "create_followup".into()
    } else if q.contains("create") && q.contains("issue") {
        "create_issue".into()
    } else if q.contains("link") && q.contains("issue") {
        "link_issue".into()
    } else if q.contains("action plan") || q.contains("propose a plan") {
        "action_plan".into()
    } else {
        "ask".into()
    }
}

fn local_draft(
    intent: &str,
    context: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
) -> AiDraft {
    let facts = context
        .split("\n\n")
        .take(4)
        .map(|chunk| chunk.trim())
        .filter(|chunk| !chunk.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let facts = if facts.is_empty() {
        "No additional workspace facts were available.".to_string()
    } else {
        facts
    };

    match intent {
        "prepare_rma" => AiDraft {
            kind: "rma".into(),
            title: Some("Open RMA".into()),
            body: format!(
                "Open an RMA from the current record.\n\n{facts}\n\nThis will create an open replacement request linked to the customer asset."
            ),
            visibility: Some("internal".into()),
            target_entity_type: entity_type.map(str::to_string),
            target_entity_id: entity_id.map(str::to_string),
        },
        "create_followup" => AiDraft {
            kind: "ticket".into(),
            title: Some("Follow-up ticket".into()),
            body: format!("Follow up on the current work.\n\n{facts}"),
            visibility: Some("internal".into()),
            target_entity_type: entity_type.map(str::to_string),
            target_entity_id: entity_id.map(str::to_string),
        },
        "link_issue" => AiDraft {
            kind: "link".into(),
            title: Some("Link ticket to issue".into()),
            body: format!(
                "Link this support ticket to the engineering issue so both records stay in sync.\n\n{facts}"
            ),
            visibility: Some("internal".into()),
            target_entity_type: entity_type.map(str::to_string),
            target_entity_id: entity_id.map(str::to_string),
        },
        "create_issue" => AiDraft {
            kind: "issue".into(),
            title: Some("Engineering follow-up".into()),
            body: format!("Create an engineering issue from the current record.\n\n{facts}"),
            visibility: Some("internal".into()),
            target_entity_type: entity_type.map(str::to_string),
            target_entity_id: entity_id.map(str::to_string),
        },
        "update_record" => AiDraft {
            kind: "record_update".into(),
            title: Some("Update record".into()),
            body: format!("Update the focused record with the latest working notes.\n\n{facts}"),
            visibility: Some("internal".into()),
            target_entity_type: entity_type.map(str::to_string),
            target_entity_id: entity_id.map(str::to_string),
        },
        "action_plan" => AiDraft {
            kind: "plan".into(),
            title: Some("Action plan".into()),
            body: format!(
                "1. Keep the support ticket in progress.\n2. Confirm the linked engineering issue.\n3. Open or update an RMA if the asset needs replacement.\n\n{facts}"
            ),
            visibility: Some("internal".into()),
            target_entity_type: entity_type.map(str::to_string),
            target_entity_id: entity_id.map(str::to_string),
        },
        "draft_article" => AiDraft {
            kind: "article".into(),
            title: Some("Draft knowledge article".into()),
            body: format!(
                "# Knowledge draft\n\n## Summary\nThis article is drafted from linked Confluity records.\n\n## Known facts\n{facts}\n\n## Workaround\nUse the linked troubleshooting steps if a customer is blocked.\n\n## Next steps\nConfirm with engineering before publishing."
            ),
            visibility: None,
            target_entity_type: entity_type.map(str::to_string),
            target_entity_id: entity_id.map(str::to_string),
        },
        "draft_update" => AiDraft {
            kind: "update".into(),
            title: Some("Customer update".into()),
            body: "Hi,\n\nQuick update from our side: this is still being tracked, and we have the right people on it.\n\nWe’ll follow up as soon as there’s something new to share.\n\nThanks,\nSupport".into(),
            visibility: Some("public".into()),
            target_entity_type: entity_type.map(str::to_string),
            target_entity_id: entity_id.map(str::to_string),
        },
        "draft_note" => AiDraft {
            kind: "note".into(),
            title: Some("Internal note".into()),
            body: format!(
                "Internal note:\n{facts}\n\nNeed: confirm current blocker, owner, and whether this should stay in support or move with engineering."
            ),
            visibility: Some("internal".into()),
            target_entity_type: entity_type.map(str::to_string),
            target_entity_id: entity_id.map(str::to_string),
        },
        _ => AiDraft {
            kind: "reply".into(),
            title: Some("Customer reply".into()),
            body: "Hi,\n\nThanks for the details. This is being tracked, and we’re looking into it.\n\nIf telemetry is still stale, force a full resync from the device console. Please don’t roll back firmware during the pilot.\n\nWe’ll follow up when we have the next update.\n\nBest,\nSupport".into(),
            visibility: Some("public".into()),
            target_entity_type: entity_type.map(str::to_string),
            target_entity_id: entity_id.map(str::to_string),
        },
    }
}

fn parse_model_draft(
    raw: &str,
    intent: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
) -> AiDraft {
    let mut title = None;
    let mut visibility = match intent {
        "draft_note" => Some("internal".to_string()),
        "draft_article" => None,
        _ => Some("public".to_string()),
    };
    let mut body = raw.trim().to_string();

    if let Some(rest) = raw.strip_prefix("TITLE:") {
        if let Some((first, remaining)) = rest.split_once('\n') {
            title = Some(first.trim().to_string());
            body = remaining.trim().to_string();
        }
    }
    if body.to_uppercase().starts_with("VISIBILITY:") {
        if let Some((first, remaining)) = body.split_once('\n') {
            let value = first.split(':').nth(1).unwrap_or("").trim().to_lowercase();
            if value == "internal" || value == "public" {
                visibility = Some(value);
            }
            body = remaining.trim().to_string();
        }
    }
    if let Some(stripped) = body.strip_prefix("BODY:") {
        body = stripped.trim().to_string();
    }

    let kind = match intent {
        "draft_note" => "note",
        "draft_article" => "article",
        "draft_update" => "update",
        "prepare_rma" => "rma",
        "create_followup" => "ticket",
        "create_issue" => "issue",
        "link_issue" => "link",
        "update_record" => "record_update",
        "action_plan" => "plan",
        _ => "reply",
    };

    AiDraft {
        kind: kind.into(),
        title: title.map(|t| humanize_prose(&t)),
        body: humanize_prose(&body),
        visibility,
        target_entity_type: entity_type.map(str::to_string),
        target_entity_id: entity_id.map(str::to_string),
    }
}

fn local_fallback_answer(context: &str, citations: &[SearchResult], question: &str) -> String {
    let q = question.to_lowercase();
    let mut lines = Vec::new();

    if q.contains("summar") {
        lines.push("Customer summary based on local Confluity records:".to_string());
    } else if q.contains("block") {
        lines.push("Blockers found in linked records:".to_string());
    } else if q.contains("related") || q.contains("everything") {
        lines.push("Related records in this workspace:".to_string());
    } else if q.contains("doc") || q.contains("documentation") {
        lines.push("Documentation matches:".to_string());
    } else {
        lines.push("Answer from local Confluity context (no API key configured):".to_string());
    }

    if context.trim().is_empty() {
        lines.push("No related records were found for the current context.".into());
    } else {
        // Keep fallback concise.
        for chunk in context.split("\n\n").take(6) {
            lines.push(format!("- {chunk}"));
        }
    }

    if !citations.is_empty() {
        lines.push(String::new());
        lines.push("Citations:".into());
        for c in citations.iter().take(8) {
            lines.push(format!("- [{}] {} ({})", c.entity_type, c.title, c.entity_id));
        }
    }

    lines.push(String::new());
    lines.push(
        "Tip: add an OpenAI-compatible API key under Settings → AI Connections → API connections."
            .into(),
    );
    lines.join("\n")
}

#[tauri::command]
pub async fn ai_chat_cmd(
    state: tauri::State<'_, Arc<AppState>>,
    request: AiChatRequest,
) -> Result<AiChatResponse, String> {
    let user_question = request
        .messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let (context, citations) = build_context(
        &state,
        request.context_entity_type.as_deref(),
        request.context_entity_id.as_deref(),
        &user_question,
    )?;
    let intent = infer_intent(request.intent.as_deref(), &user_question);
    let is_draft = intent.starts_with("draft_")
        || intent == "prepare_rma"
        || intent == "create_followup"
        || intent == "create_issue"
        || intent == "link_issue"
        || intent == "update_record"
        || intent == "action_plan";

    let (api_key, base_url, model) = {
        let conn = state.conn.lock();
        crate::db::ai_config::active_api_credentials(&conn).map_err(map_err)?
    };

    if api_key.trim().is_empty() {
        if is_draft {
            let draft = local_draft(
                &intent,
                &context,
                request.context_entity_type.as_deref(),
                request.context_entity_id.as_deref(),
            );
            let answer = humanize_prose(&format!(
                "Draft ready. Review it, then apply if it looks right.\n\n{}",
                draft.body
            ));
            return Ok(AiChatResponse {
                answer,
                citations,
                draft: Some(AiDraft {
                    body: humanize_prose(&draft.body),
                    title: draft.title.map(|t| humanize_prose(&t)),
                    ..draft
                }),
            });
        }
        return Ok(AiChatResponse {
            answer: humanize_prose(&local_fallback_answer(&context, &citations, &user_question)),
            citations,
            draft: None,
        });
    }

    let system = if is_draft {
        format!(
            "You are the Confluity AI companion drafting content for a human to approve. \
             Use only the provided workspace context. Do not invent statuses, dates, or SLAs. \
             Write like a concise professional human. Do not use em dashes. \
             For customer-facing drafts (reply or update): do not mention Confluity, tickets, issues, \
             linked records, or any internal tracking system. Say only that the issue is being tracked. \
             Share only customer-safe workarounds and next steps. \
             Return exactly this format:\nTITLE: <short title>\nVISIBILITY: public|internal\nBODY:\n<draft>\n\n\
             WORKSPACE CONTEXT:\n{context}"
        )
    } else {
        format!(
            "You are the Confluity AI companion. Answer using only the provided workspace context. \
             Be concise and practical. Write like a professional coworker, not a chatbot. \
             Do not use em dashes. Always cite records as [type: title]. \
             Do not invent invoice amounts, inventory counts, or statuses not present in context.\n\n\
             WORKSPACE CONTEXT:\n{context}"
        )
    };

    let mut messages = vec![OpenAiMessage {
        role: "system".into(),
        content: system,
    }];
    for m in request.messages {
        messages.push(OpenAiMessage {
            role: m.role,
            content: m.content,
        });
    }

    let url = format!(
        "{}/chat/completions",
        base_url.trim_end_matches('/')
    );
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .bearer_auth(api_key)
        .json(&OpenAiRequest {
            model,
            messages,
            temperature: 0.2,
        })
        .send()
        .await
        .map_err(|e| format!("AI request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("AI provider error ({status}): {body}"));
    }

    let parsed: OpenAiResponse = resp
        .json()
        .await
        .map_err(|e| format!("AI response parse failed: {e}"))?;
    let answer = parsed
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response from model.".into());

    let draft = if is_draft {
        Some(parse_model_draft(
            &answer,
            &intent,
            request.context_entity_type.as_deref(),
            request.context_entity_id.as_deref(),
        ))
    } else {
        None
    };

    Ok(AiChatResponse {
        answer: humanize_prose(&answer),
        citations,
        draft: draft.map(|d| AiDraft {
            body: humanize_prose(&d.body),
            title: d.title.map(|t| humanize_prose(&t)),
            ..d
        }),
    })
}
