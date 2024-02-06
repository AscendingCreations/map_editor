use serde::{Deserialize, Serialize};
use graphics::*;

pub const MAX_ATTRIBUTE: usize = 4;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MapAttribute {
    Walkable,
    Blocked,
    Warp(i32, i32, u64, u32, u32),
    Sign(String),
    Count,
}

#[derive(Debug)]
pub enum InsertTypes {
    Int(i64),
    UInt(u64),
    Str(String),
    Bool(bool),
}

impl InsertTypes {
    pub fn get_int(&self) -> i64 {
        match self {
            InsertTypes::Int(data) => *data,
            _ => 0,
        }
    }

    pub fn get_uint(&self) -> u64 {
        match self {
            InsertTypes::UInt(data) => *data,
            _ => 0,
        }
    }

    pub fn get_string(&self) -> String {
        match self {
            InsertTypes::Str(data) => data.clone(),
            _ => String::new(),
        }
    }
}

impl MapAttribute {
    pub fn as_str<'a>(attribute: u32) -> &'a str {
        match attribute {
            0 => "Walkable",
            1 => "Blocked",
            2 => "Warp",
            3 => "Sign",
            _ => "",
        }
    }

    pub fn as_map_str<'a>(attribute: &MapAttribute) -> &'a str {
        match attribute {
            MapAttribute::Blocked => "B",
            MapAttribute::Warp(_data1, _data2, _data3, _data4, _data5) => "W",
            MapAttribute::Sign(_datastring) => "S",
            _ => "",
        }
    }

    pub fn get_color(attribute: &MapAttribute) -> Color {
        match attribute {
            MapAttribute::Blocked => Color::rgba(200, 10, 10, 100),
            MapAttribute::Warp(_data1, _data2, _data3, _data_4, _data5) => Color::rgba(10, 10, 200, 100),
            MapAttribute::Sign(_datastring) => Color::rgba(10, 200, 10, 100),
            _ => Color::rgba(0, 0, 0, 0),
        }
    }

    pub fn convert_to_enum(attribute: u32, 
                            data: &[InsertTypes]) -> Self 
    {
        match attribute {
            1 => MapAttribute::Blocked,
            2 => MapAttribute::Warp(data[0].get_int() as i32, 
                                    data[1].get_int() as i32, 
                                    data[2].get_uint(),
                                    data[3].get_uint() as u32,
                                    data[4].get_uint() as u32),
            3 => MapAttribute::Sign(data[0].get_string()),
            _ => MapAttribute::Walkable,
        }
    }

    pub fn convert_to_plain_enum(attribute: u32) -> Self 
    {
        match attribute {
            1 => MapAttribute::Blocked,
            2 => MapAttribute::Warp(0, 0, 0, 0, 0),
            3 => MapAttribute::Sign(String::new()),
            _ => MapAttribute::Walkable,
        }
    }

    pub fn convert_to_num(attribute: &MapAttribute) -> u32 {
        match attribute {
            MapAttribute::Blocked => { 1 },
            MapAttribute::Warp(_data1, 
                                _data2, 
                                _data3,
                                _data4,
                                _data5) => { 2 },
            MapAttribute::Sign(_datastring) => { 3 },
            _ => { 0 },
        }
    }
}