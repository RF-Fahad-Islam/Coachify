import { describe, it, expect } from "vitest";
import type { PingPayload, PingResponse, RpcEnvelope } from "./gen/rpc";

describe("RPC contract definitions", () => {
  it("type checks PingPayload and PingResponse envelopes", () => {
    const payload: PingPayload = { message: "test ping" };
    const response: PingResponse = {
      status: "ok",
      database_status: "active",
      database_path: "mock.db",
      roundtrip_timestamp: "2026-09-20T00:00:00Z",
      echo: "echo: test ping",
    };
    const envelope: RpcEnvelope<PingResponse> = {
      ok: true,
      data: response,
    };

    expect(envelope.ok).toBe(true);
    expect(envelope.data?.status).toBe("ok");
    expect(payload.message).toBe("test ping");
  });
});
