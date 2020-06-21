use crate::{ArchiveError, CacheFileSystem};
use bytes::Buf;
use mithril_buf::GameBuf;
use std::io::{prelude::*, Cursor, SeekFrom};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct ObjectDefinition {
    id: u16,
    name: String,
    examine_text: String,
    impenetrable: bool,
    interactive: bool,
    obstructive: bool,
    solid: bool,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "crate::skip_empty_options")
    )]
    interact_actions: [Option<String>; 10],
    length: u8,
    width: u8,
    rotated: bool,
    casts_shadow: bool,
    hug_terrain: bool,
    low_priority_shading: bool,
    wall: bool,
}

impl Default for ObjectDefinition {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::default(),
            examine_text: String::default(),
            impenetrable: true,
            interactive: false,
            obstructive: false,
            solid: true,
            interact_actions: [None, None, None, None, None, None, None, None, None, None],
            length: 1,
            width: 1,
            rotated: false,
            casts_shadow: true,
            hug_terrain: false,
            low_priority_shading: true,
            wall: false,
        }
    }
}

impl ObjectDefinition {
    pub fn load(cache: &mut CacheFileSystem) -> crate::Result<Vec<Self>> {
        let archive = cache.get_archive(0, 2)?;
        let mut index = archive
            .get_entry("loc.idx")
            .map(|entry| Cursor::new(entry.contents()))
            .ok_or(ArchiveError::EntryNotFound("loc.idx"))?;

        let mut data = archive
            .get_entry("loc.dat")
            .map(|entry| Cursor::new(entry.contents()))
            .ok_or(ArchiveError::EntryNotFound("loc.dat"))?;

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
            let definition = decode_definition(id as u16, &mut data)?;
            definitions.push(definition);
        }
        Ok(definitions)
    }
}

fn decode_definition<B: GameBuf>(object_id: u16, buf: &mut B) -> crate::Result<ObjectDefinition> {
    let mut definition = ObjectDefinition::default();
    definition.id = object_id;
    loop {
        match buf.get_u8() {
            0 => return Ok(definition),
            1 => {
                let len = buf.get_u8();
                let _ = (0..len)
                    .map(|_| (buf.get_u16(), buf.get_u8()))
                    .collect::<Vec<_>>();
            }
            2 => definition.name = buf.get_rs_string(),
            3 => definition.examine_text = buf.get_rs_string(),
            5 => {
                let len = buf.get_u8();
                let _ = (0..len).map(|_| buf.get_u16()).collect::<Vec<_>>();
            }
            14 => definition.width = buf.get_u8(),
            15 => definition.length = buf.get_u8(),
            17 => definition.solid = false,
            18 => definition.impenetrable = false,
            19 => definition.interactive = buf.get_u8() == 1,
            21 => definition.hug_terrain = true,
            22 => definition.low_priority_shading = true,
            23 => definition.wall = true,
            24 => {
                buf.get_u16();
            }
            28 | 29 => {
                buf.get_u8();
            }
            opcode if (30..39).contains(&opcode) => {
                definition.interact_actions[opcode as usize - 30] = Some(buf.get_rs_string());
            }
            39 => {
                buf.get_u8();
            }
            40 => {
                let len = buf.get_u8();
                let _ = (0..len)
                    .map(|_| (buf.get_u16(), buf.get_u16()))
                    .collect::<Vec<_>>();
            }
            60 | 65..=68 => {
                buf.get_u16();
            }
            62 => definition.rotated = true,
            64 => definition.casts_shadow = false,
            69 => {
                buf.get_u8();
            }
            70..=72 => {
                buf.get_u16();
            }
            73 => definition.obstructive = true,
            75 => {
                buf.get_u8();
            }
            77 => {
                buf.get_u16();
                buf.get_u16();
                let len = buf.get_u8();
                let _morphisms = (0..=len).map(|_| buf.get_u16()).collect::<Vec<u16>>();
            }
            _opcode => {
                // TODO: Discuss implementation of all fields with team: Many are not used by the server.
                // return Err(CacheError::DecodeDefinition {
                //     ty: "Object",
                //     opcode: _opcode,
                // })
                continue;
            }
        }
    }
}
