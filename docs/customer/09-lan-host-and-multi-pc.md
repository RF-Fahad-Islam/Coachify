# LAN Host & Multi-PC Setup Guide (লোকাল নেটওয়ার্ক ও মাল্টি-পিসি হোস্ট নির্দেশিকা)

Coachify allows your coaching center to run across multiple computers (e.g., reception, cashier desk, principal's room) connected to the same local Wi-Fi or Ethernet network without requiring an active internet connection.

This guide explains how to configure your primary computer as the **LAN Host**, configure Windows Firewall, and ensure seamless multi-workstation connectivity.

---

## 1. Requirements (প্রয়োজনীয় শর্তাবলী)

1. **Coachify Local Pro License**:
   - LAN Multi-PC hosting requires an active **Coachify Local Pro** license or the 14-day evaluation trial.
   - Secondary workstations do not need separate licenses; they connect to your licensed host PC.
2. **Local Office Network**:
   - All computers must be connected to the same local Wi-Fi router or office switch/LAN cable.
   - An active internet connection is **not** required; Coachify runs 100% offline.
3. **Primary Host PC**:
   - One designated desktop computer or laptop acts as the authoritative database host.
   - This PC must remain powered on during counter working hours.

---

## 2. Starting the LAN Host Service (হোস্ট সার্ভিস চালু করার নিয়ম)

1. On your primary host PC, open Coachify.
2. Navigate to **Settings → LAN Host** (সেটিংস → লোকাল নেটওয়ার্ক হোস্ট).
3. Review your **Host Listener Settings**:
   - **Active Port**: Default is `50443`.
   - **Bind Address**: `0.0.0.0` (listens on all network adapters).
   - **Inhibit Windows Sleep**: Enabled by default (prevents Windows from entering sleep mode while workstations are active).
4. Click **Start Host** (হোস্ট শুরু করুন).
5. The status badge will change to **Running** (green) and your PC's local IP address (e.g., `192.168.1.55`) will be displayed.

---

## 3. Windows Firewall & Network Category (উইন্ডোজ ফায়ারওয়াল সেটআপ)

Windows Firewall protects your PC by blocking incoming network traffic on networks marked as "Public". For secondary workstations to connect to the host:

### A. Ensure Network Category is "Private"
1. In Coachify **Settings → LAN Host**, check the **Windows Firewall & Network Profile** panel.
2. If your network profile shows **Profile: Public (Blocked by Windows)**:
   - Click **Copy Command** on the PowerShell remediation script.
   - Open **Windows PowerShell as Administrator** and paste the command:
     ```powershell
     Get-NetConnectionProfile | Where-Object { $_.NetworkCategory -eq 'Public' } | Set-NetConnectionProfile -NetworkCategory Private
     ```
   - Press Enter. Coachify will detect the update and switch to **Profile: Private (Recommended)**.

### B. Allow Port 50443 Through Firewall
1. If the firewall indicator shows **Firewall: Rule Missing**:
   - Click **Copy Command** for the Windows Firewall rule.
   - Open **Command Prompt as Administrator** and paste the command:
     ```cmd
     netsh advfirewall firewall add rule name="Coachify LAN Host" dir=in action=allow protocol=TCP localport=50443 profile=private
     ```
   - Press Enter. This creates a secure inbound rule allowing Coachify workstations on your private network to communicate with the host.

---

## 4. Workstation Discovery & Manual IP Ping (কানেকশন ও ওয়ার্কস্টেশন পিং)

### Automatic mDNS Discovery
Coachify announces itself across your local network using zero-configuration mDNS (DNS-SD `_coachify._tcp.local`). When a secondary workstation opens Coachify in client mode:
- It automatically detects your host computer's name (e.g., `Coachify - Apex Coaching (FRONT-DESK-PC)`).
- Click **Connect** to start the pairing handshake.

### Manual IP Fallback (যদি অটোমেটিক ডিটেক্ট না হয়)
Some office Wi-Fi routers disable multicast packets. If a workstation cannot discover the host automatically:
1. On the host PC, note down one of the local IP addresses shown in **Settings → LAN Host** (e.g., `192.168.1.55`).
2. On the workstation, enter the host's IP address (`192.168.1.55`) and port (`50443`).
3. Click **Test Connection (Ping)** to verify direct connectivity and certificate handshake.

---

## 5. Security & TLS Fingerprint Pinning (নিরাপত্তা ও এনক্রিপশন)

Coachify uses bank-grade **TLS 1.3 encryption** for all local network traffic:
- All student data, receipts, and cash amounts transmitted over your office Wi-Fi are fully encrypted.
- Students or visitors on your coaching center Wi-Fi **cannot** read or tamper with counter transactions.
- The host displays a unique **TLS 1.3 Certificate Fingerprint (SHA-256)**:
  `SHA256:XX:XX:...`
- Secondary workstations verify this cryptographic fingerprint on initial pairing to guarantee they are connecting to your authentic host computer.

---

## 6. Keeping the Host Running in the Background (ব্যাকগ্রাউন্ডে হোস্ট চালু রাখা)

- **Minimizing to System Tray**: When LAN Host is running, clicking the window close (`X`) button hides the Coachify window to the Windows System Tray rather than quitting, so cashiers on other PCs can continue working without disruption.
- **Sleep Prevention**: While the host is active, Coachify automatically instructs Windows to stay awake, preventing automatic sleep or network card power-downs.
- **Stopping the Host**: To turn off host mode, open **Settings → LAN Host** and click **Stop Host**.
