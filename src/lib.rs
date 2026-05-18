#![no_std]

use aidoku::{
    alloc::{format, string::String, vec, vec::Vec},
    imports::net::Request,
    prelude::*,
    Chapter, DynamicFilters, Filter, FilterValue, Listing, ListingProvider,
    Manga, MangaPageResult, MultiSelectFilter, Page, PageContent,
    Result, SelectFilter, Source, TextFilter,
};

const BASE_URL: &str = "https://readcomiconline.li";

struct RcoSource;

impl Source for RcoSource {
    fn new() -> Self { Self }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let url = build_search_url(query, page, &filters);
        parse_comic_list(&url)
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

impl DynamicFilters for RcoSource {
    fn get_dynamic_filters(&self) -> Result<Vec<Filter>> {
        Ok(vec![
            MultiSelectFilter {
                id: "genres".into(),
                title: Some("Genres".into()),
                is_genre: true,
                can_exclude: true,
                uses_tag_style: true,
                options: vec![
                    "Action".into(), "Adventure".into(), "Anthology".into(),
                    "Anthropomorphic".into(), "Biography".into(), "Children".into(),
                    "Comedy".into(), "Crime".into(), "Drama".into(), "Family".into(),
                    "Fantasy".into(), "Fighting".into(), "Graphic Novels".into(),
                    "Historical".into(), "Horror".into(), "Leading Ladies".into(),
                    "LGBTQ".into(), "Literature".into(), "Manga".into(),
                    "Martial Arts".into(), "Mature".into(), "Military".into(),
                    "Mini-Series".into(), "Movies & TV".into(), "Music".into(),
                    "Mystery".into(), "Mythology".into(), "Personal".into(),
                    "Political".into(), "Post-Apocalyptic".into(), "Psychological".into(),
                    "Pulp".into(), "Religious".into(), "Robots".into(), "Romance".into(),
                    "School Life".into(), "Sci-Fi".into(), "Slice of Life".into(),
                    "Sport".into(), "Spy".into(), "Superhero".into(), "Supernatural".into(),
                    "Suspense".into(), "Thriller".into(), "Vampires".into(),
                    "Video Games".into(), "War".into(), "Western".into(), "Zombies".into(),
                ],
                ids: None,
                default_included: None,
                default_excluded: None,
                hide_from_header: None,
            }.into(),
            SelectFilter {
                id: "status".into(),
                title: Some("Status".into()),
                options: vec!["All".into(), "Completed".into(), "Ongoing".into()],
                is_genre: false,
                uses_tag_style: false,
                ids: None,
                default: None,
                hide_from_header: None,
            }.into(),
            SelectFilter {
                id: "sort".into(),
                title: Some("Sort By".into()),
                options: vec![
                    "Latest Update".into(),
                    "Newest".into(),
                    "Most Popular".into(),
                    "Alphabetical".into(),
                ],
                is_genre: false,
                uses_tag_style: false,
                ids: None,
                default: None,
                hide_from_header: None,
            }.into(),
            TextFilter {
                id: "publisher".into(),
                title: Some("Publisher".into()),
                placeholder: None,
                hide_from_header: None,
            }.into(),
            TextFilter {
                id: "writer".into(),
                title: Some("Writer".into()),
                placeholder: None,
                hide_from_header: None,
            }.into(),
        ])
    }
}

register_source!(RcoSource, ListingProvider, DynamicFilters);

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

fn build_search_url(query: Option<String>, page: i32, filters: &[FilterValue]) -> String {
    let mut status        = String::new();
    let mut sort          = String::new();
    let mut publisher     = String::new();
    let mut writer        = String::new();
    let mut inc_names: Vec<String> = Vec::new();
    let mut exc_names: Vec<String> = Vec::new();

    for f in filters {
        match f {
            FilterValue::Select { id, value } => match id.as_str() {
                "status" => {
                    if value != "All" { status = value.clone(); }
                }
                "sort" => {
                    sort = match value.as_str() {
                        "Latest Update" => String::from("LatestUpdate"),
                        "Newest"        => String::from("Newest"),
                        "Most Popular"  => String::from("MostPopular"),
                        "Alphabetical"  => String::from("Alphabetical"),
                        _               => String::new(),
                    };
                }
                _ => {}
            },
            FilterValue::MultiSelect { id, included, excluded } if id == "genres" => {
                inc_names = included.clone();
                exc_names = excluded.clone();
            }
            FilterValue::Text { id, value } => match id.as_str() {
                "publisher" => publisher = value.clone(),
                "writer"    => writer    = value.clone(),
                _ => {}
            },
            _ => {}
        }
    }

    let q = query.unwrap_or_default();
    let has_query    = !q.is_empty();
    let single_genre = inc_names.len() == 1 && exc_names.is_empty() && !has_query && status.is_empty();

    if single_genre {
        let name = inc_names[0].replace(' ', "-");
        let sort_seg = if sort.is_empty() { "LatestUpdate" } else { sort.as_str() };
        return format!("{}/Genre/{}/{}?page={}", BASE_URL, name, sort_seg, page);
    }

    if has_query || !inc_names.is_empty() || !exc_names.is_empty() || !status.is_empty() {
        let ig: Vec<&str> = inc_names.iter().filter_map(|n| genre_id(n)).collect();
        let eg: Vec<&str> = exc_names.iter().filter_map(|n| genre_id(n)).collect();
        let mut url = format!("{}/AdvanceSearch?comicName={}&page={}", BASE_URL, url_encode(&q), page);
        if !ig.is_empty() { url.push_str(&format!("&ig={}", ig.join(","))); }
        if !eg.is_empty() { url.push_str(&format!("&eg={}", eg.join(","))); }
        if !status.is_empty() { url.push_str(&format!("&status={}", status)); }
        return url;
    }

    if !publisher.is_empty() {
        let sort_seg = if sort.is_empty() { "LatestUpdate" } else { sort.as_str() };
        return format!("{}/Publisher/{}/{}?page={}", BASE_URL, publisher.replace(' ', "-"), sort_seg, page);
    }

    if !writer.is_empty() {
        let sort_seg = if sort.is_empty() { "LatestUpdate" } else { sort.as_str() };
        return format!("{}/Writer/{}/{}?page={}", BASE_URL, writer.replace(' ', "-"), sort_seg, page);
    }

    let sort_seg = if sort.is_empty() { "LatestUpdate" } else { sort.as_str() };
    format!("{}/ComicList/{}?page={}", BASE_URL, sort_seg, page)
}

fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            b => {
                out.push('%');
                out.push(char::from_digit((b >> 4) as u32, 16).unwrap_or('0'));
                out.push(char::from_digit((b & 0xf) as u32, 16).unwrap_or('0'));
            }
        }
    }
    out
}

