use std::sync::Arc;

use rusqlite::{types::ValueRef, Connection};
use serde_json::{Map, Value};

use crate::db::{
    get_meta,
    identity::{
        stored_session, supabase_anon_key, write_session, StoredIdentitySession, PROVIDER_SUPABASE,
    },
    set_meta,
};
use crate::state::AppState;

const SCHEMA_HINT: &str = "Confluity could not create its tables in this project. Enter the database password from Supabase → Project Settings → Database, then continue again.";
const NEED_DB_PASSWORD: &str = "Enter the database password from Supabase → Project Settings → Database. Confluity uses it to create the tables for accounts, cases, and files.";

struct Remote {
    base: String,
    anon: String,
    token: String,
    refresh: Option<String>,
}

pub async fn ensure_schema_present(state: &AppState) -> Result<(), String> {
    let remote = load_remote(state, false)?;
    if probe_accounts(&remote).await? {
        return Ok(());
    }
    let password = {
        let conn = state.conn.lock();
        get_meta(&conn, "identity_supabase_db_password")
            .map_err(|e| e.to_string())?
            .filter(|p| !p.trim().is_empty())
    };
    let Some(password) = password else {
        return Err(NEED_DB_PASSWORD.into());
    };
    apply_schema(&remote.base, &password).await?;
    if probe_accounts(&remote).await? {
        return Ok(());
    }
    Err(SCHEMA_HINT.into())
}

async fn probe_accounts(remote: &Remote) -> Result<bool, String> {
    let url = format!("{}/rest/v1/confluity_accounts?select=id&limit=1", remote.base);
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("apikey", &remote.anon)
        .header("Authorization", format!("Bearer {}", remote.anon))
        .send()
        .await
        .map_err(|e| format!("Could not reach Supabase: {e}"))?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    let lower = body.to_ascii_lowercase();
    if lower.contains("invalid api key") || lower.contains("no api key") {
        return Err("Supabase rejected the anon key.".into());
    }
    if schema_missing(status, &body) || status.as_u16() == 404 {
        return Ok(false);
    }
    if status.is_success() || status.as_u16() == 401 || status.as_u16() == 403 {
        return Ok(true);
    }
    Err(format!(
        "Could not check the Supabase project ({status}). {}",
        body.chars().take(160).collect::<String>()
    ))
}

/// Mark the new Confluity login confirmed so setup does not depend on the email link.
pub async fn confirm_auth_email(state: &AppState, email: &str) -> Result<(), String> {
    let (base, db_password) = {
        let conn = state.conn.lock();
        let base = get_meta(&conn, "identity_supabase_url")
            .map_err(|e| e.to_string())?
            .filter(|u| !u.trim().is_empty())
            .ok_or_else(|| "Supabase project URL is missing.".to_string())?;
        let db_password = get_meta(&conn, "identity_supabase_db_password")
            .map_err(|e| e.to_string())?
            .filter(|p| !p.trim().is_empty());
        (base, db_password)
    };
    let Some(db_password) = db_password else {
        return Err(
            "Supabase is waiting on an email confirmation. Submit the Server step with the database password so Confluity can confirm this login."
                .into(),
        );
    };
    let client = connect_supabase(&base, &db_password).await?;
    let email = email.trim();
    let updated = client
        .execute(
            "update auth.users
             set email_confirmed_at = coalesce(email_confirmed_at, now()),
                 confirmation_token = '',
                 updated_at = now()
             where lower(email) = lower($1)",
            &[&email],
        )
        .await
        .map_err(|e| scrub(format!("Could not confirm the login: {e}"), &db_password))?;
    if updated == 0 {
        return Err("Supabase did not find that account to confirm.".into());
    }
    let _ = client
        .execute(
            "update auth.identities
             set identity_data = jsonb_set(coalesce(identity_data, '{}'::jsonb), '{email_verified}', 'true'::jsonb, true)
             where user_id in (select id from auth.users where lower(email) = lower($1))",
            &[&email],
        )
        .await;
    Ok(())
}

