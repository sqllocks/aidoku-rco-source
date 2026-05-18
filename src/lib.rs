#![no_std]

use aidoku::{
    alloc::{format, string::String, vec, vec::Vec},
    imports::net::Request,
    prelude::*,
    Chapter, ContentRating, DynamicFilters, Filter, FilterValue, Listing, ListingProvider,
    Manga, MangaPageResult, MangaStatus, Page, PageContent,
    Result, Source,
};

const BASE_URL: &str = "https://readcomicsonline.ru";

struct RcoSource;

impl Source for RcoSource {
    fn new() -> Self { Self }

    fn get_search_manga_list(
        &self,
        _query: Option<String>,
        page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        // No accessible search endpoint on this site; fall back to full comic list
        parse_comic_list(&format!("{}/comic-list?page={}", BASE_URL, page))
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        if !needs_details && !needs_chapters {
            return Ok(manga);
        }

        let url = format!("{}{}", BASE_URL, manga.key);
        let doc = Request::get(&url)?.html()?;

        if needs_details {
            // Cover URL is predictable from the slug
            let slug = manga.key.trim_start_matches("/comic/");
            manga.cover = Some(format!(
                "{}/uploads/manga/{}/cover/cover_250x350.jpg",
                BASE_URL, slug
            ));

            if let Some(title) = doc.select_first("h2.widget-title").and_then(|e| e.text()) {
                manga.title = title;
            }

            manga.description = doc
                .select_first(".summary")
                .and_then(|e| e.text())
                .filter(|s| !s.is_empty());

            manga.status = MangaStatus::Unknown;
            manga.content_rating = ContentRating::Safe;
        }

        if needs_chapters {
            manga.chapters = doc.select(".chapter-item").map(|list| {
                list.filter_map(|item| {
                    let a = item.select_first("h5 > a")?;
                    let href = a.attr("href")?;
                    let key = href
                        .strip_prefix(BASE_URL)
                        .unwrap_or(href.as_str())
                        .to_string();
                    let title = a.text()?;
                    let date = item
                        .select_first("span")
                        .and_then(|e| e.text())
                        .and_then(|s| parse_date(&s));
                    Some(Chapter {
                        key,
                        title: Some(title),
                        date_uploaded: date,
                        ..Default::default()
                    })
                })
                .collect::<Vec<Chapter>>()
            });
        }

        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = format!("{}{}", BASE_URL, chapter.key);
        let doc = Request::get(&url)?.html()?;

        // First image on the page gives us the URL pattern for all pages
        let first_img = match doc
            .select_first("img[src*='/uploads/manga/']")
            .and_then(|e| e.attr("src"))
        {
            Some(u) => u,
            None => bail!("No images found on reader page — source may need updating"),
        };

        // Normalise to absolute URL
        let first_img = if first_img.starts_with("//") {
            format!("https:{}", first_img)
        } else if first_img.starts_with('/') {
            format!("{}{}", BASE_URL, first_img)
        } else {
            first_img
        };

        // Total page count = largest number in the pagination links
        let total_pages = doc
            .select(".pager a, .pagination a")
            .map(|list| {
                list.filter_map(|a| {
                    a.text().and_then(|t| t.trim().parse::<i32>().ok())
                })
                .max()
                .unwrap_or(1)
            })
            .unwrap_or(1);

        let pages = generate_page_urls(&first_img, total_pages);
        if pages.is_empty() {
            bail!("Could not generate page URLs for this chapter");
        }

        Ok(pages
            .into_iter()
            .map(|url| Page {
                content: PageContent::url(url),
                ..Default::default()
            })
            .collect())
    }
}

impl ListingProvider for RcoSource {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        let url = match listing.id.as_str() {
            "latest" => format!("{}/latest-release?page={}", BASE_URL, page),
            "all"    => format!("{}/comic-list?page={}", BASE_URL, page),
            _        => bail!("Unknown listing id"),
        };
        parse_comic_list(&url)
    }
}

impl DynamicFilters for RcoSource {
    fn get_dynamic_filters(&self) -> Result<Vec<Filter>> {
        Ok(vec![])
    }
}

register_source!(RcoSource, ListingProvider, DynamicFilters);

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_comic_list(url: &str) -> Result<MangaPageResult> {
    let doc = Request::get(url)?.html()?;

    let entries = doc
        .select("div.manga-item h3.manga-heading > a")
        .map(|list| {
            list.filter_map(|a| {
                let href = a.attr("href")?;
                let key = href
                    .strip_prefix(BASE_URL)
                    .unwrap_or(href.as_str())
                    .to_string();
                let title = a.text()?;
                let slug = key.trim_start_matches("/comic/");
                let cover = Some(format!(
                    "{}/uploads/manga/{}/cover/cover_250x350.jpg",
                    BASE_URL, slug
                ));
                Some(Manga { key, title, cover, ..Default::default() })
            })
            .collect::<Vec<Manga>>()
        })
        .unwrap_or_default();

    // has_next_page: there is a pagination "next" link (an <a> inside li.disabled
    // would mean last page; an <a> that exists means more pages remain).
    // Use a simple check: any pagination link with href containing "page=" exists.
    let has_next_page = doc
        .select("ul.pagination li > a")
        .map(|list| list.any(|a| {
            a.attr("href")
                .map(|h| h.contains("page="))
                .unwrap_or(false)
        }))
        .unwrap_or(false);

    Ok(MangaPageResult { entries, has_next_page })
}

/// Given the first page's image URL and a total page count, generate all URLs.
/// Pattern: `.../chapters/{id}/01.jpg` → `.../01.jpg`, `.../02.jpg`, ...
fn generate_page_urls(first_url: &str, total: i32) -> Vec<String> {
    let slash = match first_url.rfind('/') {
        Some(p) => p,
        None    => return Vec::new(),
    };
    let prefix   = &first_url[..slash];
    let filename = &first_url[slash + 1..];

    let (stem, ext) = match filename.rfind('.') {
        Some(p) => (&filename[..p], &filename[p..]), // ext includes the dot
        None    => (filename, ""),
    };

    let pad = stem.len().max(2);

    (1..=total)
        .map(|n| format!("{}/{:0>width$}{}", prefix, n, ext, width = pad))
        .collect()
}

fn parse_date(s: &str) -> Option<i64> {
    // Format: "12 Nov. 2025" or "5 Mar. 2026"
    let s = s.trim();
    let mut parts = s.split_whitespace();
    let day: i64   = parts.next()?.trim().parse().ok()?;
    let mon_raw    = parts.next()?;
    let month: i64 = match mon_raw.trim_end_matches('.') {
        "Jan" => 1,  "Feb" => 2,  "Mar" => 3,  "Apr" => 4,
        "May" => 5,  "Jun" => 6,  "Jul" => 7,  "Aug" => 8,
        "Sep" => 9,  "Oct" => 10, "Nov" => 11, "Dec" => 12,
        _ => return None,
    };
    let year: i64  = parts.next()?.trim().parse().ok()?;
    days_since_epoch(year, month, day).checked_mul(86400)
}

fn days_since_epoch(year: i64, month: i64, day: i64) -> i64 {
    let (y, m) = if month <= 2 { (year - 1, month + 12) } else { (year, month) };
    let jdn = day + (153 * m - 457) / 5 + 365 * y + y / 4 - y / 100 + y / 400 + 1_721_119;
    jdn - 2_440_588
}
