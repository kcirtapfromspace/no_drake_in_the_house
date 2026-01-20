//! LanceDB Vector Database Client
//!
//! Provides vector embedding storage and similarity search for:
//! - News article semantic search
//! - Artist description embeddings
//! - Entity context embeddings
//! - Similar article discovery

use anyhow::{Context, Result};
use arrow::error::ArrowError;
use arrow_array::builder::{ListBuilder, StringBuilder};
use arrow_array::types::Float32Type;
use arrow_array::{
    BooleanArray, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray,
};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use futures::TryStreamExt;
use lancedb::connect;
use lancedb::query::{ExecutableQuery, QueryBase, Select};
use std::sync::Arc;
use uuid::Uuid;

/// Vector embedding dimension (using common embedding models)
pub const EMBEDDING_DIM: usize = 768; // BERT/sentence-transformers default

const NEWS_TABLE: &str = "news_embeddings";
const ARTIST_TABLE: &str = "artist_embeddings";

/// LanceDB vector database client
pub struct LanceDbClient {
    db: lancedb::Connection,
    #[allow(dead_code)]
    db_path: String,
}

impl LanceDbClient {
    /// Create a new LanceDB client
    pub async fn new(db_path: &str) -> Result<Self> {
        let db = connect(db_path)
            .execute()
            .await
            .context("Failed to connect to LanceDB")?;

        Ok(Self {
            db,
            db_path: db_path.to_string(),
        })
    }

    /// Initialize the vector tables
    pub async fn initialize_schema(&self) -> Result<()> {
        // Tables will be created on first insert
        tracing::info!("LanceDB vector schema initialized (tables created on first insert)");
        Ok(())
    }

    /// Insert news article embedding
    pub async fn insert_news_embedding(&self, record: NewsEmbeddingRecord) -> Result<()> {
        ensure_embedding_dim(&record.embedding)?;
        let batch = news_record_batch(&record)?;
        insert_record(&self.db, NEWS_TABLE, batch).await?;
        Ok(())
    }

    /// Search for similar news articles
    pub async fn search_similar_news(
        &self,
        query_embedding: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<NewsSearchResult>> {
        ensure_embedding_dim(query_embedding)?;
        let table = match open_table_if_exists(&self.db, NEWS_TABLE).await? {
            Some(table) => table,
            None => return Ok(vec![]),
        };
        let mut query = table
            .query()
            .nearest_to(query_embedding)?
            .limit(limit)
            .select(Select::columns(&["id", "title", "_distance"]));
        if let Some(filter) = _filter {
            query = query.only_if(filter);
        }
        let stream = query.execute().await?;
        let batches: Vec<RecordBatch> = stream.try_collect().await?;
        extract_news_results(&batches)
    }

    /// Insert artist embedding
    pub async fn insert_artist_embedding(&self, record: ArtistEmbeddingRecord) -> Result<()> {
        ensure_embedding_dim(&record.embedding)?;
        let batch = artist_record_batch(&record)?;
        insert_record(&self.db, ARTIST_TABLE, batch).await?;
        Ok(())
    }

    /// Search for similar artists
    pub async fn search_similar_artists(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<ArtistSearchResult>> {
        ensure_embedding_dim(query_embedding)?;
        let table = match open_table_if_exists(&self.db, ARTIST_TABLE).await? {
            Some(table) => table,
            None => return Ok(vec![]),
        };
        let query = table
            .query()
            .nearest_to(query_embedding)?
            .limit(limit)
            .select(Select::columns(&["id", "canonical_name", "_distance"]));
        let stream = query.execute().await?;
        let batches: Vec<RecordBatch> = stream.try_collect().await?;
        extract_artist_results(&batches)
    }

    /// Delete embedding by ID
    pub async fn delete_news_embedding(&self, id: &Uuid) -> Result<()> {
        let table = match open_table_if_exists(&self.db, NEWS_TABLE).await? {
            Some(table) => table,
            None => return Ok(()),
        };
        table.delete(&format!("id = '{}'", id)).await?;
        Ok(())
    }

    /// Get table statistics
    pub async fn get_stats(&self) -> Result<VectorDbStats> {
        let tables = self.db.table_names().execute().await?;

        let news_embeddings_count = table_count(&self.db, NEWS_TABLE, &tables).await?;
        let artist_embeddings_count = table_count(&self.db, ARTIST_TABLE, &tables).await?;

        Ok(VectorDbStats {
            news_embeddings_count,
            artist_embeddings_count,
            tables,
        })
    }
}

fn ensure_embedding_dim(embedding: &[f32]) -> Result<()> {
    if embedding.len() != EMBEDDING_DIM {
        return Err(anyhow::anyhow!(
            "Embedding dimension mismatch: expected {}, got {}",
            EMBEDDING_DIM,
            embedding.len()
        ));
    }
    Ok(())
}

fn embedding_array(embedding: &[f32]) -> FixedSizeListArray {
    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        std::iter::once(Some(
            embedding
                .iter()
                .map(|value| Some(*value))
                .collect::<Vec<_>>(),
        )),
        EMBEDDING_DIM as i32,
    )
}

fn news_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            "embedding",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                EMBEDDING_DIM as i32,
            ),
            false,
        ),
        Field::new("title", DataType::Utf8, false),
        Field::new("content_hash", DataType::Utf8, false),
        Field::new("published_at", DataType::Utf8, true),
        Field::new("source_type", DataType::Utf8, true),
        Field::new("has_offense", DataType::Boolean, false),
    ]))
}

