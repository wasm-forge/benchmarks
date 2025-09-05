# `polyfill-benchmarks`

This benchmark estimates creating and listing folders from the `ic-wasi-polyfill` perspective.

## Improvement on switching from v0.9.0 to v0.10.0

<pre>
---------------------------------------------------

Benchmark: create_1000_folders
  total:
    instructions: 20.13 B (improved by 9.95%)
    heap_increase: 0 pages (improved by 100.00%)
    stable_memory_increase: 256 pages (no change)

---------------------------------------------------

Benchmark: create_1000_folders_1000_subfolders
  total:
    instructions: 20.45 B (improved by 69.24%)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 256 pages (no change)

---------------------------------------------------

Benchmark: list_1000_folders
  total:
    instructions: 34.44 M (improved by 98.75%)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------
</pre>