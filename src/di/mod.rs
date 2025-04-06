use crate::{
    app::{
        command::create_short_url::{CreateShortUrlCommand, CreateShortUrlRepository},
        query::get_full_url::{GetFullUrlQuery, GetFullUrlRepository},
    },
    id_provider::IdProvider,
};

pub struct Container<I, R, Q>
where
    I: IdProvider,
    R: CreateShortUrlRepository,
    Q: GetFullUrlRepository,
{
    pub short_url_command: CreateShortUrlCommand<I, R>,
    pub get_full_url_query: GetFullUrlQuery<Q>,
}

impl<I, R, Q> Container<I, R, Q>
where
    I: IdProvider,
    R: CreateShortUrlRepository,
    Q: GetFullUrlRepository,
{
    pub fn new(id_provider: I, repository: R, query: Q) -> Self {
        Self {
            short_url_command: CreateShortUrlCommand::new(id_provider, repository),
            get_full_url_query: GetFullUrlQuery::new(query),
        }
    }
}
