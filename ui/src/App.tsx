import { useCallback, useEffect, useState } from "react";
import { rpc, PingResponse, RpcError } from "./gen/rpc";

export function App() {
  const [loading, setLoading] = useState(false);
  const [response, setResponse] = useState<PingResponse | null>(null);
  const [error, setError] = useState<RpcError | null>(null);
  const [pingCount, setPingCount] = useState(0);

  const runPing = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setPingCount((prev) => prev + 1);
      const res = await rpc("system.ping", {
        message: "Ping from UI",
      });

      if (res.ok && res.data) {
        setResponse(res.data);
      } else if (res.error) {
        setError(res.error);
      }
    } catch (e: unknown) {
      setError({
        code: "client_exception",
        message: e instanceof Error ? e.message : String(e),
      });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void runPing();
  }, [runPing]);

  return (
    <main className="container">
      <header className="header">
        <h1 className="title">Coachify Desktop</h1>
        <p className="subtitle">
          Architecture Foundation · One Process, Three Layers (React UI → Tauri Shell →
          Rust Core)
        </p>
        <div>
          <span
            className={`badge ${
              response?.status === "ok" ? "badge-success" : "badge-pending"
            }`}
          >
            {response?.status === "ok" ? "Core Connected" : "Connecting..."}
          </span>
        </div>
      </header>

      <section className="card">
        <h2 className="card-title">RPC Boundary Verification</h2>
        <div className="card-content">
          <p>
            <strong>Contract:</strong> Single <code>rpc(command, payload)</code> bridge
          </p>
          <p>
            <strong>Command:</strong> <code>system.ping</code>
          </p>
          <p>
            <strong>Roundtrip Status:</strong>{" "}
            {response ? response.status : "Waiting for response..."}
          </p>
          <p>
            <strong>Database Status:</strong>{" "}
            {response ? response.database_status : "Checking..."}
          </p>
          <p>
            <strong>Timestamp:</strong> {response?.roundtrip_timestamp || "None"}
          </p>
          <p>
            <strong>Ping Count:</strong> {pingCount}
          </p>
          {response?.echo && (
            <p>
              <strong>Echo Received:</strong> {response.echo}
            </p>
          )}
        </div>
      </section>

      {error && (
        <section
          className="card"
          style={{ borderColor: "#fca5a5", backgroundColor: "#fef2f2" }}
        >
          <h2 className="card-title" style={{ color: "#991b1b" }}>
            Error Received
          </h2>
          <div className="card-content" style={{ color: "#991b1b" }}>
            <p>
              <strong>Code:</strong> {error.code}
            </p>
            <p>
              <strong>Message:</strong> {error.message}
            </p>
          </div>
        </section>
      )}

      <div className="actions">
        <button className="btn" onClick={runPing} disabled={loading}>
          {loading ? "Testing Roundtrip..." : "Send RPC Ping"}
        </button>
      </div>
    </main>
  );
}