/// Point an existing Supabase auth user at a new temporary password and confirm the email.
pub async fn reset_auth_password(state: &AppState, email: &str, password: &str) -> Result<String, String> {
    let (base, db_password) = {
        let conn = state.conn.lock();
        let base = get_meta(&conn, "identity_supabase_url")
            .map_err(|e| e.to_string())?
            .filter(|u| !u.trim().is_empty())
            .ok_or_else(|| "Supabase project URL is missing.".to_string())?;
        let db_password = get_meta(&conn, "identity_supabase_db_password")
            .map_err(|e| e.to_string())?
            .filter(|p| !p.trim().is_empty())
            .ok_or_else(|| {
                "Enter the database password from Supabase → Project Settings → Database before inviting members."
                    .to_string()
            })?;
        (base, db_password)
    };
    let client = connect_supabase(&base, &db_password).await?;
    let email = email.trim();
    let row = client
        .query_opt(
            "select id::text from auth.users where lower(email) = lower($1)",
            &[&email],
        )
        .await
        .map_err(|e| scrub(format!("Could not look up the account: {e}"), &db_password))?;
    let Some(row) = row else {
        return Err("Supabase does not have an account for that email.".into());
    };
    let id: String = row.get(0);
    client
        .execute(
            "update auth.users
             set encrypted_password = extensions.crypt($2, extensions.gen_salt('bf')),
                 email_confirmed_at = coalesce(email_confirmed_at, now()),
                 confirmation_token = '',
                 updated_at = now()
             where id = $1::uuid",
            &[&id, &password],
        )
        .await
        .map_err(|e| scrub(format!("Could not set the sign-in code: {e}"), &db_password))?;
    drop(client);
    confirm_auth_email(state, email).await?;
    Ok(id)
}

async fn apply_schema(base: &str, password: &str) -> Result<(), String> {
    let client = connect_supabase(base, password).await?;
    client
        .batch_execute(include_str!("../../../../../supabase/confluity.sql"))
        .await
        .map_err(|e| scrub(format!("Could not create Confluity tables: {e}"), password))?;
    Ok(())
}

async fn connect_supabase(base: &str, password: &str) -> Result<tokio_postgres::Client, String> {
    let trimmed = password.trim();
    if trimmed.starts_with("postgres://") || trimmed.starts_with("postgresql://") {
        let url = with_ssl(trimmed);
        let pending = tokio_postgres::connect(&url, supabase_tls());
        let (client, connection) = tokio::time::timeout(std::time::Duration::from_secs(25), pending)
            .await
            .map_err(|_| "The Supabase database did not respond in time.".to_string())?
            .map_err(|e| explain_db_error(&e.to_string(), password))?;
        tokio::spawn(async move {
            let _ = connection.await;
        });
        return Ok(client);
    }

    let project = project_ref(base)?;
    let mut config = tokio_postgres::Config::new();
    config.host(&format!("db.{project}.supabase.co"));
    config.port(5432);
    config.user("postgres");
    config.password(trimmed);
    config.dbname("postgres");
    config.ssl_mode(tokio_postgres::config::SslMode::Require);
    config.connect_timeout(std::time::Duration::from_secs(20));
    let pending = config.connect(supabase_tls());
    let (client, connection) = tokio::time::timeout(std::time::Duration::from_secs(25), pending)
        .await
        .map_err(|_| "The Supabase database did not respond in time.".to_string())?
        .map_err(|e| explain_db_error(&e.to_string(), password))?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    Ok(client)
}

/// Supabase encrypts Postgres with its own certificate. `sslmode=require` encrypts
/// without asking Windows to trust that private certificate.
fn supabase_tls() -> tokio_postgres_rustls::MakeRustlsConnect {
    let provider = rustls::crypto::ring::default_provider();
    let config = rustls::ClientConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()
        .expect("rustls defaults")
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(EncryptOnly))
        .with_no_client_auth();
    tokio_postgres_rustls::MakeRustlsConnect::new(config)
}

#[derive(Debug)]
struct EncryptOnly;

