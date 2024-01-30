use graphics::*;
use indexmap::IndexMap;

const MAX_CHANGE: usize = 500;

#[derive(Debug)]
pub struct ChangeData {
    pub pos: Vec3,
    pub texture_id: i32,
}

#[derive(Debug)]
pub struct Record {
    pub changes: IndexMap<String, ChangeData>,
}
pub struct Records {
    in_record: bool,
    pub undo: Vec<Record>,
    pub redo: Vec<Record>,
    last_index: Option<usize>,
}

impl Records {
    pub fn new() -> Self {
        Self {
            in_record: false,
            undo: Vec::new(),
            redo: Vec::new(),
            last_index: None,
        }
    }

    pub fn set_undo_record(&mut self) {
        if self.in_record {
            return;
        }

        self.in_record = true;
        let index = self.undo.len();
        self.last_index = Some(index);
        self.undo.push(Record {
            changes: IndexMap::new(),
        });
    }

    pub fn push_undo(&mut self, pos: Vec3, texture_id: i32) {
        if !self.in_record {
            return;
        }
        if self.undo.len() >= MAX_CHANGE {
            return;
        }

        if let Some(index) = self.last_index {
            let key_name = format!("{}_{}_{}", pos.x, pos.y, pos.z);
            if !self.undo[index].changes.contains_key(&key_name) {
                self.undo[index].changes.insert(key_name, ChangeData { pos, texture_id });
            }
        }
    }

    pub fn get_last_undo(&mut self) -> Option<Record> {
        self.undo.pop()
    }

    pub fn set_redo_record(&mut self) {
        if self.in_record {
            return;
        }

        self.in_record = true;
        let index = self.redo.len();
        self.last_index = Some(index);
        self.redo.push(Record {
            changes: IndexMap::new(),
        });
    }

    pub fn push_redo(&mut self, pos: Vec3, texture_id: i32) {
        if !self.in_record {
            return;
        }
        if self.redo.len() >= MAX_CHANGE {
            return;
        }

        if let Some(index) = self.last_index {
            let key_name = format!("{}_{}_{}", pos.x, pos.y, pos.z);
            if !self.redo[index].changes.contains_key(&key_name) {
                self.redo[index].changes.insert(key_name, ChangeData { pos, texture_id });
            }
        }
    }

    pub fn get_last_redo(&mut self) -> Option<Record> {
        self.redo.pop()
    }

    pub fn clear_redo(&mut self) {
        if self.redo.len() > 0 {
            self.redo = Vec::new();
        }
    }

    pub fn stop_record(&mut self) {
        if !self.in_record {
            return;
        }
        self.in_record = false;
        self.last_index = None;
    }
}