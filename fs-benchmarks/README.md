# `fs-benchmarks`

Benchmarks designed to test performance of the stable-fs file system implementation based on the stable structures.

## Usage

Install and launch canbench:
```bash
  cargo install canbench
```

## Benchmarking stable-fs v0.8
<table>
<tr>
<th> regular files, 16K chunk size </th>
<th> mounted memory files </th>
</tr>
<tr>
<td>


<pre>---------------------------------------------------

Benchmark: <b>write_100mb</b>
  total:
    instructions: 729.73 M (no change)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 1536 pages (no change)

---------------------------------------------------

Benchmark: <b>write_100mb_over_existing</b>
  total:
    instructions: 125.65 M (no change)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

Benchmark: <b>read_100mb</b>
  total:
    instructions: 125.27 M (no change)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

Benchmark: <b>write_100mb_in_segments</b>
  total:
    instructions: 1.50 B (no change)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 1536 pages (no change)

---------------------------------------------------

Benchmark: <b>write_100mb_in_segments_over_existing</b>
  total:
    instructions: 549.97 M (no change)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

Benchmark: <b>read_100mb_in_segments</b>
  total:
    instructions: 541.02 M (no change)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

Benchmark: <b>write_100mb_in_segments_10_files</b>
  total:
    instructions: 1.54 B (no change)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 2433 pages (no change)

---------------------------------------------------

Benchmark: <b>write_100mb_in_segments_over_existing_10_files</b>
  total:
    instructions: 666.82 M (no change)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

Benchmark: <b>read_100mb_in_segments_from_10_files</b>
  total:
    instructions: 630.65 M (no change)
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

</pre>


</td>
<td>

<pre>---------------------------------------------------

Benchmark: <b>write_100mb</b>
  total:
    <font color="#8AE234"><b>instructions: 124.45 M (improved by 90.55%)</b></font>
    heap_increase: 0 pages (no change)
    stable_memory_increase: 1536 pages (no change)

---------------------------------------------------

Benchmark: <b>write_100mb_over_existing</b>
  total:
    <font color="#8AE234"><b>instructions: 100.01 M (improved by 29.93%)</b></font>
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

Benchmark: <b>read_100mb</b>
  total:
    <font color="#8AE234"><b>instructions: 100.02 M (improved by 29.74%)</b></font>
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

Benchmark: <b>write_100mb_in_segments</b>
  total:
    <font color="#8AE234"><b>instructions: 918.28 M (improved by 59.34%)</b></font>
    heap_increase: 0 pages (no change)
    stable_memory_increase: 1536 pages (no change)

---------------------------------------------------

Benchmark: <b>write_100mb_in_segments_over_existing</b>
  total:
    <font color="#8AE234"><b>instructions: 519.91 M (improved by 20.73%)</b></font>
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

Benchmark: <b>read_100mb_in_segments</b>
  total:
    <font color="#8AE234"><b>instructions: 476.11 M (improved by 23.33%)</b></font>
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

Benchmark: <b>write_100mb_in_segments_10_files</b>
  total:
    <font color="#8AE234"><b>instructions: 1.11 B (improved by 51.48%)</b></font>
    heap_increase: 0 pages (no change)
    <font color="#EF2929"><b>stable_memory_increase: 3585 pages (regressed by 47.35%)</b></font>

---------------------------------------------------

Benchmark: <b>write_100mb_in_segments_over_existing_10_files</b>
  total:
    <font color="#8AE234"><b>instructions: 699.55 M (improved by 14.62%)</b></font>
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------

Benchmark: <b>read_100mb_in_segments_from_10_files</b>
  total:
    <font color="#8AE234"><b>instructions: 655.03 M (improved by 11.11%)</b></font>
    heap_increase: 0 pages (no change)
    stable_memory_increase: 0 pages (no change)

---------------------------------------------------
</pre>


</td>
</tr>
</table>



