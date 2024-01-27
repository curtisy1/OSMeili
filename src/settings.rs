use crate::filter::Group;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Settings {
    /// the source of the .pbf extract.
    /// Can be either a local file or uri (e.g. GeoFabrik)
    #[clap(short, long, env, required = true)]
    pub source: String,
    /// the API key for your meilisearch instance.
    #[clap(long, env, required = true)]
    pub meili_key: String,
    /// the host where your meili instance is running on.
    #[clap(long, env, default_value = "http://localhost:7700")]
    pub meili_uri: String,
    /// the index name to use for storing nodes in meilisearch.
    #[clap(long, env, default_value = "addresses")]
    pub meili_node_index_name: String,
    /// a list of keys that meilisearch uses for search.
    /// Use in combination with tags
    #[clap(long, env, default_value = "street,houseNumber,postcode,city,country")]
    pub meili_node_searchable_values: Vec<String>,
    /// specify tags to filter on.
    #[clap(short, long, env, value_parser = clap::value_parser!(Group), default_value = "addr")]
    pub tags: Vec<Group>,
    /// the source to use for replication.
    /// Like source, it can be set to either local or remote extracts
    #[clap(long, env)]
    pub replication_source: Option<String>,
    /// how often data should be updated.
    #[clap(long, env)]
    pub replication_interval_minutes: Option<usize>,
    /// how many elements will be inserted into meilisearch per request.
    #[clap(long, env, default_value = "1000")]
    pub import_chunk_size: usize,
    /// how many import requests should run in parallel.
    #[clap(long, env, default_value = "10")]
    pub import_parallel_requests: usize,
    /// the log level
    #[clap(short, long, env, default_value = "I")]
    pub log_level: String,
}
