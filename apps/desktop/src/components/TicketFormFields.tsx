import type { Company, Contact, Member, TicketForm } from "@confluity/shared";
import type { TicketDraft } from "../lib/ticketDraft";

const PRIORITIES = ["low", "medium", "high", "urgent"] as const;
const STATUSES = ["open", "in_progress", "waiting", "resolved", "closed"] as const;

export function TicketFormFields({
  form,
  draft,
  onChange,
  companies,
  contacts,
  members,
}: {
  form: TicketForm;
  draft: TicketDraft;
  onChange: (next: TicketDraft) => void;
  companies: Company[];
  contacts: Contact[];
  members: Member[];
}) {
  const companyContacts = contacts.filter((contact) => contact.company_id === draft.companyId);

  function setCustom(fieldId: string, value: string) {
    onChange({ ...draft, custom: { ...draft.custom, [fieldId]: value } });
  }

  return (
    <>
      {form.fields.map((placement) => {
        const field = placement.field;
        const label = `${field.label}${placement.required ? " *" : ""}`;
        const hint = field.hint ? <p className="meta ticket-field-hint">{field.hint}</p> : null;

        if (field.kind === "system") {
          if (field.key === "title") {
            return (
              <div className="form-row" key={field.id}>
                <label>{label}</label>
                <input
                  value={draft.title}
                  onChange={(e) => onChange({ ...draft, title: e.target.value })}
                />
                {hint}
              </div>
            );
          }
          if (field.key === "company") {
            return (
              <div className="form-row" key={field.id}>
                <label>{label}</label>
                <select
                  value={draft.companyId}
                  onChange={(e) => onChange({ ...draft, companyId: e.target.value, contactId: "" })}
                >
                  <option value="">Select a company</option>
                  {companies.map((company) => (
                    <option key={company.id} value={company.id}>
                      {company.name}
                    </option>
                  ))}
                </select>
                {hint}
              </div>
            );
          }
          if (field.key === "contact") {
            return (
              <div className="form-row" key={field.id}>
                <label>{label}</label>
                <select
                  value={draft.contactId}
                  onChange={(e) => onChange({ ...draft, contactId: e.target.value })}
                >
                  <option value="">None</option>
                  {companyContacts.map((contact) => (
                    <option key={contact.id} value={contact.id}>
                      {contact.name}
                    </option>
                  ))}
                </select>
                {hint}
              </div>
            );
          }
          if (field.key === "priority") {
            return (
              <div className="form-row" key={field.id}>
                <label>{label}</label>
                <select
                  value={draft.priority}
                  onChange={(e) =>
                    onChange({ ...draft, priority: e.target.value as TicketDraft["priority"] })
                  }
                >
                  {PRIORITIES.map((priority) => (
                    <option key={priority} value={priority}>
                      {priority.replace("_", " ")}
                    </option>
                  ))}
                </select>
                {hint}
              </div>
            );
          }
          if (field.key === "status") {
            return (
              <div className="form-row" key={field.id}>
                <label>{label}</label>
                <select
                  value={draft.status}
                  onChange={(e) =>
                    onChange({ ...draft, status: e.target.value as TicketDraft["status"] })
                  }
                >
                  {STATUSES.map((status) => (
                    <option key={status} value={status}>
                      {status.replace("_", " ")}
                    </option>
                  ))}
                </select>
                {hint}
              </div>
            );
          }
          if (field.key === "assignee") {
            return (
              <div className="form-row" key={field.id}>
                <label>{label}</label>
                <select
                  value={draft.assigneeId}
                  onChange={(e) => onChange({ ...draft, assigneeId: e.target.value })}
                >
                  <option value="">Unassigned</option>
                  {members.map((member) => (
                    <option key={member.id} value={member.id}>
                      {member.name}
                    </option>
                  ))}
                </select>
                {hint}
              </div>
            );
          }
          if (field.key === "tags") {
            return (
              <div className="form-row" key={field.id}>
                <label>{label}</label>
                <input
                  value={draft.tags}
                  onChange={(e) => onChange({ ...draft, tags: e.target.value })}
                  placeholder="firmware, sync"
                />
                {hint}
              </div>
            );
          }
          return (
            <div className="form-row" key={field.id}>
              <label>{label}</label>
              <textarea
                value={draft.description}
                onChange={(e) => onChange({ ...draft, description: e.target.value })}
              />
              {hint}
            </div>
          );
        }

        if (field.kind === "textarea") {
          return (
            <div className="form-row" key={field.id}>
              <label>{label}</label>
              <textarea
                value={draft.custom[field.id] ?? ""}
                onChange={(e) => setCustom(field.id, e.target.value)}
              />
              {hint}
            </div>
          );
        }
        if (field.kind === "dropdown") {
          return (
            <div className="form-row" key={field.id}>
              <label>{label}</label>
              <select
                value={draft.custom[field.id] ?? ""}
                onChange={(e) => setCustom(field.id, e.target.value)}
              >
                <option value="">Select</option>
                {field.options.map((option) => (
                  <option key={option} value={option}>
                    {option}
                  </option>
                ))}
              </select>
              {hint}
            </div>
          );
        }
        if (field.kind === "checkbox") {
          return (
            <label className="login-remember" key={field.id}>
              <input
                type="checkbox"
                checked={(draft.custom[field.id] ?? "") === "true"}
                onChange={(e) => setCustom(field.id, e.target.checked ? "true" : "false")}
              />
              {label}
            </label>
          );
        }
        return (
          <div className="form-row" key={field.id}>
            <label>{label}</label>
            <input
              type={field.kind === "number" ? "number" : "text"}
              value={draft.custom[field.id] ?? ""}
              onChange={(e) => setCustom(field.id, e.target.value)}
            />
            {hint}
          </div>
        );
      })}
    </>
  );
}