impl rustls::client::danger::ServerCertVerifier for EncryptOnly {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn explain_db_error(text: &str, password: &str) -> String {
    let shown = scrub(text.to_string(), password);
    let lower = shown.to_ascii_lowercase();
    if lower.contains("password") || lower.contains("authentication") {
        "That database password was not accepted. Use the password from Supabase → Project Settings → Database.".into()
    } else if lower.contains("handshake") || lower.contains("certificate") || lower.contains("tls") {
        format!("Could not encrypt the connection to the Supabase database. {shown}")
    } else {
        format!(
            "Could not reach the Supabase database. Check the database password, or paste the Session pooler URI from the Supabase Connect dialog. {shown}"
        )
    }
}

fn project_ref(base: &str) -> Result<String, String> {
    let host = base
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("");
    let project = host.split('.').next().unwrap_or("");
    if project.is_empty() || !host.ends_with("supabase.co") {
        return Err("Use the project URL from Supabase, like https://your-project.supabase.co.".into());
    }
    Ok(project.to_string())
}

fn with_ssl(url: &str) -> String {
    if url.contains("sslmode=") {
        url.to_string()
    } else if url.contains('?') {
        format!("{url}&sslmode=require")
    } else {
        format!("{url}?sslmode=require")
    }
}

fn scrub(message: String, secret: &str) -> String {
    let secret = secret.trim();
    if secret.len() < 4 {
        message
    } else {
        message.replace(secret, "••••")
    }
}

/// Push the live workspace (accounts, cases, files, and the other shared records).
pub async fn push_workspace_records(state: &AppState) -> Result<(), String> {
    let mut remote = match load_remote(state, true) {
        Err(err) if err == "not supabase" => return Ok(()),
        other => other?,
    };
    if remote.token.is_empty() {
        return Err(
            "Supabase did not return a session, so the workspace could not be saved. Confirm the account, then submit again."
                .into(),
        );
    }
    let snapshot = {
        let conn = state.conn.lock();
        let ws = match get_meta(&conn, "workspace_id").map_err(|e| e.to_string())? {
            Some(id) if !id.is_empty() => id,
            _ => return Ok(()),
        };
        collect_snapshot(&conn, &ws)?
    };
    for (table, rows) in snapshot {
        upsert_table(state, &mut remote, table, rows).await?;
    }
    Ok(())
}

pub async fn upload_attachment_if_remote(
    state: &AppState,
    workspace_id: &str,
    stored_name: &str,
    mime: Option<&str>,
    bytes: &[u8],
) -> Result<(), String> {
    let Ok(mut remote) = load_remote(state, true) else {
        return Ok(());
    };
    if remote.token.is_empty() {
        return Err("Sign in to the Confluity account before sending files.".into());
    }
    let path = format!("{workspace_id}/{stored_name}");
    let url = format!(
        "{}/storage/v1/object/confluity-files/{}",
        remote.base,
        encode_path(&path)
    );
    let content_type = mime.unwrap_or("application/octet-stream");
    let resp = authed(
        state,
        &mut remote,
        reqwest::Method::POST,
        url,
        Some(content_type),
        Some(bytes.to_vec()),
        &[("x-upsert", "true")],
    )
    .await?;
    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }
    let body = resp.text().await.unwrap_or_default();
    if schema_missing(status, &body) || body.to_ascii_lowercase().contains("bucket") {
        return Err(SCHEMA_HINT.into());
    }
    Err(interpret_rest_error(status, &body))
}

/// Sign-in is only valid when this user is a Confluity account in the project.
pub async fn require_confluity_account(state: &AppState) -> Result<(), String> {
    let mut remote = load_remote(state, true)?;
    let (user_id, email) = {
        let conn = state.conn.lock();
        let session = stored_session(&conn)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Sign-in session missing.".to_string())?;
        (session.user_id, session.email.unwrap_or_default())
    };
    let by_id = format!(
        "{}/rest/v1/confluity_accounts?id=eq.{}&select=id,email,full_name,workspace_id&limit=1",
        remote.base,
        encode_query(&user_id)
    );
    let mut rows = fetch_rows(state, &mut remote, by_id).await?;
    if rows.is_empty() && !email.is_empty() {
        let by_email = format!(
            "{}/rest/v1/confluity_accounts?email=eq.{}&select=id,email,full_name,workspace_id&limit=1",
            remote.base,
            encode_query(&email)
        );
        rows = fetch_rows(state, &mut remote, by_email).await?;
    }
    if rows.is_empty() {
        return Err(
            "This login is not a Confluity account in the Supabase project. Create it from Set up."
                .into(),
        );
    }
    Ok(())
}

