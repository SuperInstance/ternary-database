#![forbid(unsafe_code)]

//! Database operations for ternary data.

use std::collections::HashMap;

/// A ternary value stored in the database.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ternary {
    Neg,
    Zero,
    Pos,
}

impl Ternary {
    pub fn value(self) -> i8 {
        match self {
            Ternary::Neg => -1,
            Ternary::Zero => 0,
            Ternary::Pos => 1,
        }
    }

    pub fn from_value(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Neg),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Pos),
            _ => None,
        }
    }
}

/// A field value in a ternary database row.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    Ternary(Ternary),
    Integer(i64),
    Text(String),
    TernaryArray(Vec<Ternary>),
    Null,
}

/// A row in a ternary table.
#[derive(Clone, Debug)]
pub struct Row {
    pub id: u64,
    pub fields: HashMap<String, FieldValue>,
}

impl Row {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            fields: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: FieldValue) {
        self.fields.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&FieldValue> {
        self.fields.get(key)
    }
}

/// A ternary table storing rows.
pub struct TernaryTable {
    pub name: String,
    rows: HashMap<u64, Row>,
    next_id: u64,
}

impl TernaryTable {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rows: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn insert(&mut self, mut row: Row) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        row.id = id;
        self.rows.insert(id, row);
        id
    }

    pub fn get(&self, id: u64) -> Option<&Row> {
        self.rows.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Row> {
        self.rows.get_mut(&id)
    }

    pub fn delete(&mut self, id: u64) -> bool {
        self.rows.remove(&id).is_some()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn rows(&self) -> impl Iterator<Item = &Row> {
        self.rows.values()
    }

    pub fn scan_all(&self) -> Vec<&Row> {
        self.rows.values().collect()
    }
}

/// A ternary hash index for fast lookups on ternary fields.
pub struct TernaryHashIndex {
    pub field: String,
    index: HashMap<Ternary, Vec<u64>>,
}

impl TernaryHashIndex {
    pub fn new(field: &str) -> Self {
        Self {
            field: field.to_string(),
            index: HashMap::new(),
        }
    }

    pub fn build(&mut self, table: &TernaryTable) {
        self.index.clear();
        for row in table.rows() {
            if let Some(FieldValue::Ternary(t)) = row.get(&self.field) {
                self.index.entry(*t).or_default().push(row.id);
            }
        }
    }

    pub fn lookup(&self, key: Ternary) -> &[u64] {
        self.index.get(&key).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// A simple B-tree-like structure for ternary key ordering.
pub struct TernaryBTreeIndex {
    pub field: String,
    neg_ids: Vec<u64>,
    zero_ids: Vec<u64>,
    pos_ids: Vec<u64>,
}

impl TernaryBTreeIndex {
    pub fn new(field: &str) -> Self {
        Self {
            field: field.to_string(),
            neg_ids: Vec::new(),
            zero_ids: Vec::new(),
            pos_ids: Vec::new(),
        }
    }

    pub fn build(&mut self, table: &TernaryTable) {
        self.neg_ids.clear();
        self.zero_ids.clear();
        self.pos_ids.clear();
        for row in table.rows() {
            if let Some(FieldValue::Ternary(t)) = row.get(&self.field) {
                match t {
                    Ternary::Neg => self.neg_ids.push(row.id),
                    Ternary::Zero => self.zero_ids.push(row.id),
                    Ternary::Pos => self.pos_ids.push(row.id),
                }
            }
        }
    }

    /// Return row IDs sorted by ternary key value.
    pub fn sorted_ids(&self) -> Vec<u64> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.neg_ids);
        result.extend_from_slice(&self.zero_ids);
        result.extend_from_slice(&self.pos_ids);
        result
    }

    pub fn range(&self, min: Ternary, max: Ternary) -> Vec<u64> {
        let mut result = Vec::new();
        let min_v = min.value();
        let max_v = max.value();
        if min_v <= -1 && max_v >= -1 {
            result.extend_from_slice(&self.neg_ids);
        }
        if min_v <= 0 && max_v >= 0 {
            result.extend_from_slice(&self.zero_ids);
        }
        if min_v <= 1 && max_v >= 1 {
            result.extend_from_slice(&self.pos_ids);
        }
        result
    }
}

/// Filter operator for queries.
#[derive(Clone, Debug)]
pub enum FilterOp {
    Eq(Ternary),
    Ne(Ternary),
    Gt(Ternary),
    Lt(Ternary),
    Gte(Ternary),
    Lte(Ternary),
}

/// Sort direction.
#[derive(Clone, Copy, Debug)]
pub enum SortDir {
    Asc,
    Desc,
}

/// Query structure for ternary data.
pub struct TernaryQuery<'a> {
    table: &'a TernaryTable,
    filters: Vec<(String, FilterOp)>,
    sort_field: Option<(String, SortDir)>,
    limit: Option<usize>,
}

