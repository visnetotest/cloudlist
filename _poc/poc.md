## PoC 1: Native Policy Engine (MVP)

### 1. Vision & Goals

The primary goal of this PoC is to validate the feasibility and value of a native, in-process policy evaluation engine for `cloudlist`. This MVP will focus on creating a simple, fast, and easy-to-use policy system that can serve as a foundation for more advanced security and compliance features.

The goals are to:

*   **Build the Core Evaluator**: Implement a native policy evaluator in Go that can process `cloudlist`'s JSON output against a set of user-defined rules.
*   **Validate Data Models**: Confirm that the proposed `Policy` and `Violation` data models are effective and ergonomic for users.
*   **Establish a Foundation**: Build a solid, extensible foundation for future, more advanced policy features (e.g., OPA integration, auto-remediation).

### 2. Success Criteria

*   **Technical Validation**: The engine must successfully evaluate a sample set of policies against a corpus of mock assets, producing a 100% accurate report.
*   **Performance Validation**: The policy evaluation process must add no more than a 20% time overhead to a standard `cloudlist` scan for the targeted resource types.
*   **Architectural Validation**: The final code must provide a clean, pluggable interface for the evaluator, allowing a future `OPAEvaluator` to be added with minimal refactoring of the core engine.

### 3. Key Risks & Mitigation Strategies

*   **Risk: Policy Logic Complexity**: The native evaluator's simple key-value matching may be insufficient for real-world security policies, leading to a dead-end design.
    *   **Mitigation**: The PoC scope is intentionally limited to simple matching. The architectural success criterion explicitly requires a **pluggable interface**, ensuring that we can easily replace the native evaluator with a more powerful engine like OPA in the future without a major rewrite.

*   **Risk: Performance Overhead**: The evaluation process could significantly slow down `cloudlist` scans, making it impractical for large environments.
    *   **Mitigation**: A hard performance budget (<= 20% overhead) is a primary success criterion. We will implement benchmarking from the start and focus on efficient in-memory processing to stay within this budget.

### 4. Status & Next Steps

*   **Status**: **Completed & Integrated**
*   **Summary**: The core MVP is functional and has been fully integrated into the `cloudlist` application. The code, now located in `pkg/policy`, can be triggered via command-line flags (`--policy-file`, `--policy-block`). It loads policies from a YAML file, evaluates them against discovered cloud assets, and reports any violations. All goals of PoC 1 have been met.
*   **Next Steps**: With the foundational policy engine in place, the project will now move to **PoC 2: The Asset Intelligence Platform Core**. This next phase will focus on re-architecting `cloudlist` into an event-driven platform to enable real-time asset correlation and enrichment, as outlined in the goals for PoC 2.

---

## PoC 2: The Asset Intelligence Platform Core

### 1. Vision & Goals

This Proof of Concept marks a fundamental architectural shift, moving from a synchronous, batch-oriented tool to an **event-driven, stream-processing platform**. This is the foundational step required to deliver the real-time, contextual intelligence envisioned in the Asset Intelligence PRD, enabling use cases like rapid vulnerability investigation and blast radius analysis.

The goals are to:

*   **Build the Core Pipeline**: Implement a basic event-driven pipeline using an event bus (NATS) to decouple asset discovery from downstream processing.
*   **Validate the Correlation Engine**: Create a "Correlation Service" that subscribes to raw discovery events and intelligently merges them into a single, canonical asset identity.
*   **Prove the Paradigm**: Demonstrate that this event-driven model is a viable and scalable foundation for building future real-time services (e.g., enrichment, graph modeling, policy evaluation).

### 2. Success Criteria

*   **End-to-End Event Flow**: Successfully publish raw asset data from at least two different `cloudlist` providers (e.g., AWS, GCP) as distinct events onto the event bus.
*   **Successful Correlation**: The Correlation Service must correctly consume raw events and merge data for the same logical asset (e.g., an EC2 instance and a Route53 record pointing to it) into a single, unified `CanonicalAsset` record.
*   **Architectural Validation**: The final code must demonstrate clear separation of concerns between producers (cloudlist adapters) and consumers (Correlation Service), proving the decoupled nature of the architecture.

### 3. Key Risks & Mitigation Strategies

*   **Risk: Architectural Complexity**: Introducing an event bus and microservices adds significant operational complexity (deployment, monitoring, local development) compared to the current single-binary model.
    *   **Mitigation**: We will use **Docker Compose** to create a fully self-contained local development environment. This allows any developer to spin up the entire stack (NATS, Correlation Service, etc.) with a single command, mitigating setup friction. Production complexity is acknowledged but deferred to a later phase.

*   **Risk: Asset Correlation Logic**: Defining a "canonical asset" is notoriously difficult. A simple IP-based correlation may fail to merge related assets (e.g., a load balancer and its instances) or incorrectly merge unrelated ones (e.g., ephemeral IPs).
    *   **Mitigation**: The PoC will focus on a **narrow, well-defined correlation strategy** (e.g., linking a known Public IP to a DNS name). We will explicitly *not* try to solve all correlation edge cases. The goal is to prove the *pipeline* is viable, not to perfect the correlation algorithm at this stage.

### 4. Status & Next Steps

*   **Status**: **In Progress**
*   **Summary**: Foundational work for the event-driven architecture has begun. A temporary file-based event queue has been implemented in `pkg/pipeline`, which validates the basic event model. However, this is a placeholder and the core goal of PoC 2—a real-time, stream-processing platform—has not yet been achieved. The next steps will focus on replacing the file-based queue with a robust event bus (NATS) and creating the first independent microservice for asset correlation.
*   **Next Steps**:
    1.  **Integrate NATS**: Replace the file-based queue in `pkg/pipeline` with a NATS client to create a true event bus.
    2.  **Create `services/` Directory**: Create a new top-level `services/` directory to house the new, independent microservices.
    3.  **Create Correlation Service**: Create the initial directory and boilerplate for the `correlation-service` within the `services/` directory.
    4.  **Develop Event Producer**: Implement the logic in the main `cloudlist` application to publish asset discovery events to the NATS event bus.
    5.  **Develop Event Consumer**: Implement the initial logic in the `correlation-service` to subscribe to the NATS event bus and receive asset discovery events.