pub async fn flush_queued(state: &AppState) -> Result<(), String> {
    let queued_ids = {
        let conn = state.conn.lock();
        if get_meta(&conn, "setup_complete")
            .map_err(|e| e.to_string())?
            .as_deref()
            != Some("true")
        {
            return Ok(());
        }
        if get_meta(&conn, "identity_provider")
            .map_err(|e| e.to_string())?
            .as_deref()
            != Some(PROVIDER_SUPABASE)
        {
            return Ok(());
        }
        let ws = get_meta(&conn, "workspace_id")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        if ws.is_empty() {
            return Ok(());
        }
        let mut stmt = conn
            .prepare(
                "SELECT id FROM outbox WHERE workspace_id = ?1 AND status = 'queued' ORDER BY created_at LIMIT 500",
            )
            .map_err(|e| e.to_string())?;
        let ids = stmt
            .query_map(rusqlite::params![ws], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if ids.is_empty() {
            return Ok(());
        }
        let attempt = get_meta(&conn, "last_sync_attempt")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        if let Ok(prev) = chrono::DateTime::parse_from_rfc3339(&attempt) {
            if chrono::Utc::now() - prev.with_timezone(&chrono::Utc) < chrono::Duration::seconds(20)
            {
                return Ok(());
            }
        }
        set_meta(&conn, "last_sync_attempt", &chrono::Utc::now().to_rfc3339())
            .map_err(|e| e.to_string())?;
        ids
    };

    push_workspace_records(state).await.map_err(|err| {
        let conn = state.conn.lock();
        let _ = set_meta(&conn, "last_sync_error", &err);
        err
    })?;

    let ts = chrono::Utc::now().to_rfc3339();
    let conn = state.conn.lock();
    for id in queued_ids {
        conn.execute(
            "UPDATE outbox SET status = 'sent', sent_at = ?1 WHERE id = ?2 AND status = 'queued'",
            rusqlite::params![ts, id],
        )
        .map_err(|e| e.to_string())?;
    }
    set_meta(&conn, "last_sync", &ts).map_err(|e| e.to_string())?;
    set_meta(&conn, "last_sync_error", "").map_err(|e| e.to_string())?;
    Ok(())
}

fn load_remote(state: &AppState, user_token: bool) -> Result<Remote, String> {
    let conn = state.conn.lock();
    let provider = get_meta(&conn, "identity_provider")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    if provider != PROVIDER_SUPABASE {
        return Err("not supabase".into());
    }
    let url = get_meta(&conn, "identity_supabase_url")
        .map_err(|e| e.to_string())?
        .filter(|u| !u.trim().is_empty())
        .ok_or_else(|| "Supabase project URL is required.".to_string())?;
    let anon = supabase_anon_key(&conn)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Supabase anon key is required.".to_string())?;
    let session = stored_session(&conn).map_err(|e| e.to_string())?;
    let (token, refresh) = if user_token {
        match session {
            Some(s) if !s.access_token.is_empty() => (s.access_token, s.refresh_token),
            _ => (String::new(), None),
        }
    } else {
        (anon.clone(), None)
    };
    Ok(Remote {
        base: url.trim().trim_end_matches('/').to_string(),
        anon,
        token,
        refresh,
    })
}

fn collect_snapshot(conn: &Connection, ws: &str) -> Result<Vec<(&'static str, Vec<Value>)>, String> {
    let workspaces = query_json(
        conn,
        "SELECT w.id, w.organization_id, o.name AS organization_name, w.name, w.updated_at
         FROM workspaces w
         JOIN organizations o ON o.id = w.organization_id
         WHERE w.id = ?1",
        ws,
    )?;
    let mut accounts = query_json(
        conn,
        "SELECT id, workspace_id, name AS full_name, role, title, email, updated_at
         FROM members WHERE workspace_id = ?1 AND deleted_at IS NULL",
        ws,
    )?;
    let admin_id = get_meta(conn, "setup_admin_id")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let admin_email = get_meta(conn, "setup_admin_email")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    for row in &mut accounts {
        let Some(obj) = row.as_object_mut() else { continue };
        let is_admin = obj.get("id").and_then(|v| v.as_str()) == Some(admin_id.as_str());
        let has_email = obj
            .get("email")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.trim().is_empty());
        if !has_email && is_admin && !admin_email.is_empty() {
            obj.insert("email".into(), Value::String(admin_email.clone()));
        }
    }
    let companies = query_json(
        conn,
        "SELECT id, workspace_id, name, domain, industry, notes, created_at, updated_at, version
         FROM companies WHERE workspace_id = ?1 AND deleted_at IS NULL",
        ws,
    )?;
    let contacts = query_json(
        conn,
        "SELECT id, workspace_id, company_id, name, email, title, phone, notes, created_at, updated_at, version
         FROM contacts WHERE workspace_id = ?1 AND deleted_at IS NULL",
        ws,
    )?;
    let cases = query_json(
        conn,
        "SELECT id, workspace_id, company_id, contact_id, assignee_id, title, description, status, priority, tags, created_at, updated_at, version
         FROM tickets WHERE workspace_id = ?1 AND deleted_at IS NULL",
        ws,
    )?;
    let issues = query_json(
        conn,
        "SELECT id, workspace_id, ticket_id, company_id, project_id, assignee_id, title, description, status, priority, blocked_reason, created_at, updated_at, version
         FROM issues WHERE workspace_id = ?1 AND deleted_at IS NULL",
        ws,
    )?;
    let documents = query_json(
        conn,
        "SELECT id, workspace_id, title, body, category, company_id, ticket_id, issue_id, visibility, created_at, updated_at, version
         FROM documents WHERE workspace_id = ?1 AND deleted_at IS NULL",
        ws,
    )?;
    let comments = query_json(
        conn,
        "SELECT id, workspace_id, entity_type, entity_id, author, body, visibility, created_at, updated_at, version
         FROM comments WHERE workspace_id = ?1 AND deleted_at IS NULL",
        ws,
    )?;
    let files = query_json(
        conn,
        "SELECT id, workspace_id, entity_type, entity_id, filename,
                workspace_id || '/' || stored_name AS storage_path,
                mime, size_bytes, created_at, updated_at, version
         FROM attachments WHERE workspace_id = ?1 AND deleted_at IS NULL",
        ws,
    )?;
    Ok(vec![
        ("confluity_workspaces", workspaces),
        ("confluity_accounts", accounts),
        ("confluity_companies", companies),
        ("confluity_contacts", contacts),
        ("confluity_cases", cases),
        ("confluity_issues", issues),
        ("confluity_documents", documents),
        ("confluity_comments", comments),
        ("confluity_files", files),
    ])
}

