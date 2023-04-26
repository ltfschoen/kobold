// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;

use logos::{Lexer, Logos};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{File, Url};
use gloo_file::{Blob};
use serde_json::{to_string};
use log::{debug};

use crate::state::{Content, Table, Text, TextSource};

#[derive(Logos)]
enum Token {
    #[error]
    Err,
    #[token(",")]
    Comma,
    #[regex(r"[\n\r]+")]
    Newline,
    #[regex(r#"[^"\n\r,]+"#)]
    Value,
    #[regex(r#""[^"]+""#, priority = 6)]
    QuotedValue,
    #[regex(r#""([^"]|"")+""#)]
    EscapedValue,
}

#[derive(Debug)]
pub enum Error {
    NoData,
    FailedToBufferFile,
    FailedToCreateObjectBlobWithUrl,
    FailedToReadFile,
    FailedToWriteFile,
    FailedToLoadMetadata,
    InvalidRowLength,
}

fn parse_row(lex: &mut Lexer<Token>, columns: usize) -> Result<Option<Vec<Text>>, Error> {
    let mut row = Vec::with_capacity(columns);
    let mut value = None;

    while let Some(token) = lex.next() {
        value = match token {
            Token::Value => Some(Text::Insitu(lex.span())),
            Token::QuotedValue => {
                let mut span = lex.span();

                span.start += 1;
                span.end -= 1;

                Some(Text::Insitu(span))
            }
            Token::EscapedValue => {
                let mut slice = lex.slice();

                slice = &slice[1..slice.len() - 1];

                Some(Text::Owned(slice.replace("\"\"", "\"").into()))
            }
            Token::Comma => {
                row.push(value.take().unwrap_or_default());
                continue;
            }
            Token::Newline => {
                row.push(value.take().unwrap_or_default());
                break;
            }
            Token::Err => break,
        }
    }

    if let Some(value) = value {
        row.push(value);
    }

    match (columns, row.len()) {
        (_, 0) => Ok(None),
        (0, _) => Ok(Some(row)),
        (n, r) => {
            if n > r {
                row.resize_with(n, Text::default);
            }

            if r > n {
                Err(Error::InvalidRowLength)
            } else {
                Ok(Some(row))
            }
        }
    }
}

impl TryFrom<String> for Table {
    type Error = Error;

    fn try_from(source: String) -> Result<Self, Error> {
        let mut lex = Token::lexer(&source);

        let columns = parse_row(&mut lex, 0)?.ok_or(Error::NoData)?;

        let mut rows = Vec::new();

        while let Some(row) = parse_row(&mut lex, columns.len())? {
            rows.push(row);
        }

        Ok(Table {
            source: TextSource::from(source),
            columns,
            rows,
        })
    }
}

impl FromStr for Table {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        s.to_owned().try_into()
    }
}

pub async fn read_file(file: File) -> Result<Table, Error> {
    let text = JsFuture::from(file.text())
        .await
        .map_err(|_| Error::FailedToReadFile)?
        .as_string()
        .ok_or(Error::FailedToReadFile)?;

    text.parse()
}

pub async fn write_file(content: &Content) -> Result<String, Error> {
    let content_serialized: String = to_string(&content.table).unwrap_throw();
    // cast String into a byte slice
    let content_serialized_byte_slice: &[u8] = &content_serialized.as_bytes();

    let file_blob: Blob = Blob::new_with_options(
        content_serialized_byte_slice,
        Some("text/plain"),
    );
    debug!("file_blob: {:?}", file_blob);
    // convert struct `gloo_file::Blob` into struct `web_sys::Blob`
    let obj_url = match Url::create_object_url_with_blob(&file_blob.into()) {
        Ok(url) => url,
        Err(err) => return Err(Error::FailedToCreateObjectBlobWithUrl),
    };

    return Ok(obj_url);
}
