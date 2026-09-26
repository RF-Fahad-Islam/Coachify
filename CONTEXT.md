# Coachify Domain Context

Offline-first management platform for coaching centers and educational institutions in Bangladesh, operating locally with LAN workstation synchronization.

## Architecture & Execution

**Command Dispatcher**:
The centralized operational engine executing typed commands against domain modules and emitting change events.
_Avoid_: Request router, API server, command bus

**Host Mode**:
The primary execution mode of Coachify running locally with direct SQLite database ownership and hosting LAN services.
_Avoid_: Server mode, master instance

**Workstation Client Mode**:
A paired secondary instance of Coachify operating on the local network that forwards domain commands over HTTPS to the primary Host while managing local peripherals.
_Avoid_: Slave instance, remote terminal, web client

**Command Seam**:
The typed boundary where external callers invoke domain commands through serialized envelopes.
_Avoid_: API endpoint, RPC gateway, service interface

**Connection Policy**:
The declarative specification defining whether a command requires an active SQLite connection, operates on pre-database runtime state, or executes without database access.
_Avoid_: DB flag, connection requirement, access mode

**Command Locality**:
The declarative classification determining whether a command executes locally on a paired workstation or forwards over the network to the Host.
_Avoid_: Routing rule, proxy target, execution location

**Host Controller**:
The domain module interface managing Host Mode lifecycle, mDNS service advertisement, device authorization, and real-time event broadcasting.
_Avoid_: Host service, server manager, backend host

**Transport Client**:
The domain module interface managing Workstation Client Mode network transport, pinned TLS verification, command forwarding, and real-time cache event consumption.
_Avoid_: HTTP client, workstation proxy, network bridge

**Lan Runtime**:
The unified operational container encapsulating the Host Controller and Transport Client for the application lifecycle.
_Avoid_: LAN service, network coordinator

**Entitlement Seam**:
The centralized policy engine evaluating operational capabilities, plan tiers, grace periods, and device quotas against active licensing state.
_Avoid_: Feature flags, license checker, permission guard
