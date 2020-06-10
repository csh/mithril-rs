use std::io::{prelude::*, Cursor, SeekFrom};

use bytes::Buf;
use mithril_buf::GameBuf;

use crate::{ArchiveError, CacheError, CacheFileSystem};

#[derive(Debug)]
pub struct ItemDefinition {
    id: u16,
    name: String,
    examine_text: String,
    member_only: bool,
    stackable: bool,
    ground_actions: [Option<String>; 5],
    inventory_actions: [Option<String>; 5],
    noted_sprite_id: Option<u16>,
    noted_info_id: Option<u16>,
    value: i32,
    team: Option<u8>,
}

impl Default for ItemDefinition {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::default(),
            examine_text: String::default(),
            member_only: false,
            stackable: false,
            ground_actions: [None, None, None, None, None],
            inventory_actions: [None, None, None, None, None],
            noted_sprite_id: None,
            noted_info_id: None,
            value: 0,
            team: None,
        }
    }
}

impl ItemDefinition {
    pub fn load(cache: &mut CacheFileSystem) -> crate::Result<Vec<Self>> {
        let archive = cache.get_archive(0, 2)?;
        let mut index = archive
            .get_entry("obj.idx")
            .map(|entry| Cursor::new(entry.contents()))
            .ok_or(ArchiveError::EntryNotFound("obj.idx"))?;

        let mut data = archive
            .get_entry("obj.dat")
            .map(|entry| Cursor::new(entry.contents()))
            .ok_or(ArchiveError::EntryNotFound("obj.dat"))?;

        let entries = index.get_u16() as usize;
        let mut offsets = vec![0; entries];
        let mut position: u64 = 2;

        for offset in offsets.iter_mut() {
            *offset = position;
            position += index.get_u16() as u64;
        }

        let mut definitions: Vec<Self> = Vec::with_capacity(entries);
        for (id, offset) in offsets.iter().enumerate() {
            data.seek(SeekFrom::Start(*offset))?;

            let mut definition = decode_definition(id as u16, &mut data)?;
            if let Some(lookup_id) = definition.noted_info_id {
                if let Some(template) = definitions.get(lookup_id as usize) {
                    definition.name = template.name.clone();
                    definition.examine_text =
                        format!("Swap this item at any bank for {}", &template.name);
                    definition.member_only = template.member_only;
                    definition.value = template.value;
                    definition.stackable = true;
                }
            }
            definitions.push(definition);
        }
        Ok(definitions)
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn examine_text(&self) -> &String {
        &self.examine_text
    }

    pub fn is_member_only(&self) -> bool {
        self.member_only
    }

    pub fn is_stackable(&self) -> bool {
        self.stackable
    }

    pub fn is_noted(&self) -> bool {
        self.noted_info_id.is_some()
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn ground_action(&self, index: usize) -> Option<&String> {
        self.ground_actions[index].as_ref()
    }

    pub fn inventory_action(&self, index: usize) -> Option<&String> {
        self.inventory_actions[index].as_ref()
    }

    pub fn team(&self) -> Option<u8> {
        self.team
    }
}

// TODO: Can this be made less ugly?
fn decode_definition<B: GameBuf>(item_id: u16, buf: &mut B) -> crate::Result<ItemDefinition> {
    let mut definition = ItemDefinition::default();
    definition.id = item_id;
    loop {
        match buf.get_u8() {
            0 => {
                return Ok(definition);
            }
            1 => {
                buf.get_u16();
            }
            2 => {
                definition.name = buf.get_rs_string();
            }
            3 => {
                definition.examine_text = buf.get_rs_string();
            }
            4..=8 | 10 => {
                buf.get_u16();
            }
            11 => {
                definition.stackable = true;
            }
            12 => {
                definition.value = buf.get_i32();
            }
            16 => {
                definition.member_only = true;
            }
            23 => {
                buf.get_u16();
                buf.get_u8();
            }
            24 => {
                buf.get_u16();
            }
            25 => {
                buf.get_u16();
                buf.get_u8();
            }
            26 => {
                buf.get_u16();
            }
            opcode if (30..=34).contains(&opcode) => {
                definition.ground_actions[opcode as usize - 30] = Some(buf.get_rs_string());
            }
            opcode if (35..=39).contains(&opcode) => {
                definition.inventory_actions[opcode as usize - 35] = Some(buf.get_rs_string());
            }
            40 => {
                let colours = buf.get_u8();
                for _ in 0..colours {
                    buf.get_u16();
                    buf.get_u16();
                }
            }
            78 | 79 | 90..=93 | 95 => {
                buf.get_u16();
            }
            97 => {
                definition.noted_info_id = Some(buf.get_u16());
            }
            98 => {
                definition.noted_sprite_id = Some(buf.get_u16());
            }
            100..=109 => {
                buf.get_u16();
                buf.get_u16();
            }
            110..=112 => {
                buf.get_u16();
            }
            113 | 114 => {
                buf.get_u8();
            }
            115 => {
                definition.team = Some(buf.get_u8());
            }
            opcode => {
                return Err(CacheError::DecodeDefinition { ty: "Item", opcode });
            }
        }
    }
}