fn artist_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            "embedding",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                EMBEDDING_DIM as i32,
            ),
            false,
        ),
        Field::new("canonical_name", DataType::Utf8, false),
        Field::new("description", DataType::Utf8, true),
        Field::new(
            "genres",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            false,
        ),
    ]))
}

fn news_record_batch(record: &NewsEmbeddingRecord) -> Result<RecordBatch> {
    let schema = news_schema();
    let id = StringArray::from(vec![record.id.to_string()]);
    let embedding = embedding_array(&record.embedding);
    let title = StringArray::from(vec![record.title.clone()]);
    let content_hash = StringArray::from(vec![record.content_hash.clone()]);
    let published_at = StringArray::from(vec![record.published_at.clone()]);
    let source_type = StringArray::from(vec![record.source_type.clone()]);
    let has_offense = BooleanArray::from(vec![record.has_offense]);

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(id),
            Arc::new(embedding),
            Arc::new(title),
            Arc::new(content_hash),
            Arc::new(published_at),
            Arc::new(source_type),
            Arc::new(has_offense),
        ],
    )
    .context("Failed to build news embedding record batch")
}

fn artist_record_batch(record: &ArtistEmbeddingRecord) -> Result<RecordBatch> {
    let schema = artist_schema();
    let id = StringArray::from(vec![record.id.to_string()]);
    let embedding = embedding_array(&record.embedding);
    let canonical_name = StringArray::from(vec![record.canonical_name.clone()]);
    let description = StringArray::from(vec![record.description.clone()]);
    let genres = genres_array(&record.genres);

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(id),
            Arc::new(embedding),
            Arc::new(canonical_name),
            Arc::new(description),
            Arc::new(genres),
        ],
    )
    .context("Failed to build artist embedding record batch")
}

fn genres_array(genres: &[String]) -> arrow_array::ListArray {
    let mut builder = ListBuilder::new(StringBuilder::new());
    for genre in genres {
        builder.values().append_value(genre);
    }
    builder.append(true);
    builder.finish()
}

fn record_batch_reader(
    batch: RecordBatch,
) -> RecordBatchIterator<Vec<Result<RecordBatch, ArrowError>>> {
    let schema = batch.schema();
    RecordBatchIterator::new(vec![Ok(batch)], schema)
}

