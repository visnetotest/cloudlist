# Marketing Requirements Document (MRD): cloudscanner PoC

## 1. Document Overview

This document outlines the market requirements for the next-generation cloud security platform, codenamed `cloudscanner`. It defines the market opportunity, target audience, and key capabilities required to deliver a "10x" improvement over existing Cloud Security Posture Management (CSPM) solutions. This MRD will drive the product and engineering efforts for the Proof of Concept (PoC2).

## 2. Market Problem & Opportunity

**The Problem:** Current-generation CSPM tools are failing security and development teams. They are:
*   **Too Slow:** Relying on periodic, full-account scans that can take hours, leaving a wide-open window for attackers to exploit newly created misconfigurations.
*   **Too Noisy:** They produce flat lists of thousands of "low-severity" alerts without providing the context needed to understand the *actual* risk. This alert fatigue overwhelms security teams.
*   **Too Reactive:** They are fundamentally reactive, reporting on problems long after they have been created. They lack the intelligence to identify complex attack paths or the capability to prevent issues from happening in the first place.

**The Opportunity:** There is a significant and underserved market for a cloud security platform built for the speed and complexity of modern cloud environments. The market needs a solution that is:
*   **Proactive & Real-Time:** To detect and remediate threats in seconds, not hours.
*   **Context-Aware:** To move beyond simple lists and understand the relationships between assets to identify true business risk.
*   **Developer-Centric:** To integrate seamlessly into developer workflows and empower them to build securely from the start.

`cloudscanner` is designed to seize this opportunity by being a faster, smarter, and more automated security engine.

## 3. Target Audience & Personas

We are targeting three primary user personas who are frustrated with the limitations of existing tools.

*   **"Sec" - The Security Engineer:**
    *   **Needs:** To quickly identify, prioritize, and remediate critical risks across multiple cloud environments. Needs to understand the "blast radius" of a compromised resource to focus their efforts.
    *   **Frustration:** Overwhelmed by thousands of meaningless alerts from their current CSPM. Spends too much time manually correlating findings to determine what is actually important.

*   **"Dev" - The DevOps Engineer:**
    *   **Needs:** A fast, reliable security tool that integrates directly into their CI/CD pipeline. Needs clear, actionable feedback to fix security issues *before* they reach production.
    *   **Frustration:** Existing security tools are slow, clunky, and provide cryptic feedback that slows down development cycles.

*   **"Compliance" - The Compliance Manager:**
    *   **Needs:** To easily generate evidence that the organization is adhering to compliance frameworks (e.g., SOC2, PCI-DSS, CIS).
    *   **Frustration:** Existing tools require complex setup and custom querying to generate the evidence reports they need for auditors.

## 4. Key Capabilities & Value Propositions

`cloudscanner` will deliver a 10x improvement through three core pillars, which form the basis of the PoC.

| Pillar | Value Proposition | Required Capabilities |
| :--- | :--- | :--- |
| **1. Real-Time Discovery** | **Find and fix critical misconfigurations in seconds, not hours.** | The product must listen to cloud event streams (e.g., AWS CloudTrail) and trigger immediate, targeted scans of resources as they are created or modified. |
| **2. Asset Intelligence** | **See your cloud like an attacker does. Prioritize what truly matters.** | The product must build an in-memory graph of cloud assets and their relationships to enable advanced analysis like attack path discovery and blast radius calculation. |
| **3. Automated Governance** | **Create "self-healing" infrastructure that automatically fixes issues.** | The product must support policy-driven, automated remediation of violations, with safety mechanisms like `dry-run` mode to ensure user control and trust. |

## 5. Functional Requirements

To meet the needs of our target market, the product must provide the following functionality:

*   **High-Performance Discovery:** Must be able to perform rapid discovery of assets across multiple cloud providers via a flexible plugin architecture.
*   **Human-Readable Policies:** Users must be able to define security and compliance policies in a simple, human-readable format (YAML), without needing to learn a complex query language.
*   **Unified CLI Tool:** The entire workflow (discover, evaluate, remediate) must be managed through a single, high-performance binary that is easy to deploy and run anywhere.
*   **Secure Credential Handling:** The tool must integrate with standard, secure credential mechanisms (environment variables, IAM roles) and must not require secrets to be stored in configuration files.

## 6. Non-Functional Requirements

These quality attributes are critical for market adoption and success.

*   **Performance:** Event-driven scans must complete in seconds. Full scans must be significantly faster than competing solutions.
*   **Usability:** The CLI must be intuitive, with clear progress indicators and flexible verbosity controls. Configuration must be straightforward and well-documented.
*   **Reliability:** The engine must be resilient, gracefully handling failures in one provider without terminating the entire scan.
*   **Extensibility:** The plugin architecture must be well-defined to allow for rapid development of support for new cloud providers and services.

## 7. Success Metrics for PoC

The success of the PoC will be measured against the following key performance indicators:

*   **Time to Value (TTV):** A new user can install the tool, configure a provider, and find a critical misconfiguration (e.g., a public S3 bucket) in **under 15 minutes**.
*   **Mean Time to Detect (MTTD):** A new, critical misconfiguration is detected via the event-driven discovery engine in **under 60 seconds** from the time of creation.
*   **Attack Path Identification:** The asset graph can successfully identify a complete attack path from a public-facing resource to a sensitive data store in a test environment.
