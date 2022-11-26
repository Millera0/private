use cang_jie::{CangJieTokenizer, TokenizerOption, CANG_JIE};
use jieba_rs::Jieba;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError, DefaultOnNull};
use std::sync::Arc;
use tantivy::{collector::TopDocs, query::QueryParser, schema::*, Index, store::Compressor};

#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Book {
    pub id: u64,

    pub title: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub author: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub publisher: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub extension: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub filesize: u64,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub language: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub year: u64,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub pages: u64,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub isbn: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub ipfs_cid: String,
}

impl From<(&Schema, Document)> for Book {
    fn from((schema, doc): (&Schema, Document)) -> Self {
        macro_rules! get_field_text {
            ($field:expr) => {
                doc.get_first(schema.get_field($field).unwrap())
                    .unwrap()
                    .as_text()
                    .unwrap_or_default()
                    .to_owned()
            };
        }

        macro_rules! get_field_u64 {
            ($field:expr) => {
                doc.get_first(schema.get_field($field).unwrap())
                    .unwrap()
                    .as_u64()
                    .unwrap_or_default()
            };
        }

        Book {
            id: get_field_u64!("id"),
            title: get_field_text!("title"),
            author: get_field_text!("author"),
            publisher: get_field_text!("publisher"),
            extension: get_field_text!("extension"),
            filesize: get_field_u64!("filesize"),
            language: get_field_text!("language"),
            year: get_field_u64!("year"),
            pages: get_field_u64!("pages"),
            isbn: get_field_text!("isbn"),
            ipfs_cid: get_field_text!("ipfs_cid"),
        }
    }
}

pub struct Searcher {
    index: Index,
    schema: Schema,

    // fields
    id: Field,
    title: Field,
    author: Field,
    publisher: Field,
    extension: Field,
    filesize: Field,
    language: Field,
    year: Field,
    pages: Field,
    isbn: Field,
    ipfs_cid: Field,
}

impl Searcher {
    pub fn new(index_dir: &str) -> Self {
        let mut index = Index::open_in_dir(index_dir).unwrap();
        #[cfg(feature = "best-size")]
        {
            index.settings_mut().docstore_compression = Compressor::Brotli; // size: 2.1G, size is best
        }
        #[cfg(feature = "best-speed")]
        {
            index.settings_mut().docstore_compression = Compressor::Lz4; // size: 3.1G, speed is best
        }
        
        let tokenizer = CangJieTokenizer {
            worker: Arc::new(Jieba::new()),
            option: TokenizerOption::Unicode,
        };
        index.tokenizers().register(CANG_JIE, tokenizer);
        _ = index.set_default_multithread_executor();

        let text_indexing = TextFieldIndexing::default()
            .set_tokenizer(CANG_JIE)
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let text_options = TextOptions::default()
            .set_indexing_options(text_indexing)
            .set_stored();

        let mut schema_builder = Schema::builder();
        let id = schema_builder.add_u64_field("id", INDEXED | STORED);
        let title = schema_builder.add_text_field("title", text_options.clone());
        let author = schema_builder.add_text_field("author", text_options.clone());
        let publisher = schema_builder.add_text_field("publisher", text_options.clone());
        let extension = schema_builder.add_text_field("extension", STRING | STORED);
        let filesize = schema_builder.add_u64_field("filesize", STORED);
        let language = schema_builder.add_text_field("language", TEXT | STORED);
        let year = schema_builder.add_u64_field("year", STORED);
        let pages = schema_builder.add_u64_field("pages", STORED);
        let isbn = schema_builder.add_text_field("isbn", TEXT | STORED);
        let ipfs_cid = schema_builder.add_text_field("ipfs_cid", STORED);
        let schema = schema_builder.build();

        Self {
            index,
            schema,
            id,
            title,
            author,
            publisher,
            extension,
            filesize,
            language,
            year,
            pages,
            isbn,
            ipfs_cid,
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<Book> {
        let reader = self.index.reader().unwrap();
        let searcher = reader.searcher();

        let mut query_parser = QueryParser::for_index(
            &self.index,
            vec![
                self.title.clone(),
                self.author.clone(),
                self.publisher.clone(),
                self.isbn.clone(),
            ],
        );
        query_parser.set_conjunction_by_default();
        let query = query_parser.parse_query(query).unwrap();

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .unwrap();

        top_docs
            .iter()
            .map(|d| {
                let doc = searcher.doc(d.1).unwrap();
                let item: Book = (&self.schema, doc).into();
                item
            })
            .collect()
    }
}
