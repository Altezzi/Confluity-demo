-- Confluity workspace store.
-- Run this once in the Supabase SQL editor for the project Confluity should use.
-- Accounts, cases, files, and the rest of the workspace land in these tables.

create table if not exists public.confluity_workspaces (
  id text primary key,
  organization_id text,
  organization_name text,
  name text not null,
  updated_at text
);

create table if not exists public.confluity_accounts (
  id text primary key,
  workspace_id text not null,
  email text,
  full_name text not null,
  role text,
  title text,
  updated_at text
);

create table if not exists public.confluity_companies (
  id text primary key,
  workspace_id text not null,
  name text not null,
  domain text,
  industry text,
  notes text,
  created_at text,
  updated_at text,
  version bigint not null default 1
);

create table if not exists public.confluity_contacts (
  id text primary key,
  workspace_id text not null,
  company_id text,
  name text not null,
  email text,
  title text,
  phone text,
  notes text,
  created_at text,
  updated_at text,
  version bigint not null default 1
);

create table if not exists public.confluity_cases (
  id text primary key,
  workspace_id text not null,
  company_id text,
  contact_id text,
  assignee_id text,
  title text not null,
  description text,
  status text,
  priority text,
  tags text,
  created_at text,
  updated_at text,
  version bigint not null default 1
);

create table if not exists public.confluity_issues (
  id text primary key,
  workspace_id text not null,
  ticket_id text,
  company_id text,
  project_id text,
  assignee_id text,
  title text not null,
  description text,
  status text,
  priority text,
  blocked_reason text,
  created_at text,
  updated_at text,
  version bigint not null default 1
);

create table if not exists public.confluity_documents (
  id text primary key,
  workspace_id text not null,
  title text not null,
  body text,
  category text,
  company_id text,
  ticket_id text,
  issue_id text,
  visibility text,
  created_at text,
  updated_at text,
  version bigint not null default 1
);

create table if not exists public.confluity_comments (
  id text primary key,
  workspace_id text not null,
  entity_type text not null,
  entity_id text not null,
  author text,
  body text,
  visibility text,
  created_at text,
  updated_at text,
  version bigint not null default 1
);

create table if not exists public.confluity_files (
  id text primary key,
  workspace_id text not null,
  entity_type text not null,
  entity_id text not null,
  filename text not null,
  storage_path text not null,
  mime text,
  size_bytes bigint not null default 0,
  created_at text,
  updated_at text,
  version bigint not null default 1
);

create index if not exists confluity_accounts_email_idx on public.confluity_accounts (email);
create index if not exists confluity_cases_workspace_idx on public.confluity_cases (workspace_id);
create index if not exists confluity_files_entity_idx on public.confluity_files (entity_type, entity_id);

alter table public.confluity_workspaces enable row level security;
alter table public.confluity_accounts enable row level security;
alter table public.confluity_companies enable row level security;
alter table public.confluity_contacts enable row level security;
alter table public.confluity_cases enable row level security;
alter table public.confluity_issues enable row level security;
alter table public.confluity_documents enable row level security;
alter table public.confluity_comments enable row level security;
alter table public.confluity_files enable row level security;

do $$
declare
  t text;
begin
  foreach t in array array[
    'confluity_workspaces',
    'confluity_accounts',
    'confluity_companies',
    'confluity_contacts',
    'confluity_cases',
    'confluity_issues',
    'confluity_documents',
    'confluity_comments',
    'confluity_files'
  ]
  loop
    execute format('drop policy if exists %I on public.%I', t || '_all', t);
    execute format(
      'create policy %I on public.%I for all to authenticated using (true) with check (true)',
      t || '_all', t
    );
    execute format(
      'grant select, insert, update, delete on public.%I to authenticated',
      t
    );
  end loop;
end $$;

insert into storage.buckets (id, name, public)
values ('confluity-files', 'confluity-files', false)
on conflict (id) do nothing;

drop policy if exists confluity_files_read on storage.objects;
create policy confluity_files_read on storage.objects
  for select to authenticated
  using (bucket_id = 'confluity-files');

drop policy if exists confluity_files_insert on storage.objects;
create policy confluity_files_insert on storage.objects
  for insert to authenticated
  with check (bucket_id = 'confluity-files');

drop policy if exists confluity_files_update on storage.objects;
create policy confluity_files_update on storage.objects
  for update to authenticated
  using (bucket_id = 'confluity-files');
