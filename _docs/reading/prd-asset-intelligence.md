# PRD: The Real-Time Asset Intelligence Platform

## 1. Document Overview
This document provides the technical and product requirements for transforming the `cloudlist` tool into a real-time Asset Intelligence Platform. The primary goal is to address the deep, unmet needs of our core user persona by moving from periodic, disconnected data dumps to a live, contextual model of the entire digital environment.

**Guiding Principle:** This PRD is built upon the following Point-of-View (PoV) statement, derived from empathetic user research:

> A **Senior Cloud Security Engineer** needs to **instantly connect any cloud asset to its owner, business purpose, and potential blast radius** because **she is currently drowning in disconnected, stale inventory data, forcing her to be a reactive auditor of past events instead of a proactive defender of business-critical services.**

---

## 2. Objective
To build a 10x Asset Intelligence Platform that empowers security and operations teams to move from a reactive to a proactive stance. The platform will achieve this by providing a real-time, queryable, and context-rich graph of all assets, their dependencies, and their relationship to the business.

---

## 3. Scope

### In Scope:
*   Real-time ingestion of asset data from major cloud providers (initially AWS, GCP, Azure).
*   A graph-based data model representing assets and their relationships.
*   An API and user interface for querying asset relationships, ownership, and business context.
*   Functionality to perform "blast radius" analysis (i.e., trace dependencies).
*   Integration with identity providers to correlate assets with owners/teams.

### Out of Scope:
*   Automated remediation of misconfigurations (v1).
*   Full-fledged security policy enforcement engine (v1).
*   Agent-based asset discovery (v1 will be API- and event-driven).

---

## 4. User Personas and Use Cases

### Persona: Alex, Senior Cloud Security Engineer
*   **Needs:** Immediacy, context, and proactivity.
*   **Goals:** Quickly assess risk, understand the business impact of vulnerabilities, prevent misconfigurations before they happen, and reduce the mean time to response (MTTR).
*   **Frustrations:** Stale data, disconnected tools, and the inability to answer "what if" questions.

### Use Case 1: Investigating a Zero-Day Vulnerability
1.  A new critical vulnerability (e.g., "Log4j") is announced.
2.  Alex navigates to the platform's search interface.
3.  She types a natural language query: `show me all public-facing servers running java with log4j < 2.17.1`
4.  The platform returns a list of vulnerable assets within seconds.
5.  Alex clicks on a critical server. The interface instantly shows her:
    *   **Owner:** "Team: payments-api"
    *   **Business Service:** "Payments Processing"
    *   **Dependencies:** The databases it connects to, the load balancers it sits behind.
    *   **Blast Radius:** A visual graph showing that if this server is compromised, it could potentially impact the "PCI Data Environment" and the "Customer Billing" service.
6.  With this complete context, Alex can prioritize remediation with the correct team immediately.

---

## 5. Functional Requirements

| ID | Requirement | Description |
|:---|:---|:---|
| FR-01 | **Real-Time Event Ingestion** | The system must be able to ingest and process asset change events from cloud providers (e.g., AWS CloudTrail, GCP Audit Logs) with a p99 latency of < 5 seconds. |
| FR-02 | **Graph-Based Relationship Querying** | The system must provide an API endpoint that allows querying of relationships between assets (e.g., `(EC2_Instance)-[:CONNECTS_TO]->(RDS_Database)`). |
| FR-03 | **"Blast Radius" Analysis** | Given an asset ID, the system must be able to traverse the asset graph to identify all upstream and downstream dependencies and return them in a single API call. |
| FR-04 | **Ownership & Context API** | The system must expose an API to enrich assets with metadata, including `owner` and `business_service`. This data can be ingested from external sources like a CMDB or identity provider. |
| FR-05 | **Natural Language Query Interface** | The primary user interface must feature a search bar that can parse natural language queries into formal graph queries, enabling users like Alex to ask questions in plain English. (Inferred: This requires a service that translates NLP to Cypher/GSQL). |

---

## 6. Non-Functional Requirements

| Attribute | Requirement | Justification / Inference |
|:---|:---|:---|
| **Data Freshness** | Asset data presented to the user must be no more than 10 seconds stale. | Addresses Alex's need for **immediacy** over periodic reports. |
| **Performance** | API query latency for typical asset lookups and relationship queries must be < 50ms (p99). Ingestion throughput must support 10,000 events/sec. | Fulfills the 10x performance goal and ensures the platform feels instantaneous. |
| **Scalability** | The platform must be horizontally scalable to support a 100x increase in assets and events without a linear increase in operational overhead. | Based on the need to support large, growing enterprise environments. |
| **Usability** | The primary query interface should be as intuitive as a web search engine, abstracting away the complexity of the underlying graph database. | Addresses Alex's need for **context over raw data** by making complex queries easy. |

---

## 7. Follow-up Questions & Answers

**Q1: How is this fundamentally different from just running `cloudlist` every 5 minutes?**
*   **A:** Running `cloudlist` in a tight loop is a "brute-force" approach that is inefficient and doesn't scale. It repeatedly queries all assets, missing the discrete changes between runs. An event-driven model is far more efficient, as it only processes actual changes. More importantly, the new architecture builds a persistent, queryable *graph* of relationships, something a batch tool cannot do.

**Q2: Will users have to learn a complex graph query language like Cypher?**
*   **A:** No. The core product vision is that the primary interface will be a Natural Language Query (NLQ) search bar. While an advanced API for power users will expose the underlying graph query language, the primary user experience is designed for security professionals like Alex, not database experts.

**Q3: Where does the "business context" data come from?**
*   **A:** V1 will focus on two primary sources: 1) Cloud provider tags (e.g., `owner`, `service`), which are often the ground truth for infrastructure teams, and 2) Pluggable integrations with external identity providers or CMDBs via a dedicated "Enrichment API." The platform's value increases exponentially as it is enriched with more context.