impl<'a> TernaryQuery<'a> {
    pub fn new(table: &'a TernaryTable) -> Self {
        Self {
            table,
            filters: Vec::new(),
            sort_field: None,
            limit: None,
        }
    }

    pub fn filter(mut self, field: &str, op: FilterOp) -> Self {
        self.filters.push((field.to_string(), op));
        self
    }

    pub fn sort(mut self, field: &str, dir: SortDir) -> Self {
        self.sort_field = Some((field.to_string(), dir));
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    fn matches_filter(&self, row: &Row, field: &str, op: &FilterOp) -> bool {
        let val = match row.get(field) {
            Some(FieldValue::Ternary(t)) => t.value(),
            Some(FieldValue::Integer(i)) => *i as i8,
            _ => return false,
        };
        match op {
            FilterOp::Eq(t) => val == t.value(),
            FilterOp::Ne(t) => val != t.value(),
            FilterOp::Gt(t) => val > t.value(),
            FilterOp::Lt(t) => val < t.value(),
            FilterOp::Gte(t) => val >= t.value(),
            FilterOp::Lte(t) => val <= t.value(),
        }
    }

    pub fn execute(&self) -> Vec<&Row> {
        let mut results: Vec<&Row> = self
            .table
            .rows()
            .filter(|row| {
                self.filters
                    .iter()
                    .all(|(f, op)| self.matches_filter(row, f, op))
            })
            .collect();

        if let Some((ref field, dir)) = self.sort_field {
            results.sort_by(|a, b| {
                let va = a.get(field).and_then(|v| match v {
                    FieldValue::Ternary(t) => Some(t.value() as i64),
                    FieldValue::Integer(i) => Some(*i),
                    _ => None,
                });
                let vb = b.get(field).and_then(|v| match v {
                    FieldValue::Ternary(t) => Some(t.value() as i64),
                    FieldValue::Integer(i) => Some(*i),
                    _ => None,
                });
                let cmp = va.cmp(&vb);
                match dir {
                    SortDir::Asc => cmp,
                    SortDir::Desc => cmp.reverse(),
                }
            });
        }

        if let Some(limit) = self.limit {
            results.truncate(limit);
        }

        results
    }

    /// Aggregate: count rows per ternary value in a field.
    pub fn aggregate_count(&self, field: &str) -> HashMap<Ternary, usize> {
        let mut counts = HashMap::new();
        for row in self.table.rows() {
            if let Some(FieldValue::Ternary(t)) = row.get(field) {
                *counts.entry(*t).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Aggregate: sum of ternary values in a field.
    pub fn aggregate_sum(&self, field: &str) -> i64 {
        self.table
            .rows()
            .filter_map(|row| match row.get(field) {
                Some(FieldValue::Ternary(t)) => Some(t.value() as i64),
                _ => None,
            })
            .sum()
    }
}

/// A simple transaction log entry.
#[derive(Clone, Debug)]
pub enum TxnOp {
    Insert(u64, String), // row_id, table_name
    Delete(u64, String),
    Update(u64, String, String, FieldValue), // row_id, table, field, value
}

/// A transaction that tracks operations for rollback.
pub struct TernaryTransaction {
    pub id: u64,
    log: Vec<TxnOp>,
    committed: bool,
}

impl TernaryTransaction {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            log: Vec::new(),
            committed: false,
        }
    }

    pub fn log_insert(&mut self, row_id: u64, table_name: &str) {
        self.log.push(TxnOp::Insert(row_id, table_name.to_string()));
    }

    pub fn log_delete(&mut self, row_id: u64, table_name: &str) {
        self.log.push(TxnOp::Delete(row_id, table_name.to_string()));
    }

    pub fn log_update(&mut self, row_id: u64, table: &str, field: &str, value: FieldValue) {
        self.log.push(TxnOp::Update(row_id, table.to_string(), field.to_string(), value));
    }

    pub fn commit(&mut self) {
        self.committed = true;
    }

    pub fn is_committed(&self) -> bool {
        self.committed
    }

    pub fn rollback_log(&self) -> &[TxnOp] {
        &self.log
    }

    pub fn op_count(&self) -> usize {
        self.log.len()
    }
}

/// A simple SQL-like query parser for ternary data.
pub fn parse_ternary_query(sql: &str) -> Result<ParsedQuery, String> {
    let sql = sql.trim();
    let lower = sql.to_lowercase();

    if !lower.starts_with("select") {
        return Err("Query must start with SELECT".to_string());
    }

    let mut parts: Vec<&str> = sql.splitn(4, ' ').collect();
    if parts.len() < 4 {
        return Err("Incomplete query".to_string());
    }

    // Simple: SELECT field FROM table WHERE field op value
    let select_field = parts[1].to_string();

    // Find FROM
    let rest = &sql[7..]; // after "SELECT "
    let from_pos = rest.to_lowercase().find("from").ok_or("Missing FROM")?;
    let table_part = rest[from_pos + 5..].trim();
    let table_name: String = table_part.split_whitespace().next().unwrap_or("").to_string();

    if table_name.is_empty() {
        return Err("Missing table name".to_string());
    }

    // Find optional WHERE
    let mut filter = None;
    if let Some(where_pos) = table_part.to_lowercase().find("where") {
        let where_clause = table_part[where_pos + 6..].trim();
        let tokens: Vec<&str> = where_clause.split_whitespace().collect();
        if tokens.len() >= 3 {
            let field = tokens[0].to_string();
            let op = tokens[1];
            let val = parse_ternary_value(tokens[2])?;
            let fop = match op {
                "=" => FilterOp::Eq(val),
                "!=" => FilterOp::Ne(val),
                ">" => FilterOp::Gt(val),
                "<" => FilterOp::Lt(val),
                ">=" => FilterOp::Gte(val),
                "<=" => FilterOp::Lte(val),
                _ => return Err(format!("Unknown operator: {}", op)),
            };
            filter = Some((field, fop));
        }
    }

    Ok(ParsedQuery {
        select_field,
        table_name,
        filter,
    })
}

fn parse_ternary_value(s: &str) -> Result<Ternary, String> {
    match s.trim().trim_matches('\'') {
        "-1" | "neg" | "NEG" => Ok(Ternary::Neg),
        "0" | "zero" | "ZERO" => Ok(Ternary::Zero),
        "1" | "pos" | "POS" => Ok(Ternary::Pos),
        _ => Err(format!("Invalid ternary value: {}", s)),
    }
}

/// A parsed query result.
pub struct ParsedQuery {
    pub select_field: String,
    pub table_name: String,
    pub filter: Option<(String, FilterOp)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_values() {
        assert_eq!(Ternary::Neg.value(), -1);
        assert_eq!(Ternary::Zero.value(), 0);
        assert_eq!(Ternary::Pos.value(), 1);
    }

    #[test]
    fn test_ternary_from_value() {
        assert_eq!(Ternary::from_value(-1), Some(Ternary::Neg));
        assert_eq!(Ternary::from_value(2), None);
    }

    #[test]
    fn test_row_set_get() {
        let mut row = Row::new(1);
        row.set("status", FieldValue::Ternary(Ternary::Pos));
        assert_eq!(row.get("status"), Some(&FieldValue::Ternary(Ternary::Pos)));
        assert_eq!(row.get("missing"), None);
    }

    #[test]
    fn test_table_insert_and_get() {
        let mut table = TernaryTable::new("test");
        let mut row = Row::new(0);
        row.set("val", FieldValue::Ternary(Ternary::Neg));
        let id = table.insert(row);
        assert_eq!(id, 1);
        let fetched = table.get(id).unwrap();
        assert_eq!(fetched.id, id);
    }

    #[test]
    fn test_table_delete() {
        let mut table = TernaryTable::new("test");
        let row = Row::new(0);
        let id = table.insert(row);
        assert!(table.delete(id));
        assert!(!table.delete(id));
        assert!(table.get(id).is_none());
    }

    #[test]
    fn test_table_len_and_empty() {
        let mut table = TernaryTable::new("test");
        assert!(table.is_empty());
        table.insert(Row::new(0));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_hash_index() {
        let mut table = TernaryTable::new("test");
        for t in [Ternary::Pos, Ternary::Neg, Ternary::Pos] {
            let mut row = Row::new(0);
            row.set("status", FieldValue::Ternary(t));
            table.insert(row);
        }
        let mut idx = TernaryHashIndex::new("status");
        idx.build(&table);
        assert_eq!(idx.lookup(Ternary::Pos).len(), 2);
        assert_eq!(idx.lookup(Ternary::Neg).len(), 1);
        assert_eq!(idx.lookup(Ternary::Zero).len(), 0);
    }

    #[test]
    fn test_btree_index_sorted() {
        let mut table = TernaryTable::new("test");
        for t in [Ternary::Pos, Ternary::Neg, Ternary::Zero] {
            let mut row = Row::new(0);
            row.set("level", FieldValue::Ternary(t));
            table.insert(row);
        }
        let mut idx = TernaryBTreeIndex::new("level");
        idx.build(&table);
        let sorted = idx.sorted_ids();
        assert_eq!(sorted.len(), 3);
        // First should be Neg, then Zero, then Pos
    }

    #[test]
    fn test_btree_index_range() {
        let mut table = TernaryTable::new("test");
        for t in [Ternary::Pos, Ternary::Neg, Ternary::Zero, Ternary::Pos] {
            let mut row = Row::new(0);
            row.set("x", FieldValue::Ternary(t));
            table.insert(row);
        }
        let mut idx = TernaryBTreeIndex::new("x");
        idx.build(&table);
        let range = idx.range(Ternary::Neg, Ternary::Zero);
        assert_eq!(range.len(), 2); // Neg + Zero
    }

    #[test]
    fn test_query_filter_eq() {
        let mut table = TernaryTable::new("test");
        for t in [Ternary::Pos, Ternary::Neg, Ternary::Pos] {
            let mut row = Row::new(0);
            row.set("status", FieldValue::Ternary(t));
            table.insert(row);
        }
        let query = TernaryQuery::new(&table)
            .filter("status", FilterOp::Eq(Ternary::Pos));
        let results = query.execute();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_filter_gt() {
        let mut table = TernaryTable::new("test");
        for t in [Ternary::Pos, Ternary::Neg, Ternary::Zero] {
            let mut row = Row::new(0);
            row.set("val", FieldValue::Ternary(t));
            table.insert(row);
        }
        let query = TernaryQuery::new(&table)
            .filter("val", FilterOp::Gt(Ternary::Neg));
        let results = query.execute();
        assert_eq!(results.len(), 2); // Zero and Pos
    }

    #[test]
    fn test_query_sort() {
        let mut table = TernaryTable::new("test");
        for t in [Ternary::Pos, Ternary::Neg, Ternary::Zero] {
            let mut row = Row::new(0);
            row.set("val", FieldValue::Ternary(t));
            table.insert(row);
        }
        let query = TernaryQuery::new(&table)
            .sort("val", SortDir::Asc);
        let results = query.execute();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_query_limit() {
        let mut table = TernaryTable::new("test");
        for _ in 0..10 {
            let mut row = Row::new(0);
            row.set("val", FieldValue::Ternary(Ternary::Pos));
            table.insert(row);
        }
        let query = TernaryQuery::new(&table).limit(3);
        let results = query.execute();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_aggregate_count() {
        let mut table = TernaryTable::new("test");
        for t in [Ternary::Pos, Ternary::Neg, Ternary::Pos] {
            let mut row = Row::new(0);
            row.set("x", FieldValue::Ternary(t));
            table.insert(row);
        }
        let query = TernaryQuery::new(&table);
        let counts = query.aggregate_count("x");
        assert_eq!(counts.get(&Ternary::Pos), Some(&2));
        assert_eq!(counts.get(&Ternary::Neg), Some(&1));
    }

    #[test]
    fn test_aggregate_sum() {
        let mut table = TernaryTable::new("test");
        for t in [Ternary::Pos, Ternary::Neg, Ternary::Pos, Ternary::Zero] {
            let mut row = Row::new(0);
            row.set("x", FieldValue::Ternary(t));
            table.insert(row);
        }
        let query = TernaryQuery::new(&table);
        assert_eq!(query.aggregate_sum("x"), 1); // 1 -1 + 1 + 0 = 1
    }

    #[test]
    fn test_transaction_commit() {
        let mut txn = TernaryTransaction::new(1);
        txn.log_insert(1, "test");
        txn.log_insert(2, "test");
        txn.commit();
        assert!(txn.is_committed());
        assert_eq!(txn.op_count(), 2);
    }

    #[test]
    fn test_transaction_rollback_log() {
        let mut txn = TernaryTransaction::new(1);
        txn.log_insert(1, "test");
        txn.log_delete(2, "test");
        txn.log_update(3, "test", "field", FieldValue::Ternary(Ternary::Pos));
        assert_eq!(txn.rollback_log().len(), 3);
        assert!(!txn.is_committed());
    }

    #[test]
    fn test_parse_query_simple() {
        let q = parse_ternary_query("SELECT status FROM sensors WHERE status = pos").unwrap();
        assert_eq!(q.select_field, "status");
        assert_eq!(q.table_name, "sensors");
        assert!(q.filter.is_some());
    }

    #[test]
    fn test_parse_query_no_where() {
        let q = parse_ternary_query("SELECT val FROM data").unwrap();
        assert_eq!(q.select_field, "val");
        assert_eq!(q.table_name, "data");
        assert!(q.filter.is_none());
    }

    #[test]
    fn test_parse_query_invalid() {
        assert!(parse_ternary_query("INVALID").is_err());
        assert!(parse_ternary_query("SELECT x").is_err());
    }

    #[test]
    fn test_field_value_types() {
        let fv1 = FieldValue::Integer(42);
        let fv2 = FieldValue::Text("hello".to_string());
        let fv3 = FieldValue::Null;
        let _fv4 = FieldValue::TernaryArray(vec![Ternary::Pos, Ternary::Neg]);
        assert_eq!(fv1, FieldValue::Integer(42));
        assert_eq!(fv2, FieldValue::Text("hello".to_string()));
        assert_eq!(fv3, FieldValue::Null);
    }
}
