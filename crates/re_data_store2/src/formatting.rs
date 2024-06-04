use crate::DataStore2;

// ---

impl std::fmt::Display for DataStore2 {
    #[allow(clippy::string_add)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            id,
            config,
            // TODO
            // type_registry: _,
            // metadata_registry: _,
            chunks_per_chunk_id,
            chunks_per_min_row_id,
            temporal_chunk_ids_per_entity: _,
            static_chunks_per_entity: _,
            insert_id: _,
            query_id: _,
            gc_id: _,
            event_id: _,
        } = self;

        f.write_str("DataStore2 {\n")?;

        f.write_str(&indent::indent_all_by(4, format!("id: {id}\n")))?;
        f.write_str(&indent::indent_all_by(4, format!("config: {config:?}\n")))?;

        f.write_str(&indent::indent_all_by(4, "chunks: [\n"))?;
        for chunk_ids in chunks_per_min_row_id.values() {
            for chunk_id in chunk_ids {
                if let Some(chunk) = chunks_per_chunk_id.get(chunk_id) {
                    f.write_str(&indent::indent_all_by(8, format!("{chunk}\n")))?;
                } else {
                    f.write_str(&indent::indent_all_by(8, "<not_found>\n"))?;
                }
            }
        }
        f.write_str(&indent::indent_all_by(4, "]\n"))?;

        f.write_str("}")?;

        Ok(())
    }
}