fn query_json(conn: &Connection, sql: &str, workspace_id: &str) -> Result<Vec<Value>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let names: Vec<String> = (0..stmt.column_count())
        .map(|i| stmt.column_name(i).unwrap_or("col").to_string())
        .collect();
    let mut rows = stmt
        .query(rusqlite::params![workspace_id])
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let mut obj = Map::new();
        for (i, name) in names.iter().enumerate() {
            let value = match row.get_ref(i).map_err(|e| e.to_string())? {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(n) => Value::from(n),
                ValueRef::Real(n) => Value::from(n),
                ValueRef::Text(bytes) => Value::String(String::from_utf8_lossy(bytes).into_owned()),
                ValueRef::Blob(_) => Value::Null,
            };
            obj.insert(name.clone(), value);
        }
        out.push(Value::Object(obj));
    }
    Ok(out)
}

async fn upsert_table(
    state: &AppState,
    remote: &mut Remote,
    table: &str,
    rows: Vec<Value>,
) -> Result<(), String> {
    if rows.is_empty() {
        return Ok(());
    }
    for chunk in rows.chunks(100) {
        let url = format!("{}/rest/v1/{table}?on_conflict=id", remote.base);
        let body = serde_json::to_vec(chunk).map_err(|e| e.to_string())?;
        let resp = authed(
            state,
            remote,
            reqwest::Method::POST,
            url,
            Some("application/json"),
            Some(body),
            &[("Prefer", "resolution=merge-duplicates,return=minimal")],
        )
        .await?;
        let status = resp.status();
        if status.is_success() {
            continue;
        }
        let text = resp.text().await.unwrap_or_default();
        return Err(interpret_rest_error(status, &text));
    }
    Ok(())
}

async fn fetch_rows(state: &AppState, remote: &mut Remote, url: String) -> Result<Vec<Value>, String> {
    let resp = authed(state, remote, reqwest::Method::GET, url, None, None, &[]).await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(interpret_rest_error(status, &body));
    }
    serde_json::from_str(&body).map_err(|e| format!("Unexpected account response: {e}"))
}

