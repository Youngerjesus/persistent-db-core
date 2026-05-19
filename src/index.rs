use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrimaryIndex {
    positions_by_key: BTreeMap<i64, usize>,
}

impl PrimaryIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: i64, row_position: usize) -> Result<(), DuplicatePrimaryKey> {
        if self.positions_by_key.contains_key(&key) {
            return Err(DuplicatePrimaryKey);
        }
        self.positions_by_key.insert(key, row_position);
        Ok(())
    }

    pub fn get(&self, key: i64) -> Option<usize> {
        self.positions_by_key.get(&key).copied()
    }

    pub fn remove(&mut self, key: i64) -> Option<usize> {
        self.positions_by_key.remove(&key)
    }

    pub fn ordered_positions(&self) -> Vec<usize> {
        self.positions_by_key.values().copied().collect()
    }

    pub fn len(&self) -> usize {
        self.positions_by_key.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions_by_key.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuplicatePrimaryKey;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SecondaryIndex {
    positions_by_key_and_tie_break: BTreeMap<(i64, i64), usize>,
}

impl SecondaryIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        key: i64,
        tie_break: i64,
        row_position: usize,
    ) -> Result<(), DuplicateSecondaryIndexEntry> {
        let entry_key = (key, tie_break);
        if self.positions_by_key_and_tie_break.contains_key(&entry_key) {
            return Err(DuplicateSecondaryIndexEntry);
        }
        self.positions_by_key_and_tie_break
            .insert(entry_key, row_position);
        Ok(())
    }

    pub fn equality_positions(&self, key: i64) -> Vec<usize> {
        self.positions_by_key_and_tie_break
            .range((key, i64::MIN)..=(key, i64::MAX))
            .map(|(_, row_position)| *row_position)
            .collect()
    }

    pub fn range_positions(&self, low: i64, high: i64) -> Vec<usize> {
        if low > high {
            return Vec::new();
        }
        self.positions_by_key_and_tie_break
            .range((low, i64::MIN)..=(high, i64::MAX))
            .map(|(_, row_position)| *row_position)
            .collect()
    }

    pub fn remove(&mut self, key: i64, tie_break: i64) -> Option<usize> {
        self.positions_by_key_and_tie_break
            .remove(&(key, tie_break))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuplicateSecondaryIndexEntry;
