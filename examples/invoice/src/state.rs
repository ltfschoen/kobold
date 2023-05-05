// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use gloo_storage::{LocalStorage, Storage};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::ops::{Deref, DerefMut, Range};
use wasm_bindgen::UnwrapThrowExt;

const KEY_MAIN: &str = "kobold.invoice.main";
const KEY_DETAILS: &str = "kobold.invoice.details";

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum TableVariant {
    Main,
    Details,
    Unknown,
}

#[derive(Deserialize, Debug)]
pub enum Error {
    StorageError,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Editing {
    None,
    Column { col: usize },
    Cell { col: usize, row: usize },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub filename: String,
    pub table: Table,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub editing_main: Editing,
    pub editing_details: Editing,
    pub main: Content,
    pub details: Content,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    pub variant: TableVariant,
    pub source: TextSource,
    pub columns: Vec<Text>,
    pub rows: Vec<Vec<Text>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Text {
    Insitu(Range<usize>),
    Owned(Box<str>),
}

impl Default for Text {
    fn default() -> Self {
        Text::Insitu(0..0)
    }
}

impl Default for State {
    fn default() -> Self {
        let main_local_storage: Table = match LocalStorage::get(KEY_MAIN) {
            Ok(local_storage) => local_storage,
            Err(err) => Table::mock_file_main(),
        };
        let details_local_storage: Table = match LocalStorage::get(KEY_DETAILS) {
            Ok(local_storage) => local_storage,
            // TODO - check that this actually converts to Table type
            Err(err) => Table::mock_file_details(),
        };
        debug!(
            "loading local storage: {:?}\n\n{:?}",
            main_local_storage, details_local_storage
        );

        State {
            editing_main: Editing::None,
            editing_details: Editing::None,
            main: Content {
                filename: "main.csv".to_owned(),
                table: main_local_storage,
            },
            details: Content {
                filename: "details.csv".to_owned(),
                table: details_local_storage,
            },
        }
    }
}

impl State {
    pub fn mock() -> Self {
        State {
            editing_main: Editing::None,
            editing_details: Editing::None,
            main: Content {
                filename: "main.csv".to_owned(),
                table: Table::mock_file_main(),
            },
            details: Content {
                filename: "details.csv".to_owned(),
                table: Table::mock_file_details(),
            },
        }
    }

    #[inline(never)]
    // store the updated state in web browser local storage
    pub fn store(&self) {
        debug!(
            "updating store: {:?}\n\n{:?}",
            &self.main.table, &self.details.table
        );
        LocalStorage::set(KEY_MAIN, &self.main.table).unwrap_throw();
        LocalStorage::set(KEY_DETAILS, &self.details.table).unwrap_throw();
    }

    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=53e5b5c0c241be2f5b37815a685e7da6
    pub fn remove_row_main(&mut self, row_idx_remove: usize) {
        let binding_source: &str = &self.main.table.source.source;
        let mut rows_vec: Vec<&str> = binding_source.split('\n').collect();
        debug!("rows_vec {:?}", rows_vec);
        let rows_start_idx = 1; // after the label row
        let mut rows_vec2 = rows_vec.split_off(rows_start_idx);
        debug!("rows_vec2 {:?}: ", rows_vec2);
        rows_vec2.remove(row_idx_remove);
        rows_vec.append(&mut rows_vec2);
        // label row + remaining rows after removing a row
        debug!("rows_vec {:?}: ", rows_vec);
        let rows_str: String = rows_vec.join("\n");
        println!("rows_str {:?}", rows_str);

        let rows_str_textsource = TextSource { source: rows_str };
        self.main.table.source = rows_str_textsource;
        debug!("self.main.table.source {:?}", self.main.table.source);

        let mut last_insitu_range_end: usize = 0;
        if row_idx_remove == 0 {
            // TODO - move into common function and use `if let Some(` instead of `unwrap`
            // we need get end of last col range in columns, since old row0 removed and replaced with old row1
            // that will now need to start from that (last col range + 1)
            last_insitu_range_end = self
                .main
                .table
                .columns
                .iter()
                .rev()
                .inspect(|x| println!("processing: {:?}", x))
                .find_map(|x| match x {
                    Text::Insitu(span) => Some(span.end),
                    Text::Owned(string) => None, // keep looking for last Insitu in column
                    _ => None,                   // keep looking for last Insitu in column
                })
                .unwrap();
        // repeat for if user removes the 2nd row, and the 3rd row, etc
        } else if row_idx_remove >= 1 {
            // this row changed
            last_insitu_range_end = self.main.table.rows[row_idx_remove - 1]
                .iter()
                .rev()
                .inspect(|x| println!("processing: {:?}", x))
                .find_map(|x| match x {
                    Text::Insitu(span) => Some(span.end),
                    Text::Owned(string) => None, // keep looking for last Insitu in column
                    _ => None,                   // keep looking for last Insitu in column
                })
                .unwrap();
        }

        println!("last_insitu_range_end: {:?}", last_insitu_range_end);
        let first_insitu_range_start = last_insitu_range_end + 1;

        // let's assume `last_insitu_range_end` is `20`, then
        // then go all `rows` associated with rows in `rows_vec2`, which is the remaining rows after removing the specific row
        // and reduce all the values so the first Insitu starts from (`last_insitu_range_end` + 1), i.e. 21, and
        // all other Insitu elements have their range values reduced to start from that, so if next one is 37..47,
        // and next after that was 49..54,
        // the first one would still be 10 usize long, but change to 21..31, and the second would still be 5 usize
        // long but reduce by 16 like the difference of the other one, so change to 49-16=33 and 54-16=38, so would become 33-38,
        // and any Owned values would remain unchanged.
        &self.main.table.rows.remove(row_idx_remove); // remove from `rows`
        let mut new_rows: Vec<Vec<_>> = vec![];
        let mut new_row: Vec<_> = vec![];
        let mut current_diff = 0usize;
        let mut current_insitu_end = 0usize;
        let mut next_insitu_start = first_insitu_range_start;
        println!("self.main.table.rows {:?}", self.main.table.rows);
        for (i, row) in self.main.table.rows.iter_mut().enumerate() {
            // TODO - try to remove the use of `.clone()`

            // keep the indexes from rows before the row that was removed, since later
            // rows were moved back one index and only those need to be changed
            if i < row_idx_remove {
                new_rows.push(row.clone()); // push the whole row
                continue;
            }

            for (j, cell) in row.clone().iter_mut().enumerate() {
                match cell {
                    Text::Insitu(span) => {
                        current_diff = span.end - span.start;
                        current_insitu_end = next_insitu_start + current_diff;
                        new_row.push(Text::Insitu(Range {
                            start: next_insitu_start,
                            end: current_insitu_end,
                        }));
                        next_insitu_start = current_insitu_end + 1; //first_insitu_range_start + current_diff;
                    }
                    Text::Owned(string) => {
                        new_row.push(Text::Owned((*string.clone()).into()));
                    } // no change
                    _ => panic!("unexpected element"),
                }

                if j == row.clone().len() - 1 {
                    new_rows.push(new_row.clone());
                    new_row.clear(); // empty read for next `row`
                }
            }
        }
        println!("new_rows: {:?}", new_rows);

        self.store();
    }
}

impl Deref for Content {
    type Target = Table;

    fn deref(&self) -> &Table {
        &self.table
    }
}

impl DerefMut for Content {
    fn deref_mut(&mut self) -> &mut Table {
        &mut self.table
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextSource {
    pub source: String,
}

impl From<String> for TextSource {
    fn from(source: String) -> Self {
        TextSource { source }
    }
}

impl TextSource {
    pub fn get_text<'a>(&'a self, text: &'a Text) -> &'a str {
        debug!("get_text source {:?}", self.source);
        match text {
            Text::Insitu(span) => &self.source[span.clone()],
            Text::Owned(string) => string,
        }
    }
}

impl Table {
    fn mock_file_main() -> Self {
        "#main,description,total,qr\ntask1,10,0x000|h160\ntask2,20,0x100|h160"
            .parse()
            .expect_throw("unable to parse mock file main")
    }

    // `#details,` is not a column, it is only to identify the table variant. if it was this value it would be stored
    // in `Table`'s `variant` property as `TableVariant::Details` if that was the configured mapping supported.
    // it is removed from the source during the upload process using `parse_table_variant` in csv.rs.
    // if it is not specified then a value of `TableVariant::Unknown` is assigned.
    fn mock_file_details() -> Self {
        "#details,invoice date,invoice number,name person from,organisation name from,organisation address from,email from,name person attention to,title to,organisation name to,email to\n01.04.2023,0001,luke,clawbird,1 metaverse ave,test@test.com,recipient_name,director,nftverse,test2@test.com"
            .parse()
            .expect_throw("unable to parse mock file details")
    }

    pub fn rows(&self) -> Range<usize> {
        0..self.rows.len()
    }

    pub fn columns(&self) -> Range<usize> {
        0..self.columns.len()
    }
}
