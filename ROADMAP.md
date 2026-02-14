# ReVillage Society | Regenerate Skagit

## Shared Technology Stack Roadmap

---

## Working Codename (internal)

**Project Heron**

Heron is a place-based name tied to water, patience, and attentiveness rather than scale or control. It signals infrastructure that observes carefully, acts deliberately, and belongs to the landscape it serves.

The name is intentionally internal. It exists to support shared authorship and coordination, not branding or productization.

---

## Purpose of Project Heron

Project Heron is the shared technology stack powering ReVillage Society and Regenerate Skagit, with an explicit eye toward reuse by other landscape-scale initiatives.
This technology stack exists to support **place-based human relationships**, not to function as a generalized platform or SaaS product.

Its role is to:

* Support participation, belonging, and contribution within specific landscapes
* Allow people to move between related initiatives without fragmenting identity
* Reduce duplicated effort across local and watershed-scale projects
* Remain legible, stewardable, and adaptable over time

Non-goals are as important as goals and are listed explicitly below.

---

## Non-Goals (Explicit)

* This is not a startup platform
* This is not optimized for growth metrics or virality
* This is not a one-size-fits-all solution for bioregional organizing
* This does not aim to replace existing tools where they are sufficient
* This is not controlled by a single organization once adopted elsewhere

---

## Architectural Principles of Project Heron

### 1. Singular Identity, Contextual Participation

* A person has one identity
* Participation is scoped by site / landscape
* Roles and permissions are contextual, not global

### 2. Sovereignty by Default

* Each landscape instance owns its data
* Shared infrastructure does not imply shared governance
* Federation is opt-in and reversible

### 3. Boring Technology, Clear Boundaries

* Prefer explicit data models over clever abstractions
* Prefer clarity over performance until proven otherwise
* Favor systems that can be understood by a second maintainer

---

## Current Core Components (Phase 0: Existing)

* Rust backend runtime
* Shared authentication and user model (ReVillage + Regenerate Skagit)
* Membership-based access control
* Site-scoped content and events
* Editorial, long-form reading emphasis

---

## Phase 1: Stabilization & Legibility

**Goal:** Make the system explainable and safe to share with another builder.

Deliverables:

* Clear site / landscape abstraction in code
* Documented data model (users, sites, memberships, roles)
* Permission model that is explicit and testable
* Internal developer documentation (why decisions were made, not just what)

Risks addressed:

* Knowledge trapped in one person
* Implicit defaults bleeding across sites

---

## Phase 2: Skills & Participation Graph

**Goal:** Surface human capacity across projects without centralizing control.

Deliverables:

* Skill profiles scoped to sites but optionally federated
* Ability to express skills as offerings, not credentials
* Simple discovery mechanisms (within-site first)

Design constraints:

* Skills are descriptive, not hierarchical
* Visibility is opt-in
* No gamification or reputation scoring

---

## Phase 3: Landscape Instances (Adoption Without Capture)

**Goal:** Enable other landscapes to adopt Project Heron as local infrastructure without dependency or loss of sovereignty.

Deliverables:

* One shared codebase (Project Heron)
* Separate database per landscape by default
* Clear boundaries between core infrastructure and local customization
* Versioned, slow-moving releases
* Minimal onboarding guide for new landscape stewards

Political constraints:

* Adoption must not imply loss of autonomy
* No central authority over local instances
* Exit must remain possible

**Goal:** Enable other landscapes to adopt the stack without dependency.

Deliverables:

* Multi-instance deployment strategy
* Separate database per landscape by default
* Shared codebase with versioned releases
* Minimal onboarding guide for new landscape stewards

Political constraint:

* Adoption must not imply loss of autonomy

---

## Phase 4: Federation Surfaces

**Goal:** Allow collaboration without consolidation.

Possible federation surfaces:

* Public event feeds
* Skill directories (opt-in)
* Narrative links and shared stories

Explicit exclusions:

* No shared authentication across sovereign instances
* No forced synchronization

---

## Relationship to Regen Cascadia

* Cascadia is treated as an external, federated body
* No expectation of stack alignment
* Value is demonstrated via usable outputs (events, skills, stories)
* Deeper integration only considered if invited

---

## Team Development Intent

This stack is intended to be built by **at least two co-authors**.

Desired collaborator profile:

* Comfortable designing systems with social consequences
* Able to work from intent rather than fixed specs
* Interested in stewardship over optimization

Next step:

* Identify a bounded, real subsystem for shared authorship

---

## Open Questions (Intentionally Unresolved)

* When does a landscape graduate from instance to federation peer?
* How much variation between instances is acceptable?
* What forms of governance need technical expression, if any?

These are expected to evolve through use, not pre-decision.

---

*This roadmap is a living document. Changes should be deliberate and documented, not reactive.*