async fn open_table_if_exists(
    db: &lancedb::Connection,
    name: &str,
) -> Result<Option<lancedb::Table>> {
    match db.open_table(name).execute().await {
        Ok(table) => Ok(Some(table)),
        Err(err) => {
            if matches!(err, lancedb::Error::TableNotFound { .. }) {
                Ok(None)
            } else {
                Err(err.into())
            }
        }
    }
}

async fn insert_record(
    db: &lancedb::Connection,
    table_name: &str,
    batch: RecordBatch,
) -> Result<()> {
    match db.open_table(table_name).execute().await {
        Ok(table) => {
            let reader = record_batch_reader(batch);
            table.add(reader).execute().await?;
        }
        Err(err) => {
            if matches!(err, lancedb::Error::TableNotFound { .. }) {
                let reader = record_batch_reader(batch);
                db.create_table(table_name, reader).execute().await?;
            } else {
                return Err(err.into());
            }
        }
    }
    Ok(())
}

fn similarity_from_distance(distance: f32) -> f32 {
    1.0 / (1.0 + distance)
}

fn extract_news_results(batches: &[RecordBatch]) -> Result<Vec<NewsSearchResult>> {
    let mut results = Vec::new();
    for batch in batches {
        let id_array = batch
            .column_by_name("id")
            .context("Missing id column for news search")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Invalid id column type for news search")?;
        let title_array = batch
            .column_by_name("title")
            .context("Missing title column for news search")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Invalid title column type for news search")?;
        let distance_array = batch
            .column_by_name("_distance")
            .context("Missing _distance column for news search")?
            .as_any()
            .downcast_ref::<Float32Array>()
            .context("Invalid _distance column type for news search")?;

        for row in 0..batch.num_rows() {
            if id_array.is_null(row) || title_array.is_null(row) {
                continue;
            }
            let id = Uuid::parse_str(id_array.value(row))
                .context("Invalid UUID stored in news embeddings")?;
            let title = title_array.value(row).to_string();
            let distance = distance_array.value(row);
            results.push(NewsSearchResult {
                id,
                title,
                distance,
                similarity: similarity_from_distance(distance),
            });
        }
    }
    Ok(results)
}

fn extract_artist_results(batches: &[RecordBatch]) -> Result<Vec<ArtistSearchResult>> {
    let mut results = Vec::new();
    for batch in batches {
        let id_array = batch
            .column_by_name("id")
            .context("Missing id column for artist search")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Invalid id column type for artist search")?;
        let name_array = batch
            .column_by_name("canonical_name")
            .context("Missing canonical_name column for artist search")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Invalid canonical_name column type for artist search")?;
        let distance_array = batch
            .column_by_name("_distance")
            .context("Missing _distance column for artist search")?
            .as_any()
            .downcast_ref::<Float32Array>()
            .context("Invalid _distance column type for artist search")?;

        for row in 0..batch.num_rows() {
            if id_array.is_null(row) || name_array.is_null(row) {
                continue;
            }
            let id = Uuid::parse_str(id_array.value(row))
                .context("Invalid UUID stored in artist embeddings")?;
            let name = name_array.value(row).to_string();
            let distance = distance_array.value(row);
            results.push(ArtistSearchResult {
                id,
                name,
                distance,
                similarity: similarity_from_distance(distance),
            });
        }
    }
    Ok(results)
}

async fn table_count(
    db: &lancedb::Connection,
    table_name: &str,
    tables: &[String],
) -> Result<u64> {
    if !tables.iter().any(|name| name == table_name) {
        return Ok(0);
    }
    let table = match open_table_if_exists(db, table_name).await? {
        Some(table) => table,
        None => return Ok(0),
    };
    Ok(table.count_rows(None).await? as u64)
}

