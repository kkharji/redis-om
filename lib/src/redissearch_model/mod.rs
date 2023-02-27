/// Redis Search Model used only in migration
pub trait RedisSearchModel {
    /// full redis search schema
    const _REDIS_SEARCH_SCHEMA: &'static str;
}