fn genre_id(name: &str) -> Option<&'static str> {
    match name {
        "Action"           => Some("1"),
        "Adventure"        => Some("2"),
        "Anthology"        => Some("38"),
        "Anthropomorphic"  => Some("46"),
        "Biography"        => Some("41"),
        "Children"         => Some("49"),
        "Comedy"           => Some("3"),
        "Crime"            => Some("17"),
        "Drama"            => Some("19"),
        "Family"           => Some("25"),
        "Fantasy"          => Some("20"),
        "Fighting"         => Some("31"),
        "Graphic Novels"   => Some("5"),
        "Historical"       => Some("28"),
        "Horror"           => Some("15"),
        "Leading Ladies"   => Some("35"),
        "LGBTQ"            => Some("51"),
        "Literature"       => Some("44"),
        "Manga"            => Some("40"),
        "Martial Arts"     => Some("4"),
        "Mature"           => Some("8"),
        "Military"         => Some("33"),
        "Mini-Series"      => Some("56"),
        "Movies & TV"      => Some("47"),
        "Music"            => Some("55"),
        "Mystery"          => Some("23"),
        "Mythology"        => Some("21"),
        "Personal"         => Some("48"),
        "Political"        => Some("42"),
        "Post-Apocalyptic" => Some("43"),
        "Psychological"    => Some("27"),
        "Pulp"             => Some("39"),
        "Religious"        => Some("53"),
        "Robots"           => Some("9"),
        "Romance"          => Some("32"),
        "School Life"      => Some("52"),
        "Sci-Fi"           => Some("16"),
        "Slice of Life"    => Some("50"),
        "Sport"            => Some("54"),
        "Spy"              => Some("30"),
        "Superhero"        => Some("22"),
        "Supernatural"     => Some("24"),
        "Suspense"         => Some("29"),
        "Thriller"         => Some("18"),
        "Vampires"         => Some("34"),
        "Video Games"      => Some("37"),
        "War"              => Some("26"),
        "Western"          => Some("45"),
        "Zombies"          => Some("36"),
        _                  => None,
    }
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

    #[aidoku_test]
    fn test_search_batman() {
        let result = parse_comic_list(
            &format!("{}/AdvanceSearch?comicName=batman&page=1", BASE_URL)
        );
        assert!(result.is_ok());
        let page = result.unwrap();
        assert!(!page.entries.is_empty(), "No results for 'batman'");
        println!("Search 'batman': {} results", page.entries.len());
    }
}
