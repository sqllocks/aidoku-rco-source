#![no_std]

use aidoku::{
    alloc::{format, string::String, vec, vec::Vec},
    imports::net::Request,
    prelude::*,
    Chapter, FilterValue, Listing, ListingProvider, Manga,
    MangaPageResult, Page, PageContent, Result, Source,
};

const BASE_URL: &str = "https://readcomiconline.li";

struct RcoSource;

impl Source for RcoSource {
    fn new() -> Self { Self }

    fn get_search_manga_list(
        &self,
        _query: Option<String>,
        _page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        Ok(MangaPageResult { entries: vec![], has_next_page: false })
    }

    fn get_manga_update(&self, manga: Manga, _nd: bool, _nc: bool) -> Result<Manga> {
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
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        let path = match listing.id.as_str() {
            "popular" => "ComicList/MostPopular",
            "latest"  => "ComicList/LatestUpdate",
            "newest"  => "ComicList/Newest",
            _         => bail!("Unknown listing id"),
        };
        parse_comic_list(&format!("{}/{}?page={}", BASE_URL, path, page))
    }
}

register_source!(RcoSource, ListingProvider);

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_comic_list(url: &str) -> Result<MangaPageResult> {
    let doc = Request::get(url)?.html()?;

    let entries = doc
        .select(".list-comic > .item > a:first-child")
        .map(|list| {
            list.filter_map(|el| {
                let key   = el.attr("href")?;
                let title = el.text()?;
                let cover = el.select_first("img").and_then(|i| i.attr("src"));
                Some(Manga { key, title, cover, ..Default::default() })
            })
            .collect::<Vec<Manga>>()
        })
        .unwrap_or_default();

    let has_next_page = doc
        .select_first("ul.pager > li > a:contains(Next)")
        .is_some();

    Ok(MangaPageResult { entries, has_next_page })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku_test::aidoku_test;

    #[aidoku_test]
    fn test_listing_newest() {
        let result = parse_comic_list(
            &format!("{}/ComicList/Newest?page=1", BASE_URL)
        );
        assert!(result.is_ok(), "parse_comic_list failed: {:?}", result);
        let page = result.unwrap();
        assert!(!page.entries.is_empty(), "No entries returned");
        let first = &page.entries[0];
        assert!(!first.key.is_empty(), "Empty key");
        assert!(!first.title.is_empty(), "Empty title");
        println!("First comic: {} ({})", first.title, first.key);
    }
}
