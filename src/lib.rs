#![no_std]

use aidoku::{
    alloc::{format, string::String, vec, vec::Vec},
    imports::net::Request,
    prelude::*,
    Chapter, ContentRating, FilterValue, Listing, ListingProvider, Manga,
    MangaPageResult, MangaStatus, Page, PageContent, Result, Source,
};

const BASE_URL: &str = "https://readcomiconline.li";

struct RcoSource;

impl Source for RcoSource {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        _query: Option<String>,
        _page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        Ok(MangaPageResult {
            entries: vec![],
            has_next_page: false,
        })
    }

    fn get_manga_update(
        &self,
        manga: Manga,
        _needs_details: bool,
        _needs_chapters: bool,
    ) -> Result<Manga> {
        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, _chapter: Chapter) -> Result<Vec<Page>> {
        Ok(vec![Page {
            content: PageContent::url("https://aidoku.app/images/icon.png"),
            ..Default::default()
        }])
    }
}

impl ListingProvider for RcoSource {
    fn get_manga_list(&self, _listing: Listing, _page: i32) -> Result<MangaPageResult> {
        Ok(MangaPageResult {
            entries: vec![],
            has_next_page: false,
        })
    }
}

register_source!(RcoSource, ListingProvider);
