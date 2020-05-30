use std::io::{prelude::*, Cursor, SeekFrom};

use bytes::Buf;

use mithril_buf::GameBuf;

use crate::CacheFileSystem;

pub enum EntityAnimation {
    Idle,
    Walk,
    TurnAway,
    TurnLeft,
    TurnRight,
}

#[derive(Debug)]
pub struct EntityDefinition {
    id: u16,
    combat_level: Option<u16>,
    name: String,
    examine_text: String,
    interact_actions: [Option<String>; 5],
    size: u8,
    anim_stand: Option<u16>,
    anim_walk: [Option<u16>; 4],
    visible_on_minimap: bool,
    clickable: bool,
    visible: bool,
}

impl Default for EntityDefinition {
    fn default() -> Self {
        Self {
            id: 0,
            combat_level: None,
            name: String::default(),
            examine_text: String::default(),
            interact_actions: [None, None, None, None, None],
            size: 1,
            anim_stand: None,
            anim_walk: [None, None, None, None],
            visible_on_minimap: true,
            clickable: false,
            visible: false,
        }
    }
}

impl EntityDefinition {
    pub fn load(cache: &mut CacheFileSystem) -> anyhow::Result<Vec<Self>> {
        let archive = cache.get_archive(0, 2)?;
        let mut index = archive
            .get_entry("npc.idx")
            .map(|entry| Cursor::new(entry.contents()))
            .expect("Failed to read obj.idx");

        let mut data = archive
            .get_entry("npc.dat")
            .map(|entry| Cursor::new(entry.contents()))
            .expect("Failed to read obj.dat");

        let count = index.get_u16() as usize;
        let mut offsets = vec![0; count];
        let mut position = 2;

        for offset in offsets.iter_mut() {
            *offset = position;
            position += index.get_u16() as u64;
        }

        let mut definitions: Vec<Self> = Vec::with_capacity(count);
        for (id, offset) in offsets.iter().enumerate() {
            data.seek(SeekFrom::Start(*offset))?;
            let definition = decode_definition(id as u16, &mut data);
            definitions.push(definition);
        }
        Ok(definitions)
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn combat_level(&self) -> Option<u16> {
        self.combat_level
    }

    pub fn examine(&self) -> &String {
        &self.examine_text
    }

    pub fn size(&self) -> u8 {
        self.size
    }

    pub fn animation(&self, animation_type: EntityAnimation) -> Option<u16> {
        match animation_type {
            EntityAnimation::Idle => self.anim_stand,
            EntityAnimation::Walk => self.anim_walk[0],
            EntityAnimation::TurnAway => self.anim_walk[1],
            EntityAnimation::TurnLeft => self.anim_walk[2],
            EntityAnimation::TurnRight => self.anim_walk[3],
        }
    }

    pub fn is_clickable(&self) -> bool {
        self.clickable
    }

    pub fn is_visible_on_minimap(&self) -> bool {
        self.visible_on_minimap
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

fn decode_definition<B: GameBuf>(npc_id: u16, buf: &mut B) -> EntityDefinition {
    let mut definition = EntityDefinition::default();
    definition.id = npc_id;
    loop {
        match buf.get_u8() {
            0 => return definition,
            1 => {
                let len = buf.get_u8();
                let _models = (0..len).map(|_| buf.get_u16()).collect::<Vec<_>>();
            }
            2 => definition.name = buf.get_rs_string(),
            3 => definition.examine_text = buf.get_rs_string(),
            12 => definition.size = buf.get_u8(),
            13 => {
                definition.anim_stand = Some(buf.get_u16());
            }
            14 => {
                definition.anim_walk[0] = Some(buf.get_u16());
            }
            17 => {
                definition.anim_walk[0] = Some(buf.get_u16());
                definition.anim_walk[1] = Some(buf.get_u16());
                definition.anim_walk[2] = Some(buf.get_u16());
                definition.anim_walk[3] = Some(buf.get_u16());
            }
            opcode if (30..40).contains(&opcode) => {
                let action = buf.get_rs_string();
                definition.interact_actions[opcode as usize - 30] = Some(action);
            }
            40 => {
                // replacement colours
                let len = buf.get_u8();
                let _ = (0..len)
                    .map(|_| (buf.get_u16(), buf.get_u16()))
                    .collect::<Vec<_>>();
            }
            60 => {
                let len = buf.get_u8();
                let _additional_models = (0..len).map(|_| buf.get_u16()).collect::<Vec<u16>>();
            }
            90..=92 => {
                buf.get_u16();
            }
            93 => {
                definition.visible_on_minimap = false;
            }
            95 => {
                definition.combat_level = match buf.get_u16() {
                    0 => None,
                    level => Some(level),
                }
            }
            97 | 98 => {
                buf.get_u16();
            }
            99 => {
                definition.visible = true;
            }
            100 | 101 => {
                buf.get_u8();
            }
            102 | 103 => {
                buf.get_u16();
            }
            106 => {
                buf.get_u16();
                buf.get_u16();
                let len = buf.get_u8();
                let _ = (0..len + 1).map(|_| buf.get_u16()).collect::<Vec<u16>>();
            }
            107 => definition.clickable = false,
            opcode => unimplemented!("opcode: {}", opcode),
        }
    }
}
