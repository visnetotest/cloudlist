
# TRD-002: Language Choice - Rust

## 1. Status

**Decided**

## 2. Context

A foundational decision for `cloudscanner` is the choice of programming language. The previous generation was written in Go. While Go is a capable language, the next generation of the platform has requirements for higher performance, stricter safety guarantees, and more predictable resource utilization, especially for a highly concurrent scanning engine.

## 3. Decision

`cloudscanner` will be built from the ground up in Rust.

## 4. Consequences

### 4.1. Advantages

*   **Memory Safety & Fearless Concurrency:** Rust's ownership model and borrow checker eliminate entire classes of concurrency-related bugs and memory safety issues (e.g., null pointer dereferences, data races) at compile time. This is critical for building a fast and safe concurrent scanning engine.
*   **Performance:** Rust offers C-level performance with high-level abstractions, which is essential for processing large volumes of data from cloud APIs efficiently.
*   **No Garbage Collector:** The absence of a garbage collector (GC) means more predictable latency and resource consumption, which is beneficial for a performance-critical application.
*   **Reliable Single-Binary Deployment:** Rust compiles to a single, statically-linked native binary with no external runtime dependencies. This simplifies deployment across all major platforms and in containerized environments.
*   **Rich Ecosystem:** The Rust ecosystem (Cargo and crates.io) provides high-quality libraries for asynchronous I/O (`tokio`), observability (`tracing`), and more.

### 4.2. Disadvantages

*   **Steeper Learning Curve:** Rust has a steeper learning curve compared to Go, which may impact developer onboarding and initial development velocity.
*   **Compilation Time:** Rust compilation times can be longer than Go's.
*   **Verbosity:** In some cases, Rust code can be more verbose than Go, especially around error handling and lifetimes.

### 4.3. Go vs. Rust: A Comparative Analysis

While the previous version of `cloudlist` was successful in Go, the decision to migrate to Rust is based on key technical trade-offs where Rust offers a distinct advantage for this specific use case.

| Aspect                | Go                                                                           | Rust                                                                                  | Advantage for `cloudscanner`                                                           |
| :-------------------- | :--------------------------------------------------------------------------- | :------------------------------------------------------------------------------------ | :------------------------------------------------------------------------------------- |
| **Speed & Performance** | Fast, but with GC pauses that can introduce latency spikes.                   | C-level performance with no GC. Sustained, predictable speed.                         | **Rust**. Predictable performance is critical for a scanning engine.                      |
| **Memory Usage**      | Generally higher due to the GC runtime.                                      | Minimal memory footprint, with manual-like control over memory allocation and lifetimes. | **Rust**. Lower memory usage is crucial when processing millions of assets.               |
| **Concurrency Model** | Goroutines and channels are easy to use but the risk of data races exists.     | Ownership model and borrow checker eliminate data races at compile time.              | **Rust**. Guarantees thread safety, which is essential for a highly concurrent scanner.     |
| **Binary File Size**  | Produces a single, statically-linked binary, but includes the Go runtime.      | Produces a single, statically-linked binary with no runtime overhead.                   | **Rust**. Binaries are typically smaller, leading to more efficient distribution.        |

**Summary of Comparison:**

*   **Go** excels in developer velocity and simplicity, making it excellent for general-purpose network services.
*   **Rust** excels in performance-critical applications where safety, resource control, and predictable speed are paramount.

For a security tool like `cloudscanner`, which must be both fast and absolutely reliable while handling large amounts of data, the guarantees provided by Rust justify the steeper learning curve.

## 5. Reference

This decision is referenced in the main technical specification: [CLOUDSCANNER_TECHNICAL_SPECIFICATION.md#1.1.-Language-Justification:-Rust](./CLOUDSCANNER_TECHNICAL_SPECIFICATION.md#1.1.-Language-Justification:-Rust)
