use graphics::*;
use indexmap::IndexMap;

const MAX_CHANGE: usize = 500;

#[derive(Debug)]
pub struct ChangeData {
    pub pos: Vec3,
    pub texture_id: i32,
}

pub struct Record {
    pub undo: IndexMap<String, ChangeData>,
}
pub struct Records {
    in_record: bool,
    pub data: Vec<Record>,
    last_index: Option<usize>,
}

impl Records {
    pub fn new() -> Self {
        Self {
            in_record: false,
            data: Vec::new(),
            last_index: None,
        }
    }

    pub fn set_record(&mut self) {
        if self.in_record {
            return;
        }

        self.in_record = true;
        let index = self.data.len();
        self.last_index = Some(index);
        self.data.push(Record {
            undo: IndexMap::new()
        });
    }

    pub fn push_change(&mut self, pos: Vec3, texture_id: i32) {
        if !self.in_record {
            return;
        }
        if self.data.len() >= MAX_CHANGE {
            return;
        }

        if let Some(index) = self.last_index {
            let key_name = format!("{}_{}_{}", pos.x, pos.y, pos.z);
            if !self.data[index].undo.contains_key(&key_name) {
                self.data[index].undo.insert(key_name, ChangeData { pos, texture_id });
            }
        }
    }

    pub fn stop_record(&mut self) {
        if !self.in_record {
            return;
        }
        self.in_record = false;
        self.last_index = None;
    }

    pub fn get_last_change(&mut self) -> Option<Record> {
        self.data.pop()
    }
}