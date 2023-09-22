//! defmt-printer based api for embedding a defmt printer in your project
//!
//! This library was created to make it simple to add defmt printer functionality to your project.
//! 
//! There are 2 main advantages for using this crate:
//! - This crate is that it handles most of the data needed for using a decoder and provides an easy to use helper.
//! - This crate can save you some time because if you can't use existing tooling then you probably would have written something similar to this crate.
//! 
//! ## Notice!!! 
//! This crate is based on the defmt_decoder unstable api use at your own risk
//! 
//! ##How to use:
//! 
//!     let mut helper = DefmtPrintHelper::new(new(elf_path)?;
//!     ...
//!     let frame: &[u8] = ....
//!     helper.handle_frame(frame)?;
//! 
//! See the printer example for compering the original defmt-printer to a rewritten printer using this api

#![cfg(feature = "unstable")]
use std::{collections::BTreeMap, env, fs, path::PathBuf};
use anyhow::Context;
use defmt_decoder::{Frame, Location, StreamDecoder, Table, DecodeError};

#[derive(Debug)]
struct LocationInfo {
    pub file: Option<String>, 
    pub line: Option<u32>, 
    pub mod_path: Option<String>
}

impl Default for LocationInfo {
    fn default() -> LocationInfo {
        LocationInfo {
            file: None,
            line: None,
            mod_path: None
        }
    }
}

struct HelperLocData {
    locs: Option<BTreeMap<u64, Location>>,
    current_dir: PathBuf
}

impl HelperLocData {
    pub fn new(locs: Option<BTreeMap<u64, Location>>, current_dir: PathBuf) -> Self {
        HelperLocData {
            locs: locs,
            current_dir: current_dir
        }
    }

    pub fn frame_location_info(&self, frame: &Frame) -> LocationInfo {
        let mut loc_info = LocationInfo::default();

        let loc = self.locs.as_ref().map(|locs| locs.get(&frame.index()));

        if let Some(Some(loc)) = loc {
            // try to get the relative path, else the full one
            let path = loc
                .file
                .strip_prefix(self.current_dir.clone())
                .unwrap_or(&loc.file);

            loc_info.file = Some(path.display().to_string());
            loc_info.line = Some(loc.line as u32);
            loc_info.mod_path = Some(loc.module.clone());
        }
        loc_info
    }
}

pub struct DefmtPrintHelper {
    loc_data: HelperLocData,
    table: Table,
    decoder: Box<dyn StreamDecoder>
}

impl DefmtPrintHelper {
    pub fn new(elf_path: PathBuf) -> Result<Self, anyhow::Error> {
        let bytes = fs::read(elf_path)?;
        let mut table = Table::parse(&bytes)?.context("elf is missing a defmt section")?;
        let locs = table.get_locations(&bytes)?;
        let locs = match table.indices().all(|idx| locs.contains_key(&(idx as u64))) {
            true => Some(locs),
            false => {
                log::warn!("(BUG) location info is incomplete; it will be omitted from the output");
                None
            }
        };
        let t_table: *mut Table = &mut table;
        let t_decoder =  unsafe {(*t_table).new_stream_decoder()}; // self referential struct members with lifetimes are painful
        Ok(
            DefmtPrintHelper {
                loc_data: HelperLocData::new(locs, env::current_dir()?),
                table: table,
                decoder: t_decoder
            }
        )
    }

    fn forward_to_logger(frame: &Frame, location_info: LocationInfo) {
        defmt_decoder::log::log_defmt(frame, location_info.file.as_deref(), location_info.line, location_info.mod_path.as_deref());
    }

    pub fn handle_frame(&mut self, frame: &[u8]) -> Result<(), DecodeError> {
        self.decoder.received(frame);
        let log_frame = self.decoder.decode()?;
        Self::forward_to_logger(&log_frame, self.loc_data.frame_location_info(&log_frame));
        Ok(())
    }

    // exposing the table to allow the user to check the table state
    pub fn table(&self) -> &Table {
        &self.table
    }
}