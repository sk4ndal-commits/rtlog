# Log Monitoring TUI - Guidelines

## **Vision**

The purpose of this tool is to provide a **portable, real-time, and interactive log monitoring solution** for customer machines. It is designed to:

* Display logs from files or streams in real-time.
* Enable dynamic filtering and highlighting for quick issue identification.
* Be fully **file-agnostic**, supporting multiple log sources and formats.
* Offer a **Text User Interface (TUI)** with panels for main log display, filter input, summary stats, context inspection, and alerts.
* Be a **single-binary deployable tool**, requiring minimal installation on customer machines.

---

## **Core Principles**

1. **Performance:** Handle high-frequency log streams efficiently without blocking the UI.
2. **Portability:** Produce statically linked binaries for Linux, Windows, and optionally macOS.
3. **Usability:** Provide a clear, intuitive interface with auto-scrolling, filter panels, and status indicators.
4. **Safety:** Use Rust’s memory safety guarantees to prevent crashes, memory leaks, and undefined behavior.

---

## **Basic Coding Guidelines (Rust)**

### **Project Structure**

* `src/main.rs` → entry point, TUI initialization, async runtime start.
* `src/log.rs` → log input, streaming, and parsing logic.
* `src/ui.rs` → TUI layout, panels, drawing routines.
* `src/filter.rs` → pattern matching, regex, context handling.
* `src/state.rs` → shared app state (logs, filters, stats).

### **Rust Coding Practices**

* Use **async/await** and **Tokio** for non-blocking log streaming.
* Avoid **blocking calls** in the main UI thread.
* Use **channels (`tokio::sync::mpsc`)** for passing log lines to the UI thread.
* Use **structs/enums** to clearly model log entries, filters, and events.
* Prefer **immutable data** where possible; use `Arc<Mutex<...>>` only when shared mutable state is required.
* Keep TUI updates **decoupled from log processing**.
* Write **unit tests** for parsing, filtering, and context extraction logic.

### **UI Guidelines**

* Main log window is the primary focus; other panels are secondary.
* Colorize only matched patterns or critical entries.
* Support **keyboard navigation**: scroll, toggle filters, switch sources.
* Ensure TUI updates remain smooth even under high log volume.
* Avoid rendering unnecessary widgets to maintain performance.

---

## **What This Tool Shall Not Do**

1. **Not alter log files** – read-only operation only.
2. **Not execute external commands or scripts** – purely a monitoring tool.
3. **Not assume specific log formats** – must remain file-agnostic.
4. **Not store sensitive customer data** – logs are ephemeral unless explicitly saved by the user.
5. **Not require installation of additional runtimes** – must run as a single binary.
6. **Not replace full-featured log analysis platforms** – this is a lightweight, real-time viewer, not a log database or analytics suite.

---

## **Deployment Notes**

* Build **statically linked binaries** for target platforms (`cargo build --release`).
* Optional configuration file for filters, colors, and alert patterns.
* Keep the binary self-contained and portable for direct customer use.
