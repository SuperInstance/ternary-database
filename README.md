# ternary-database

Storage, indexing, querying, and transactions for rows with ternary-valued fields (−1, 0, +1).

## Why This Exists

If your data model is ternary — sentiment scores, approval states, sensor classifications — you shouldn't have to shoe-horn that into a binary database. This crate provides a purpose-built storage engine where ternary values are first-class. It includes hash and B-tree indexes optimized for three-valued keys, a builder-pattern query system with filter/sort/limit, aggregation (count and sum by ternary value), transactions with operation logging, and a SQL-like query parser. It's an in-process, in-memory database for ternary data.

## Core Concepts

- **Ternary** — The core value type: Neg (−1), Zero (0), Pos (+1). Stored as an enum, convertible to/from i8.
- **FieldValue** — A row field can hold a Ternary, Integer, Text, TernaryArray, or Null. This lets you mix ternary classifications with conventional data.
- **TernaryTable** — A named collection of rows, stored in a HashMap keyed by auto-incrementing u64 ID.
- **TernaryHashIndex** — A HashMap from Ternary → Vec<row_id>. O(1) lookup for exact ternary matches.
- **TernaryBTreeIndex** — A three-bucket structure (neg_ids, zero_ids, pos_ids) supporting range queries and sorted traversal. Not a true B-tree; it's a partitioned index exploiting the fixed three-value domain.
- **TernaryQuery** — Builder pattern: `.filter(field, op).sort(field, dir).limit(n).execute()`. Filter operators: Eq, Ne, Gt, Lt, Gte, Lte — all comparing against ternary values.
- **Transaction** — Tracks insert, delete, and update operations in a log. Supports commit (marks committed) and exposes the rollback log for manual undo. No automatic rollback — the log is a record for application-level recovery.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-database = "0.1"
```

```rust
use ternary_database::*;

fn main() {
    // Create a table
    let mut table = TernaryTable::new("sensors");

    // Insert rows
    let mut row = Row::new(0);
    row.set("status", FieldValue::Ternary(Ternary::Pos));
    row.set("label", FieldValue::Text("normal".into()));
    let id1 = table.insert(row);

    let mut row = Row::new(0);
    row.set("status", FieldValue::Ternary(Ternary::Neg));
    row.set("label", FieldValue::Text("alert".into()));
    let id2 = table.insert(row);

    // Query with filter
    let results = TernaryQuery::new(&table)
        .filter("status", FilterOp::Eq(Ternary::Pos))
        .execute();
    println!("Positive sensors: {} rows", results.len());

    // Aggregate
    let query = TernaryQuery::new(&table);
    let sum: i64 = query.aggregate_sum("status");
    println!("Net ternary sum: {}", sum); // 1 + (-1) = 0
}
```

## API Overview

| Type | Description |
|------|-------------|
| `Ternary` | Core value: Neg/Zero/Pos |
| `FieldValue` | Polymorphic field: Ternary, Integer, Text, TernaryArray, Null |
| `Row` | A record with u64 ID and HashMap of named fields |
| `TernaryTable` | Named table with insert/get/delete/scan |
| `TernaryHashIndex` | Hash index for exact ternary lookups |
| `TernaryBTreeIndex` | Partitioned index for range queries and sorted access |
| `TernaryQuery` | Builder for filtered, sorted, limited queries |
| `FilterOp` | Comparison operators (Eq, Ne, Gt, Lt, Gte, Lte) |
| `TernaryTransaction` | Operation log with commit tracking |
| `ParsedQuery` | Result of SQL-like query parsing |

## How It Works

**Storage:** Rows live in a `HashMap<u64, Row>`. IDs auto-increment starting at 1. Insertions return the assigned ID.

**Hash index:** After building, the index maps each Ternary value to a list of row IDs. Lookups are O(1) to the vector, then O(k) to retrieve k matching rows.

**B-tree index:** Despite the name, this is a three-bucket partition. Each bucket holds row IDs for one ternary value. `sorted_ids()` concatenates neg → zero → pos. `range(min, max)` includes buckets whose value falls within [min, max]. This is constant-time for the three-value domain.

**Query execution:** The builder collects filters, sort spec, and limit. `execute()` iterates all table rows, applies filters, sorts if requested (by ternary or integer value), and truncates to the limit. This is a full table scan — no index acceleration on queries (indexes are separate lookup structures).

**Aggregation:** `aggregate_count` returns a HashMap from each Ternary value to its count. `aggregate_sum` returns the signed sum of all ternary values in the field.

**Transactions:** Each transaction logs operations (Insert, Delete, Update) with table name and row/field details. `commit()` sets a flag. The rollback log is exposed for application-level undo — the database itself doesn't auto-rollback on drop.

**SQL parser:** A minimal parser handles `SELECT field FROM table [WHERE field op value]`. Values can be specified as `-1`/`0`/`1` or `neg`/`zero`/`pos`. Operators: `=`, `!=`, `>`, `<`, `>=`, `<=`.

## Known Limitations

- **Full table scans on queries.** The `TernaryQuery::execute()` method iterates every row. Indexes exist but aren't integrated into the query planner. For large tables, this is O(n).
- **No concurrent access.** No locking, no MVCC. Single-threaded access only.
- **Transaction rollback is manual.** The log records what happened, but you must write your own undo logic. There's no `rollback()` method that automatically reverses operations.
- **SQL parser is fragile.** It splits on whitespace and expects exact syntax. No quoted identifiers, no AND/OR, no JOINs, no GROUP BY.

## Use Cases

- **Sensor data storage** — Store readings classified as below/above/normal threshold, query by classification, aggregate net signal.
- **Moderation systems** — Rows with ternary approval status (rejected/pending/approved), indexed for fast lookup by status.
- **Configuration management** — Feature flags with ternary state (disabled/neutral/enabled), queryable and auditable through transactions.

## Ecosystem Context

Part of the SuperInstance ternary crate family. `ternary-database` sits at the data layer. `ternary-voting` results can be stored here, `ternary-cell` tissue states can be persisted, and `ternary-visualization` can render query results. The `Ternary` type uses the standard {-1, 0, +1} encoding consistent with all other crates.

## See Also

- **ternary-archive** — Append-only knowledge store with lifecycle management
- **ternary-memory** — Short-term memory and recall for ternary agents
- **ternary-hash** — Hashing algorithms for ternary data
- **ternary-compression** — Compression for ternary-valued streams
- **ternary-tensor** — Tensor operations on ternary-valued arrays

## License

MIT
