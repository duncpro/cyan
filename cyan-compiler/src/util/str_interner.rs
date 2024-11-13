use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, MutexGuard, RwLock};
use crate::util::str_list::{StrRef, StrList, StrListKey};
use crate::util::bits::fast_hash;

#[derive(Default, Debug)]
pub struct StrInterner {
    table: Mutex<Table>,
    str_list: StrList
}

impl StrInterner {  
    pub fn intern(&self, s: &[u8]) -> StrListKey { 
        let table = &mut *self.table.lock().unwrap();
        if let Some(str_list_key) = lookup(table, &self.str_list, s) {
            return str_list_key;
        }
        if (table.occupancy as f64) / (table.arr.len() as f64) >= 0.75 {
            grow_table(table, &self.str_list);
        }
        let new_str_list_key = self.str_list.push(s);
        insert(table, &self.str_list, new_str_list_key);
        return new_str_list_key;
    }

    pub fn str_list(&self) -> &StrList { return &self.str_list; }
}

#[derive(Default, Debug)]
struct Table { 
    arr: Vec<Option<StrListKey>>,
    occupancy: usize
}

fn lookup(table: &mut Table, str_list: &StrList, s: &[u8]) -> Option<StrListKey> {
    let place = fast_hash(s).checked_rem(table.arr.len())?;
    for idx in place..table.arr.len() {
        let occupant = table.arr[idx]?;
        if str_list.get(occupant) == s {
            return Some(occupant);
        }
    }
    return None;
}

fn insert(table: &mut Table, str_list: &StrList, new_str_list_key: StrListKey) {
    let new_str = str_list.get(new_str_list_key);
    loop {
        let place = fast_hash(new_str).checked_rem(table.arr.len()).unwrap_or(0);
        for idx in place..table.arr.len() {
            if table.arr[idx].is_none() {
                table.arr[idx] = Some(new_str_list_key);
                table.occupancy += 1;
                return;
            }
        }
        grow_table(table, str_list);
    }
}

fn grow_table(table: &mut Table, str_list: &StrList) {
    let new_capacity = usize::max(1, table.arr.len()) * 2;
    let mut new_table = Table::default();
    new_table.arr = vec![None; new_capacity];
    for entry in &table.arr {
        if let Some(str_list_key) = entry {
            insert(&mut new_table, str_list, *str_list_key);
        }
    }
    *table = new_table;
}
