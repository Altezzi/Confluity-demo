import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import type { Member, OrgCategory, Position, Role } from "@confluity/shared";
import { api } from "../../lib/api";
import { useApp } from "../../context/AppContext";

export function AdminDashboard() {
  const { settings, can } = useApp();
  const [members, setMembers] = useState<Member[]>([]);
  const [roles, setRoles] = useState<Role[]>([]);
  const [categories, setCategories] = useState<OrgCategory[]>([]);
  const [positions, setPositions] = useState<Position[]>([]);
  const [integrationsOn, setIntegrationsOn] = useState(false);

  useEffect(() => {
    void (async () => {
      const [memberRows, roleRows, cats, posRows] = await Promise.all([
        api.listMembers().catch(() => [] as Member[]),
        api.listRoles().catch(() => [] as Role[]),
        api.listOrgCategories().catch(() => [] as OrgCategory[]),
        api.listPositions().catch(() => [] as Position[]),
      ]);
      setMembers(memberRows);
      setRoles(roleRows);
      setCategories(cats);
      setPositions(posRows);
      try {
        const obs = await api.getObsSettings();
        setIntegrationsOn(
          Boolean(obs.datadog_api_key || obs.grafana_url || obs.cloudwatch_region || obs.outbound_enabled),
        );
      } catch {
        setIntegrationsOn(false);
      }
    })();
  }, []);

  const org = settings?.organization_name || "Your company";
  const storage = settings?.storage_mode || "local";
  const storageLabel = storage.charAt(0).toUpperCase() + storage.slice(1);

  const stats = [
    can("settings.manage_members")
      ? { to: "/admin/members", value: String(members.length), label: "Members" }
      : null,
    can("settings.manage_roles")
      ? { to: "/admin/roles", value: String(roles.length), label: "Roles" }
      : null,
    can("settings.manage_roles")
      ? { to: "/admin/positions", value: String(positions.length), label: "Org chart" }
      : null,
    can("settings.data_server")
      ? { to: "/admin/data", value: storageLabel, label: "Data & sync" }
      : null,
  ].filter((stat) => stat !== null);

  const steps = [
    {
      done: categories.length > 0 && positions.length > 0,
      label: "Org chart",
      detail: `${categories.length} categories · ${positions.length} positions`,
      to: "/admin/positions",
      allowed: can("settings.manage_roles"),
    },
    {
      done: roles.length > 0,
      label: "Roles & permissions",
      detail: `${roles.length} roles defining what people can see`,
      to: "/admin/roles",
      allowed: can("settings.manage_roles"),
    },
    {
      done: members.length > 1,
      label: "Invite the company",
      detail: `${members.length} members in this workspace`,
      to: "/admin/members",
      allowed: can("settings.manage_members"),
    },
    {
      done: integrationsOn || Boolean(settings?.server_url),
      label: "Company integrations",
      detail: "Shared for everyone who opens this workspace",
      to: "/admin/integrations",
      allowed: can("settings.integrations"),
    },
    {
      done: Boolean(settings?.server_url) || storage !== "local",
      label: "Data & sync",
      detail: `Mode: ${storageLabel}`,
      to: "/admin/data",
      allowed: can("settings.data_server"),
    },
  ];
  const setupOpen = steps.some((step) => !step.done);

  return (
    <div className="admin-dash">
      <div className="home-hero">
        <div className="home-hero-copy">
          <h2>{org}</h2>
          <p>People, roles, and connections for this workspace.</p>
        </div>
        <div className="home-hero-actions">
          {can("settings.manage_members") && (
            <Link className="btn btn-primary" to="/admin/members">
              Invite the company
            </Link>
          )}
          {can("settings.integrations") && (
            <Link className="btn" to="/admin/integrations">
              Integrations
            </Link>
          )}
        </div>
      </div>

      {stats.length > 0 && (
        <div className="stat-row" data-count={stats.length}>
          {stats.map((stat) => (
            <Link key={stat.label} className="stat" to={stat.to}>
              <div className="value">{stat.value}</div>
              <div className="label">{stat.label}</div>
            </Link>
          ))}
        </div>
      )}

      {setupOpen && (
        <section className="panel">
          <div className="panel-head">
            <h2>Setup</h2>
          </div>
          <div className="admin-setup-grid">
            {steps.map((step) => {
              const body = (
                <>
                  <span className={`admin-setup-check${step.done ? " done" : ""}`} aria-hidden>
                    {step.done ? "✓" : "○"}
                  </span>
                  <span>
                    <span className="title">{step.label}</span>
                    <span className="meta">{step.detail}</span>
                  </span>
                </>
              );
              return step.allowed ? (
                <Link key={step.to} className="admin-setup-card" to={step.to}>
                  {body}
                </Link>
              ) : (
                <div key={step.to} className="admin-setup-card static">
                  {body}
                </div>
              );
            })}
          </div>
        </section>
      )}
    </div>
  );
}
