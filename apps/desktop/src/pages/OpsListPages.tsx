import { useEffect, useMemo, useState, type FormEvent, type ReactNode } from "react";
import { Link, useNavigate } from "react-router-dom";
import type { Asset, Company, Product, Rma, Ticket } from "@confluity/shared";
import { api, formatLabel, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

function ListShell({
  eyebrow,
  title,
  subtitle,
  backTo,
  newLabel,
  searchPlaceholder,
  query,
  onQuery,
  onNew,
  children,
  modalOpen,
  modalTitle,
  onClose,
  form,
}: {
  eyebrow: string;
  title: string;
  subtitle: string;
  backTo: string;
  newLabel: string;
  searchPlaceholder: string;
  query: string;
  onQuery: (v: string) => void;
  onNew: () => void;
  children: ReactNode;
  modalOpen: boolean;
  modalTitle: string;
  onClose: () => void;
  form: ReactNode;
}) {
  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">{eyebrow}</div>
          <h1>{title}</h1>
          <p>{subtitle}</p>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn btn-primary" type="button" onClick={onNew}>
            {newLabel}
          </button>
          <Link className="btn" to={backTo}>
            Back
          </Link>
        </div>
      </div>

      <div className="panel">
        <div className="docs-toolbar">
          <input
            className="docs-search"
            value={query}
            onChange={(e) => onQuery(e.target.value)}
            placeholder={searchPlaceholder}
          />
        </div>
        <div className="list">{children}</div>
      </div>

      {modalOpen && (
        <div className="modal-backdrop" onClick={onClose}>
          <div
            className="modal modal-form"
            role="dialog"
            aria-modal="true"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-head">
              <h2>{modalTitle}</h2>
              <button className="btn btn-ghost" type="button" onClick={onClose}>
                Close
              </button>
            </div>
            {form}
          </div>
        </div>
      )}
    </div>
  );
}

export function ProductsPage() {
  const navigate = useNavigate();
  const { setFocus } = useApp();
  const [products, setProducts] = useState<Product[]>([]);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [name, setName] = useState("");
  const [sku, setSku] = useState("");

  async function reload() {
    setProducts(await api.listProducts());
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Products" });
    void reload();
  }, [setFocus]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return products;
    return products.filter((p) =>
      `${p.name} ${p.sku ?? ""} ${p.description ?? ""}`.toLowerCase().includes(q),
    );
  }, [products, query]);

  function closeModal() {
    setOpen(false);
    setName("");
    setSku("");
  }

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    if (!name.trim() || busy) return;
    setBusy(true);
    try {
      const product = await api.createProduct({
        name: name.trim(),
        sku: sku || null,
        inventory_qty: 0,
      });
      closeModal();
      await reload();
      navigate(recordPath("product", product.id));
    } finally {
      setBusy(false);
    }
  }

  return (
    <ListShell
      eyebrow="Operations"
      title="Products"
      subtitle="Catalog SKUs and inventory counts."
      backTo="/operations"
      newLabel="New product"
      searchPlaceholder="Search products…"
      query={query}
      onQuery={setQuery}
      onNew={() => setOpen(true)}
      modalOpen={open}
      modalTitle="New product"
      onClose={closeModal}
      form={
        <form className="form-grid" onSubmit={onCreate}>
          <div className="form-row">
            <label>Name</label>
            <input
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Escape" && closeModal()}
            />
          </div>
          <div className="form-row">
            <label>SKU</label>
            <input value={sku} onChange={(e) => setSku(e.target.value)} />
          </div>
          <div className="modal-actions">
            <button className="btn" type="button" onClick={closeModal}>
              Cancel
            </button>
            <button className="btn btn-primary" type="submit" disabled={busy || !name.trim()}>
              Create product
            </button>
          </div>
        </form>
      }
    >
      {filtered.map((p) => (
        <Link key={p.id} className="list-item" to={recordPath("product", p.id)}>
          <div>
            <div className="title">{p.name}</div>
            <div className="meta">
              {p.sku || "No SKU"} · qty {p.inventory_qty}
            </div>
          </div>
        </Link>
      ))}
      {filtered.length === 0 && (
        <div className="empty">{query ? "No matching products" : "No products yet"}</div>
      )}
    </ListShell>
  );
}