async fn authed(
    state: &AppState,
    remote: &mut Remote,
    method: reqwest::Method,
    url: String,
    content_type: Option<&str>,
    body: Option<Vec<u8>>,
    extra: &[(&str, &str)],
) -> Result<reqwest::Response, String> {
    let resp = send_once(remote, method.clone(), url.clone(), content_type, body.clone(), extra).await?;
    if resp.status().as_u16() != 401 {
        return Ok(resp);
    }
    refresh_token(state, remote).await?;
    send_once(remote, method, url, content_type, body, extra).await
}

async fn send_once(
    remote: &Remote,
    method: reqwest::Method,
    url: String,
    content_type: Option<&str>,
    body: Option<Vec<u8>>,
    extra: &[(&str, &str)],
) -> Result<reqwest::Response, String> {
    let client = reqwest::Client::new();
    let mut req = client
        .request(method, url)
        .header("apikey", &remote.anon)
        .header("Authorization", format!("Bearer {}", remote.token));
    if let Some(ct) = content_type {
        req = req.header("content-type", ct);
    }
    for (name, value) in extra {
        req = req.header(*name, *value);
    }
    if let Some(body) = body {
        req = req.body(body);
    }
    req.send().await.map_err(|e| format!("Could not reach Supabase: {e}"))
}

async fn refresh_token(state: &AppState, remote: &mut Remote) -> Result<(), String> {
    let refresh = remote
        .refresh
        .clone()
        .filter(|t| !t.is_empty())
        .ok_or_else(|| "Sign in again to send workspace data.".to_string())?;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!(
            "{}/auth/v1/token?grant_type=refresh_token",
            remote.base
        ))
        .header("apikey", &remote.anon)
        .header("Authorization", format!("Bearer {}", remote.anon))
        .json(&serde_json::json!({ "refresh_token": refresh }))
        .send()
        .await
        .map_err(|e| format!("Could not refresh the Supabase session: {e}"))?;
    if !resp.status().is_success() {
        return Err("Sign in again to send workspace data.".into());
    }
    let parsed: Value = resp
        .json()
        .await
        .map_err(|e| format!("Unexpected refresh response: {e}"))?;
    let access = parsed
        .get("access_token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if access.is_empty() {
        return Err("Sign in again to send workspace data.".into());
    }
    let new_refresh = parsed
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| remote.refresh.clone());
    let user_id = parsed
        .pointer("/user/id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let email = parsed
        .pointer("/user/email")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let expires_in = parsed.get("expires_in").and_then(|v| v.as_i64());
    let expires_at = expires_in.map(|secs| (chrono::Utc::now() + chrono::Duration::seconds(secs)).to_rfc3339());
    let session = StoredIdentitySession {
        provider: PROVIDER_SUPABASE.into(),
        user_id,
        email,
        access_token: access.clone(),
        refresh_token: new_refresh.clone(),
        expires_at,
    };
    {
        let conn = state.conn.lock();
        write_session(&conn, Some(&session)).map_err(|e| e.to_string())?;
    }
    remote.token = access;
    remote.refresh = new_refresh;
    Ok(())
}

fn interpret_rest_error(status: reqwest::StatusCode, body: &str) -> String {
    if schema_missing(status, body) {
        return SCHEMA_HINT.into();
    }
    let lower = body.to_ascii_lowercase();
    if lower.contains("row-level security") || lower.contains("permission denied") {
        return SCHEMA_HINT.into();
    }
    let snippet: String = body.chars().take(180).collect();
    if snippet.is_empty() {
        format!("Supabase rejected the workspace data ({status}).")
    } else {
        format!("Supabase rejected the workspace data ({status}). {snippet}")
    }
}

fn schema_missing(status: reqwest::StatusCode, body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    status.as_u16() == 404
        && (lower.contains("pgrst205")
            || lower.contains("schema cache")
            || lower.contains("does not exist")
            || lower.contains("not found"))
}

fn encode_query(value: &str) -> String {
    let mut out = String::new();
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn encode_path(value: &str) -> String {
    value
        .split('/')
        .map(encode_query)
        .collect::<Vec<_>>()
        .join("/")
}
