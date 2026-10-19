import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import type { Asset, Product, Rma } from "@confluity/shared";
import { api, formatLabel } from "../lib/api";
import { AttachmentsPanel } from "../components/AttachmentsPanel";
import { CommentsPanel } from "../components/CommentsPanel";
import { LinkedRecordsPanel } from "../components/LinkedRecords";
import { RecordActions } from "../components/RecordActions";
import { useApp } from "../context/AppContext";

export function ProductDetailPage() {
  const { id = "" } = useParams();
  const { setFocus } = useApp();
  const [product, setProduct] = useState<Product | null>(null);

  useEffect(() => {
    void (async () => {
      const p = await api.getProduct(id);
      setProduct(p);
      if (p) setFocus({ entityType: "product", entityId: p.id, label: p.name });
    })();
  }, [id, setFocus]);

  if (!product) return <div className="empty">Product not found</div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">Product</div>
          <h1>{product.name}</h1>
          <p>
            {product.sku || "No SKU"} · qty {product.inventory_qty}
          </p>
        </div>
        <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <RecordActions entityType="product" entityId={product.id} label={product.name} />
          <Link className="btn" to="/operations/products">
            Back
          </Link>
        </div>
      </div>
      <div className="grid-2">
        <div className="panel detail-block">
          <h2>Description</h2>
          <p>{product.description || "No description"}</p>
          <form
            className="form-grid"
            onSubmit={async (e) => {
              e.preventDefault();
              const qty = Number((e.currentTarget.elements.namedItem("qty") as HTMLInputElement).value);
              const updated = await api.updateProduct({
                id: product.id,
                name: product.name,
                sku: product.sku,
                description: product.description,
                inventory_qty: Number.isFinite(qty) ? qty : product.inventory_qty,
              });
              setProduct(updated);
            }}
          >
            <div className="form-row">
              <label>Inventory</label>
              <input name="qty" defaultValue={String(product.inventory_qty)} />
            </div>
            <button className="btn" type="submit">
              Save inventory
            </button>
          </form>
        </div>
        <LinkedRecordsPanel entityType="product" entityId={product.id} />
      </div>
    </div>
  );
}

export function AssetDetailPage() {
  const { id = "" } = useParams();
  const { setFocus, can } = useApp();
  const [asset, setAsset] = useState<Asset | null>(null);

  useEffect(() => {
    void (async () => {
      const a = await api.getAsset(id);
      setAsset(a);
      if (a) setFocus({ entityType: "asset", entityId: a.id, label: a.name });
    })();
  }, [id, setFocus]);

  if (!asset) return <div className="empty">Asset not found</div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">Asset</div>
          <h1>{asset.name}</h1>
          <p>
            {can("operations.field.serial_numbers")
              ? asset.serial || "No serial"
              : "Serial hidden"}
            {asset.warranty_expires_at ? ` · warranty ${asset.warranty_expires_at}` : ""}
          </p>
        </div>
        <Link className="btn" to="/operations/assets">
          Back
        </Link>
      </div>
      <div className="grid-2">
        <div>
          <div className="panel detail-block">
            <h2>Notes</h2>
            <p>{asset.notes || "No notes"}</p>
          </div>
          <AttachmentsPanel entityType="asset" entityId={asset.id} />
          <CommentsPanel entityType="asset" entityId={asset.id} />
        </div>
        <LinkedRecordsPanel entityType="asset" entityId={asset.id} />
      </div>
    </div>
  );
}

export function RmaDetailPage() {
  const { id = "" } = useParams();
  const { setFocus } = useApp();
  const [rma, setRma] = useState<Rma | null>(null);
  const [status, setStatus] = useState("open");

  useEffect(() => {
    void (async () => {
      const r = await api.getRma(id);
      setRma(r);
      if (r) {
        setStatus(r.status);
        setFocus({ entityType: "rma", entityId: r.id, label: r.reason || "RMA" });
      }
    })();
  }, [id, setFocus]);

  async function save() {
    if (!rma) return;
    const updated = await api.updateRma({
      id: rma.id,
      status,
      reason: rma.reason,
      resolution: rma.resolution,
    });
    setRma(updated);
  }

  if (!rma) return <div className="empty">RMA not found</div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">RMA</div>
          <h1>{rma.reason || "Replacement"}</h1>
          <p>
            <span className={`badge ${rma.status}`}>{formatLabel(rma.status)}</span>
          </p>
        </div>
        <div className="ticket-status-save">
          <label>
            Status
            <select value={status} onChange={(e) => setStatus(e.target.value)}>
              <option value="open">open</option>
              <option value="approved">approved</option>
              <option value="shipped">shipped</option>
              <option value="closed">closed</option>
            </select>
          </label>
          <button className="btn btn-primary" disabled={status === rma.status} onClick={() => void save()}>
            Save
          </button>
          <Link className="btn" to="/operations/rmas">
            Back
          </Link>
        </div>
      </div>
      <div className="grid-2">
        <CommentsPanel entityType="rma" entityId={rma.id} />
        <LinkedRecordsPanel entityType="rma" entityId={rma.id} />
      </div>
    </div>
  );
}
