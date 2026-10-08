---
"wry": minor
---

On Linux, add `set_default_memory_pressure_settings` and `default_memory_pressure_settings`: a process-wide default that every `WebContext::new` applies afterwards, for applications whose framework creates the `WebContext` itself. Call it before the first webview.
