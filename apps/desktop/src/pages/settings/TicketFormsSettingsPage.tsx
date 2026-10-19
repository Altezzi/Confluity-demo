import { useEffect, useMemo, useState } from "react";
import type { TicketField, TicketForm, TicketFormField } from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";

const FIELD_TYPES = [
  { id: "text", label: "Short text" },
  { id: "textarea", label: "Multi-line" },
  { id: "dropdown", label: "Dropdown" },
  { id: "checkbox", label: "Checkbox" },
  { id: "number", label: "Number" },
] as const;

function blankForm(from: TicketForm | undefined): TicketForm {
  const keep = new Set(["title", "company", "description"]);
  return {
    id: "",
    name: "New form",
    description: "",
    active: true,
    is_default: false,
    position: 0,
    fields: (from?.fields ?? []).filter((field) => keep.has(field.field.key)),
  };
}

export function TicketFormsSettingsPage() {
  const { can } = useApp();
  const [forms, setForms] = useState<TicketForm[]>([]);
  const [catalog, setCatalog] = useState<TicketField[]>([]);
  const [draft, setDraft] = useState<TicketForm | null>(null);
  const [addFieldId, setAddFieldId] = useState("");
  const [newLabel, setNewLabel] = useState("");
  const [newKind, setNewKind] = useState<(typeof FIELD_TYPES)[number]["id"]>("dropdown");
  const [newOptions, setNewOptions] = useState("Question\nIncident\nTask");
  const [newHint, setNewHint] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [saved, setSaved] = useState(false);

  const available = useMemo(() => {
    const used = new Set(draft?.fields.map((field) => field.field_id) ?? []);
    return catalog.filter((field) => field.active && !used.has(field.id));
  }, [catalog, draft]);

  async function reload(preferId?: string) {
    const [nextForms, nextFields] = await Promise.all([
      api.listTicketForms(),
      api.listTicketFields(),
    ]);
    setForms(nextForms);
    setCatalog(nextFields);
    if (!preferId) return;
    const selected = nextForms.find((form) => form.id === preferId);
    if (selected) setDraft(structuredClone(selected));
  }

  useEffect(() => {
    void reload().catch((e) => setError(e instanceof Error ? e.message : String(e)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (!can("settings.workspace_admin")) {
    return <div className="empty">You do not have permission to edit ticket forms.</div>;
  }

  function updateField(index: number, patch: Partial<TicketFormField> & { label?: string; hint?: string | null; options?: string[] }) {
    if (!draft) return;
    setSaved(false);
    const fields = draft.fields.map((field, i) => {
      if (i !== index) return field;
      return {
        ...field,
        required: patch.required ?? field.required,
        field: {
          ...field.field,
          label: patch.label ?? field.field.label,
          hint: patch.hint === undefined ? field.field.hint : patch.hint,
          options: patch.options ?? field.field.options,
        },
      };
    });
    setDraft({ ...draft, fields });
  }

  function move(index: number, direction: -1 | 1) {
    if (!draft) return;
    const next = index + direction;
    if (next < 0 || next >= draft.fields.length) return;
    const fields = [...draft.fields];
    const [item] = fields.splice(index, 1);
    fields.splice(next, 0, item);
    setDraft({ ...draft, fields });
    setSaved(false);
  }

  function remove(index: number) {
    if (!draft) return;
    const field = draft.fields[index];
    if (field.field.key === "title" || field.field.key === "company") return;
    setDraft({ ...draft, fields: draft.fields.filter((_, i) => i !== index) });
    setSaved(false);
  }

  async function save() {
    if (!draft || busy) return;
    setBusy(true);
    setError(null);
    setSaved(false);
    try {
      const savedForm = await api.saveTicketForm({
        id: draft.id || null,
        name: draft.name.trim(),
        description: draft.description,
        active: draft.active,
        is_default: draft.is_default,
        fields: draft.fields.map((field) => ({
          field_id: field.field_id,
          required: field.required,
          label: field.field.label,
          hint: field.field.hint,
          options: field.field.options,
        })),
      });
      await reload(savedForm.id);
      setSaved(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function addCustomField() {
    if (!newLabel.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      const created = await api.createTicketField({
        label: newLabel.trim(),
        kind: newKind,
        options: newKind === "dropdown" ? newOptions.split("\n") : [],
        hint: newHint.trim() || null,
        form_id: null,
      });
      setNewLabel("");
      setNewHint("");
      setCatalog((curr) => [...curr, created]);
      setDraft((curr) =>
        curr
          ? {
              ...curr,
              fields: [
                ...curr.fields,
                { field_id: created.id, position: curr.fields.length, required: false, field: created },
              ],
            }
          : curr,
      );
      setSaved(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function removeForm(form: TicketForm) {
    if (!confirm(`Delete the “${form.name}” form? Existing tickets keep their saved values.`)) return;
    setBusy(true);
    setError(null);
    try {
      const next = await api.deleteTicketForm(form.id);
      setForms(next);
      setDraft(null);
      setSaved(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function deleteCustom(field: TicketFormField) {
    if (field.field.system) return;
    if (!confirm(`Delete the “${field.field.label}” field from every form?`)) return;
    setBusy(true);
    setError(null);
    try {
      await api.deleteTicketField(field.field.id);
      if (draft?.id) await reload(draft.id);
      else {
        setCatalog((curr) => curr.filter((item) => item.id !== field.field.id));
        setDraft((curr) =>
          curr ? { ...curr, fields: curr.fields.filter((item) => item.field.id !== field.field.id) } : curr,
        );
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  function startNewForm() {
    setDraft(blankForm(forms.find((form) => form.is_default) ?? forms[0]));
    setSaved(false);
  }

  const newFormInHeader = !draft && forms.length >= 5;

  return (
    <div className="panel">
      <div className="panel-head">
        <h2>Ticket forms</h2>
        {newFormInHeader && (
          <button className="btn" type="button" onClick={startNewForm}>
            New form
          </button>
        )}
      </div>
      <p className="meta" style={{ padding: "0 18px 12px", margin: 0 }}>
        Choose which fields appear when someone opens a ticket, the order they show in, and which
        ones are required. This is the same idea as Zendesk ticket forms.
      </p>
      {error && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{error}</p>}

      <div className="ticket-forms-layout">
        {!draft && (
          <>
            <div className="ticket-form-list">
              {forms.map((form) => (
                <button
                  key={form.id}
                  type="button"
                  className="list-item"
                  onClick={() => {
                    setDraft(structuredClone(form));
                    setSaved(false);
                  }}
                >
                  <div>
                    <div className="title">{form.name}</div>
                    <div className="meta">
                      {form.is_default ? "Default" : form.active ? "Active" : "Hidden"}
                      {` · ${form.fields.length} fields`}
                    </div>
                  </div>
                  <span className="meta">Edit</span>
                </button>
              ))}
              {forms.length === 0 && <div className="empty">No ticket forms yet</div>}
            </div>
            {forms.length < 5 && (
              <button className="btn" type="button" onClick={startNewForm}>
                New form
              </button>
            )}
          </>
        )}

        {draft && (
          <div className="ticket-form-editor">
            <button
              className="btn"
              type="button"
              onClick={() => {
                setDraft(null);
                setSaved(false);
              }}
            >
              ← Forms
            </button>
            <div className="form-row">
              <label>Form name</label>
              <input
                value={draft.name}
                onChange={(e) => {
                  setDraft({ ...draft, name: e.target.value });
                  setSaved(false);
                }}
              />
            </div>
            <div className="form-row">
              <label>Description</label>
              <input
                value={draft.description ?? ""}
                onChange={(e) => {
                  setDraft({ ...draft, description: e.target.value });
                  setSaved(false);
                }}
                placeholder="Shown to admins only"
              />
            </div>
            <div className="ticket-form-flags">
              <label className="login-remember">
                <input
                  type="checkbox"
                  checked={draft.active || draft.is_default}
                  disabled={draft.is_default}
                  onChange={(e) => {
                    setDraft({ ...draft, active: e.target.checked });
                    setSaved(false);
                  }}
                />
                Active
              </label>
              <label className="login-remember">
                <input
                  type="checkbox"
                  checked={draft.is_default}
                  onChange={(e) => {
                    setDraft({ ...draft, is_default: e.target.checked, active: e.target.checked || draft.active });
                    setSaved(false);
                  }}
                />
                Default form
              </label>
            </div>

            <div className="nav-label">Fields</div>
            <div className="ticket-form-fields">
              {draft.fields.map((field, index) => {
                const locked = field.field.key === "title" || field.field.key === "company";
                return (
                  <div className="ticket-form-field" key={field.field_id}>
                    <div>
                      <input
                        value={field.field.label}
                        onChange={(e) => updateField(index, { label: e.target.value })}
                        aria-label="Field label"
                      />
                      <div className="meta">
                        {field.field.system ? "Built-in" : field.field.kind}
                        {field.field.options.length > 0 ? ` · ${field.field.options.join(", ")}` : ""}
                      </div>
                    </div>
                    <div className="ticket-form-field-actions">
                      <label className="login-remember">
                        <input
                          type="checkbox"
                          checked={locked || field.required}
                          disabled={locked}
                          onChange={(e) => updateField(index, { required: e.target.checked })}
                        />
                        Required
                      </label>
                      <button className="btn btn-ghost" type="button" onClick={() => move(index, -1)} disabled={index === 0}>
                        Up
                      </button>
                      <button
                        className="btn btn-ghost"
                        type="button"
                        onClick={() => move(index, 1)}
                        disabled={index === draft.fields.length - 1}
                      >
                        Down
                      </button>
                      {!locked && (
                        <button className="btn btn-ghost" type="button" onClick={() => remove(index)}>
                          Remove
                        </button>
                      )}
                      {!field.field.system && (
                        <button className="btn btn-ghost" type="button" onClick={() => void deleteCustom(field)}>
                          Delete
                        </button>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>

            <div className="form-row">
              <label>Add an existing field</label>
              <div className="ticket-form-add">
                <select value={addFieldId} onChange={(e) => setAddFieldId(e.target.value)}>
                  <option value="">Choose a field</option>
                  {available.map((field) => (
                    <option key={field.id} value={field.id}>
                      {field.label}
                    </option>
                  ))}
                </select>
                <button
                  className="btn"
                  type="button"
                  disabled={!addFieldId}
                  onClick={() => {
                    const field = catalog.find((item) => item.id === addFieldId);
                    if (!field) return;
                    setDraft({
                      ...draft,
                      fields: [
                        ...draft.fields,
                        { field_id: field.id, position: draft.fields.length, required: false, field },
                      ],
                    });
                    setAddFieldId("");
                    setSaved(false);
                  }}
                >
                  Add
                </button>
              </div>
            </div>

            <div className="settings-subblock">
              <div className="settings-subblock-head">
                <h4 style={{ margin: 0 }}>New custom field</h4>
              </div>
              <div className="form-grid" style={{ gap: 10 }}>
                <div className="form-row">
                  <label>Name</label>
                  <input value={newLabel} onChange={(e) => setNewLabel(e.target.value)} placeholder="Ticket type" />
                </div>
                <div className="form-row">
                  <label>Type</label>
                  <select value={newKind} onChange={(e) => setNewKind(e.target.value as typeof newKind)}>
                    {FIELD_TYPES.map((type) => (
                      <option key={type.id} value={type.id}>
                        {type.label}
                      </option>
                    ))}
                  </select>
                </div>
                {newKind === "dropdown" && (
                  <div className="form-row">
                    <label>Options, one per line</label>
                    <textarea value={newOptions} onChange={(e) => setNewOptions(e.target.value)} />
                  </div>
                )}
                <div className="form-row">
                  <label>Hint</label>
                  <input value={newHint} onChange={(e) => setNewHint(e.target.value)} placeholder="Optional help text" />
                </div>
                <button className="btn" type="button" disabled={busy || !newLabel.trim()} onClick={() => void addCustomField()}>
                  Add custom field
                </button>
              </div>
            </div>

            <div className="form-actions">
              <button className="btn btn-primary" type="button" disabled={busy || !draft.name.trim()} onClick={() => void save()}>
                Save form
              </button>
              {draft.id && forms.length > 1 && (
                <button className="btn" type="button" disabled={busy} onClick={() => void removeForm(draft)}>
                  Delete form
                </button>
              )}
              {saved && <span className="badge open">Saved</span>}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
