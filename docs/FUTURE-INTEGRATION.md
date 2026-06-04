# Future Integration: ternary-database

## Current State
Provides in-memory database operations for ternary data: tables with ternary-typed fields, basic queries (insert, select, filter), aggregation (count, sum, ternary tally), and indexing for fast lookup.

## Integration Opportunities

### With ternary-room (Room State Persistence)
Rooms need persistent state. `TernaryTable` stores room state histories: each row is a room snapshot, each column is a state dimension (energy, population, health). `select()` queries room histories. `ternary_tally()` aggregates room state distributions over time. This is the room historian.

### With ternary-compression-v2
Historical data grows. Compression keeps it manageable. `ternary-database` stores raw data; `ternary-compression-v2` compresses old data. Hot data (recent) stays uncompressed for fast queries. Cold data (historical) is compressed for storage efficiency. Entropy coding tracks information density per time period.

### With ternary-protocol (Message Logging)
All protocol messages need logging for debugging and audit. `TernaryTable` with columns for sender, receiver, payload type, and ternary content. `filter()` queries message logs by any dimension. `index()` on sender/receiver for fast per-agent or per-room retrieval.

## Potential in Mature Systems
In room-as-codespace, `ternary-database` is the local room database. Each Codespace runs an instance. PLATO synchronizes tables between rooms. Room state, message logs, and agent histories all live in ternary tables. The ternary schema is naturally compact — three-state fields compress well and query fast.

## Cross-Pollination Ideas
- Ternary tally as a room health dashboard — distribution of states over time shows trends
- Database indices as room bookmarks — fast access to frequently queried room states
- Cross-room table joins for fleet-wide analysis (when rooms share a database server)

## Dependencies for Next Steps
- ternary-room needs database integration for state persistence
- Integration with ternary-compression-v2 for cold data compression
- ternary-protocol needs message logging to ternary tables