export function AssetsPage() {
  const navigate = useNavigate();
  const { setFocus } = useApp();
  const [assets, setAssets] = useState<Asset[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  const [companies, setCompanies] = useState<Company[]>([]);
  const [tickets, setTickets] = useState<Ticket[]>([]);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [name, setName] = useState("");
  const [serial, setSerial] = useState("");
  const [productId, setProductId] = useState("");
  const [companyId, setCompanyId] = useState("");
  const [ticketId, setTicketId] = useState("");

  async function reload() {
    const [a, p, c, t] = await Promise.all([
      api.listAssets(),
      api.listProducts(),
      api.listCompanies(),
      api.listTickets(),
    ]);
    setAssets(a);
    setProducts(p);
    setCompanies(c);
    setTickets(t);
    if (!productId && p[0]) setProductId(p[0].id);
    if (!companyId && c[0]) setCompanyId(c[0].id);
    if (!ticketId && t[0]) setTicketId(t[0].id);
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Assets" });
    void reload();
  }, [setFocus]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return assets;
    return assets.filter((a) =>
      `${a.name} ${a.serial ?? ""} ${a.notes ?? ""}`.toLowerCase().includes(q),
    );
  }, [assets, query]);

  function closeModal() {
    setOpen(false);
    setName("");
    setSerial("");
  }

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    if (!name.trim() || busy) return;
    setBusy(true);
    try {
      const asset = await api.createAsset({
        name: name.trim(),
        serial: serial || null,
        product_id: productId || null,
        company_id: companyId || null,
        ticket_id: ticketId || null,
      });
      closeModal();
      await reload();
      navigate(recordPath("asset", asset.id));
    } finally {
      setBusy(false);
    }
  }

  return (
    <ListShell
      eyebrow="Operations"
      title="Assets"
      subtitle="Serialized units, warranty, and customer links."
      backTo="/operations"
      newLabel="New asset"
      searchPlaceholder="Search assets…"
      query={query}
      onQuery={setQuery}
      onNew={() => setOpen(true)}
      modalOpen={open}
      modalTitle="New asset"
      onClose={closeModal}
      form={
        <form className="form-grid" onSubmit={onCreate}>
          <div className="form-row">
            <label>Asset name</label>
            <input
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === "Escape" && closeModal()}
            />
          </div>
          <div className="form-row">
            <label>Serial</label>
            <input value={serial} onChange={(e) => setSerial(e.target.value)} />
          </div>
          <div className="form-row">
            <label>Product</label>
            <select value={productId} onChange={(e) => setProductId(e.target.value)}>
              {products.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Company</label>
            <select value={companyId} onChange={(e) => setCompanyId(e.target.value)}>
              {companies.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Linked ticket</label>
            <select value={ticketId} onChange={(e) => setTicketId(e.target.value)}>
              <option value="">None</option>
              {tickets.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.title}
                </option>
              ))}
            </select>
          </div>
          <div className="modal-actions">
            <button className="btn" type="button" onClick={closeModal}>
              Cancel
            </button>
            <button className="btn btn-primary" type="submit" disabled={busy || !name.trim()}>
              Create asset
            </button>
          </div>
        </form>
      }
    >
      {filtered.map((a) => (
        <Link key={a.id} className="list-item" to={recordPath("asset", a.id)}>
          <div>
            <div className="title">{a.name}</div>
            <div className="meta">{a.serial || "No serial"}</div>
          </div>
        </Link>
      ))}
      {filtered.length === 0 && (
        <div className="empty">{query ? "No matching assets" : "No assets yet"}</div>
      )}
    </ListShell>
  );
}

export function RmasPage() {
  const navigate = useNavigate();
  const { setFocus } = useApp();
  const [rmas, setRmas] = useState<Rma[]>([]);
  const [assets, setAssets] = useState<Asset[]>([]);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [rmaAssetId, setRmaAssetId] = useState("");
  const [reason, setReason] = useState("");

  async function reload() {
    const [r, a] = await Promise.all([api.listRmas(), api.listAssets()]);
    setRmas(r);
    setAssets(a);
    if (!rmaAssetId && a[0]) setRmaAssetId(a[0].id);
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "RMAs" });
    void reload();
  }, [setFocus]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return rmas;
    return rmas.filter((r) =>
      `${r.reason ?? ""} ${r.status} ${r.resolution ?? ""}`.toLowerCase().includes(q),
    );
  }, [rmas, query]);

  function closeModal() {
    setOpen(false);
    setReason("");
  }

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    if (busy) return;
    setBusy(true);
    try {
      const asset = assets.find((a) => a.id === rmaAssetId);
      const rma = await api.createRma({
        asset_id: rmaAssetId || null,
        company_id: asset?.company_id ?? null,
        ticket_id: asset?.ticket_id ?? null,
        reason: reason || null,
      });
      closeModal();
      await reload();
      navigate(recordPath("rma", rma.id));
    } finally {
      setBusy(false);
    }
  }

  return (
    <ListShell
      eyebrow="Operations"
      title="RMAs"
      subtitle="Replacement requests linked to assets and tickets."
      backTo="/operations"
      newLabel="Open RMA"
      searchPlaceholder="Search RMAs…"
      query={query}
      onQuery={setQuery}
      onNew={() => setOpen(true)}
      modalOpen={open}
      modalTitle="Open RMA"
      onClose={closeModal}
      form={
        <form className="form-grid" onSubmit={onCreate}>
          <div className="form-row">
            <label>Asset</label>
            <select value={rmaAssetId} onChange={(e) => setRmaAssetId(e.target.value)}>
              {assets.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.name}
                </option>
              ))}
            </select>
          </div>
          <div className="form-row">
            <label>Reason</label>
            <textarea
              autoFocus
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              onKeyDown={(e) => e.key === "Escape" && closeModal()}
            />
          </div>
          <div className="modal-actions">
            <button className="btn" type="button" onClick={closeModal}>
              Cancel
            </button>
            <button className="btn btn-primary" type="submit" disabled={busy}>
              Open RMA
            </button>
          </div>
        </form>
      }
    >
      {filtered.map((r) => (
        <Link key={r.id} className="list-item" to={recordPath("rma", r.id)}>
          <div>
            <div className="title">{r.reason || "RMA"}</div>
            <div className="meta">
              <span className={`badge ${r.status}`}>{formatLabel(r.status)}</span>
            </div>
          </div>
        </Link>
      ))}
      {filtered.length === 0 && (
        <div className="empty">{query ? "No matching RMAs" : "No RMAs yet"}</div>
      )}
    </ListShell>
  );
}