/// News embedding record for storage
#[derive(Debug, Clone)]
pub struct NewsEmbeddingRecord {
    pub id: Uuid,
    pub embedding: Vec<f32>,
    pub title: String,
    pub content_hash: String,
    pub published_at: Option<String>,
    pub source_type: Option<String>,
    pub has_offense: bool,
}

/// Artist embedding record for storage
#[derive(Debug, Clone)]
pub struct ArtistEmbeddingRecord {
    pub id: Uuid,
    pub embedding: Vec<f32>,
    pub canonical_name: String,
    pub description: Option<String>,
    pub genres: Vec<String>,
}

/// News search result
#[derive(Debug, Clone)]
pub struct NewsSearchResult {
    pub id: Uuid,
    pub title: String,
    pub distance: f32,
    pub similarity: f32,
}

/// Artist search result
#[derive(Debug, Clone)]
pub struct ArtistSearchResult {
    pub id: Uuid,
    pub name: String,
    pub distance: f32,
    pub similarity: f32,
}

/// Vector database statistics
#[derive(Debug, Clone)]
pub struct VectorDbStats {
    pub news_embeddings_count: u64,
    pub artist_embeddings_count: u64,
    pub tables: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_lancedb_initialization() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_lance");
        let client = LanceDbClient::new(db_path.to_str().unwrap()).await.unwrap();
        client.initialize_schema().await.unwrap();
    }

    #[tokio::test]
    async fn test_news_embedding_lifecycle() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_news");
        let client = LanceDbClient::new(db_path.to_str().unwrap()).await.unwrap();

        let id = Uuid::new_v4();
        let embedding = vec![0.0; EMBEDDING_DIM];
        let record = NewsEmbeddingRecord {
            id,
            embedding: embedding.clone(),
            title: "Test news".to_string(),
            content_hash: "hash".to_string(),
            published_at: None,
            source_type: Some("rss".to_string()),
            has_offense: false,
        };
        client.insert_news_embedding(record).await.unwrap();

        let results = client
            .search_similar_news(&embedding, 5, None)
            .await
            .unwrap();
        assert!(results.iter().any(|result| result.id == id));

        client.delete_news_embedding(&id).await.unwrap();
        let results_after = client
            .search_similar_news(&embedding, 5, None)
            .await
            .unwrap();
        assert!(results_after.is_empty());
    }

    #[tokio::test]
    async fn test_artist_embedding_search() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_artist");
        let client = LanceDbClient::new(db_path.to_str().unwrap()).await.unwrap();

        let id = Uuid::new_v4();
        let embedding = vec![1.0; EMBEDDING_DIM];
        let record = ArtistEmbeddingRecord {
            id,
            embedding: embedding.clone(),
            canonical_name: "Test Artist".to_string(),
            description: Some("Test description".to_string()),
            genres: vec!["hip hop".to_string()],
        };
        client.insert_artist_embedding(record).await.unwrap();

        let results = client
            .search_similar_artists(&embedding, 5)
            .await
            .unwrap();
        assert!(results.iter().any(|result| result.id == id));
    }

    #[tokio::test]
    async fn test_embedding_dimension_mismatch() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_dim");
        let client = LanceDbClient::new(db_path.to_str().unwrap()).await.unwrap();

        let record = NewsEmbeddingRecord {
            id: Uuid::new_v4(),
            embedding: vec![0.0; EMBEDDING_DIM - 1],
            title: "Bad news".to_string(),
            content_hash: "hash".to_string(),
            published_at: None,
            source_type: None,
            has_offense: false,
        };
        let err = client.insert_news_embedding(record).await.unwrap_err();
        assert!(err
            .to_string()
            .contains("Embedding dimension mismatch"));

        let err = client
            .search_similar_news(&vec![0.0; EMBEDDING_DIM + 1], 5, None)
            .await
            .unwrap_err();
        assert!(err
            .to_string()
            .contains("Embedding dimension mismatch"));
    }
}
