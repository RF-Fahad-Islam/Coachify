# 0029. Role Dashboards & Zero Training Mode (Rationale)

## Context

Coaching centers in Bangladesh operate in fast-paced environments with high counter traffic. Staff members come from varied backgrounds:
1. **Front-desk receptionists**: Often young graduates or part-time staff who register new admissions, welcome visitors, and take attendance for batches.
2. **Accounts staff / Cashiers**: Focus almost entirely on rapid cash and mobile wallet (bKash/Nagad) fee collections, receipt printing, and daily drawer balancing.
3. **Subject teachers**: Step into the staff room between lectures for 10 minutes to record attendance or input marks on a physical keyboard.
4. **Owners / Directors**: Run the business, inspect profit margins, audit cash drawers, and review academic performance.

When conventional ERP software displays the same dense, 20-item sidebar and administrative controls to every user, several problems emerge:
- Front-desk staff get lost in deep navigation hierarchies, resulting in long queues during admission seasons.
- Teachers are confused by financial jargon or accidentally click administrative configuration options.
- Software adoption fails because training multiple shifts of staff requires hours of manual instruction ("Zero Training Mode" solves this).

---

## Options Considered

### Option 1: Dedicated Role-Specific Dashboard Views with Action-First Cards and Owner Impersonation (Chosen)

- Render tailored dashboards for `Receptionist`, `Cashier`, `Teacher`, `Admin`, and `Owner`.
- Place primary actions in prominent, large action cards with explicit hotkeys (`N S`, `N P`, `G A`).
- Dynamically prune sidebar navigation links based on user permissions.
- Provide an Owner "View As Role" preview dropdown so administrators can inspect counter staff views.

Pros:
- True "Zero Training": A new receptionist or cashier can perform their job on minute one.
- Eliminates visual clutter and prevents accidental access to sensitive financial or configuration screens.
- Enhances operational speed during peak counter hours.
- Owner preview allows seamless internal auditing and customer support.

Cons:
- Requires maintaining 5 distinct dashboard component layouts in the frontend.

### Option 2: Single Generic Dashboard with Permission-Based Visibility Toggles

Maintain a single unified dashboard, hiding individual metric cards or widgets if the user lacks permissions.

Pros:
- Only one dashboard template to build.

Cons:
- Results in fragmented layouts with blank holes or awkward spacing for limited roles.
- Fails to prioritize role-specific daily actions (e.g. a teacher does not need a receipt reprint button or an admissions graph).
- Does not achieve the "Zero Training" operational standard.

### Option 3: Hardcoded Multiple Desktop App Builds (e.g. Coachify Cashier Edition)

Package separate binary releases for cashiers, receptionists, and teachers.

Pros:
- Absolute isolation of modules.

Cons:
- Massively multiplies build, testing, and deployment overhead.
- In single-PC coaching centers, staff share the same computer across shifts (e.g. morning receptionist, afternoon cashier), making separate apps unviable.
