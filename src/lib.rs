#![no_std]

use aidoku::{
    alloc::{format, string::String, vec, vec::Vec},
    imports::{js::JsContext, net::Request},
    prelude::*,
    Chapter, ContentRating, DynamicFilters, Filter, FilterValue, Listing, ListingProvider,
    Manga, MangaPageResult, MangaStatus, MultiSelectFilter, Page, PageContent,
    Result, SelectFilter, Source, TextFilter,
};

const BASE_URL: &str = "https://readcomiconline.li";

const IMAGE_DECRYPT_EVAL: &str = r#"const pageLinks=new Array;const urlPattern=/^https?:\/\/(?:www\.)?[a-z0-9-]+(?:\.[a-z0-9-]+)+\b(?:[\/a-z0-9-._~:?#@!$&'()*+,;=%]*)$/i;const reverseOrder=false;const replacePatternRegex=/\.replace\(\s*\/(\w+__\w+_)\/g\s*,\s*['"]([\w])['\"]\s*\)/;const replaceMatch=_encryptedString.match(replacePatternRegex);const obfuscationPattern=replaceMatch?new RegExp(replaceMatch[1],"g"):/\w{2}__\w{6}_/g;const replacementChar=replaceMatch?replaceMatch[2]:"e";const arrayVars=[..._encryptedString.matchAll(/var\s+(\w+)\s*=\s*new\s+Array\(\)\s*;/g)].map((t=>t[1]));const baseUrlMatch=_encryptedString.match(/baeu\(\w+,\s*["'](https?:\/\/[^"']+)["']\)/);const detectedBaseUrl=baseUrlMatch?baseUrlMatch[1]:null;arrayVars.forEach((t=>{const e=new RegExp("\\w+\\s*\\([^)]*\\b"+t+"\\b[^)]*,\\s*[\"']([^\"']{20,})[\"'][,\\s]*\\)","g");const s=[..._encryptedString.matchAll(e)];if(s.length===0)return;const r=s.map((t=>t[1]));const n=findPrefixOffset(r);s.forEach((t=>{if(t[1])pageLinks.push(decryptLink(t[1],n))}))}));function findPrefixOffset(t){if(t.length===0)return 0;const e=t[0];let s=0;for(let r=0;r<e.length;r++){const n=e[r];if(t.every((t=>t[r]===n))){s++;if(s>=5&&e.slice(s-5,s)==="https")return s-5}else break}return s}function atob(t){const e="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";let s=String(t).replace(/=+$/,"");if(s.length%4===1)throw new Error("'atob' failed: The string to be decoded is not correctly encoded.");let r="";for(let t=0,n,o,c=0;o=s.charAt(c++);~o&&(n=t%4?n*64+o:o,t++%4)?r+=String.fromCharCode(255&n>>(-2*t&6)):0)o=e.indexOf(o);return r}function decryptLink(t,e=0){let s=t.replace(obfuscationPattern,replacementChar).replace(/pw_.g28x/g,"b").replace(/d2pr.x_27/g,"h");if(e!=0)s=s.substr(e,s.length-e);if(s.endsWith("=s0")||s.endsWith("=s1600"))s=s.replace("https://2.bp.blogspot.com/","")+"?";if(!s.startsWith("https")){const t=s.indexOf("?");const e=s.substring(t);const r=s.includes("=s0?");const n=r?s.indexOf("=s0?"):s.indexOf("=s1600?");let o=s.substring(0,n);o=o.substring(15,33)+o.substring(50);const c=o.length;o=o.substring(0,c-11)+o[c-2]+o[c-1];const a=atob(o);let i=decodeURIComponent(a);i=i.substring(0,13)+i.substring(17);i=i.substring(0,i.length-2)+(r?"=s0":"=s1600");const p=detectedBaseUrl??(_useServer2?"https://ano1.rconet.biz/pic":"https://2.bp.blogspot.com");s=`${p}/${i}${e}${_useServer2?"&t=10":""}`}return s}const blocklist=["https://2.bp.blogspot.com/pw/AP1GczP6zCVVfdmN6OoVnm7CLvEfmHMUawyEwJWouX9C6SHwsiuYfLkUr9FsM6Zo34qNzPKeQeahBx9ckBZJQckiJmX1UwKD7uh900yz5rKyG4zT2rfIrqFviEJIev1Pg_pGRuSG57rIH6BDwGCTmiE4MjA","https://2.bp.blogspot.com/pw/AP1GczP48thKMga7cud0tjtHtYqsvZzhYY0HyAxVzM3O1D6tkLbi0fT9NDZFFFH69hNnoGsnqJSEIh4mmpEoU1BJSfNXIz1f5aLXl41RM9os7ePn7ipbrYbIuqiQxAV0hhJZrNLl7FmauwLQ01paCrP6KAE","https://2.bp.blogspot.com/pw/AP1GczNXprTMfAP2AHFFWvCbKq6qReXrqSohz87KeBjV0nh6XoLsE1NpzL7Rp9llxoY208IPARiIDON_TO6dZB0ZMNeB8J7xzUzbS9h6To7aGpOZshFofw-wFQ0KJ3y3wolSwzLrduZZ_0w8_6gGuTEB-98","https://2.bp.blogspot.com/pw/AP1GczMVY_zWeag2n981CRX7jaZ73Sr0NtidtJhnvJ3-Rmh2fIo-PoQRI0ZksQEbpTjDHgBeNYbQ2hQodsY-Dv0FXUhiU_mus5z5L5lMVAH82kXYqOd2IEw","https://2.bp.blogspot.com/pw/AP1GczOKY-6EDGVvlQGB2wj0xxB5JgcyiujFJC3CHgwqBOLIidwmoP6DLiMpX__Fw6MMPvLezN6soeV0A8pKSHUrC4rxZyO5vov40g1g4ipZdkFlzUouAFA","https://2.bp.blogspot.com/pw/AP1GczO8AETT3k19nhJwxHm0sHCSy0tXyhSOYxnq3EUrmlvgY5yPqDaxcd1XZ7reQKH-lKgpGK4o3sW_9Yu6feqii79riXN3Ghi8Xs1S5Z4wi-aeHrq5PzOX"];function getCleanedLinks(){const t=pageLinks.filter(((t,e)=>{if(!t)return false;const s=t.split("?")[0].split("=")[0];const r=pageLinks.findIndex((t=>t.split("?")[0].split("=")[0]===s))===e;const n=blocklist.indexOf(s)===-1;const o=urlPattern.test(s);return r&&n&&o}));return reverseOrder?t.reverse():t}JSON.stringify(getCleanedLinks());"#;

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
            let info = match doc.select_first("div.barContent") {
                Some(el) => el,
                None => bail!("comic detail section not found in page"),
            };

            if let Some(title) = info.select_first("a.bigChar").and_then(|e| e.text()) {
                manga.title = title;
            }

            manga.cover = doc
                .select_first(".rightBox:eq(0) img")
                .and_then(|e| e.attr("src"));

            manga.description = info
                .select("p:has(span:contains(Summary:)) ~ p")
                .map(|list| {
                    list.filter_map(|e| e.text())
                        .collect::<Vec<_>>()
                        .join("\n\n")
                })
                .filter(|s| !s.is_empty());

            manga.tags = info
                .select("p:has(span:contains(Genres:)) > a")
                .map(|list| list.filter_map(|e| e.text()).collect::<Vec<_>>());

            manga.status = info
                .select_first("p:has(span:contains(Status:))")
                .and_then(|e| e.text())
                .map(|s| {
                    if s.contains("Ongoing") {
                        MangaStatus::Ongoing
                    } else if s.contains("Completed") {
                        MangaStatus::Completed
                    } else {
                        MangaStatus::Unknown
                    }
                })
                .unwrap_or(MangaStatus::Unknown);

            manga.content_rating = ContentRating::Safe;
        }

        if needs_chapters {
            manga.chapters = doc
                .select("table.listing tr")
                .map(|list| {
                    list.skip(2)
                        .filter_map(|row| {
                            let a    = row.select_first("a")?;
                            let key  = a.attr("href")?;
                            let name = a.text()?;
                            let date = row
                                .select_first("td:eq(1)")
                                .and_then(|e| e.text())
                                .and_then(|s| parse_date(&s));
                            Some(Chapter {
                                key,
                                title: Some(name),
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
        let url = format!("{}{}?readType=1&quality=hq", BASE_URL, chapter.key);
        let doc = Request::get(&url)?.html()?;

        let combined = doc
            .select("script")
            .map(|list| {
                list.filter_map(|el| el.data())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        let js = format!(
            "let _encryptedString = {}; let _useServer2 = false; {}",
            json_escape(&combined),
            IMAGE_DECRYPT_EVAL
        );

        let ctx = JsContext::new();
        let json_result = match ctx.eval(&js) {
            Ok(r) => r,
            Err(_) => bail!("JavaScript image decryption failed — source may need updating"),
        };

        let urls = parse_json_string_array(&json_result);

        if urls.is_empty() {
            bail!("No images found — decryption may need updating");
        }

        Ok(urls
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

fn parse_date(s: &str) -> Option<i64> {
    // Format: MM/dd/yyyy
    let mut parts = s.trim().split('/');
    let month: i64 = parts.next()?.trim().parse().ok()?;
    let day:   i64 = parts.next()?.trim().parse().ok()?;
    let year:  i64 = parts.next()?.trim().parse().ok()?;
    days_since_epoch(year, month, day).checked_mul(86400)
}

fn days_since_epoch(year: i64, month: i64, day: i64) -> i64 {
    let (y, m) = if month <= 2 { (year - 1, month + 12) } else { (year, month) };
    let jdn = day + (153 * m - 457) / 5 + 365 * y + y / 4 - y / 100 + y / 400 + 1721119;
    jdn - 2440588
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"'  => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 32 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c    => out.push(c),
        }
    }
    out.push('"');
    out
}

fn parse_json_string_array(json: &str) -> Vec<String> {
    let mut result  = Vec::new();
    let mut in_str  = false;
    let mut start   = 0usize;
    let bytes       = json.as_bytes();
    let mut i       = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b'"' if !in_str => {
                in_str = true;
                start  = i + 1;
            }
            b'"' if in_str => {
                let mut backs = 0usize;
                let mut j = i;
                while j > 0 && bytes[j - 1] == b'\\' {
                    backs += 1;
                    j -= 1;
                }
                if backs % 2 == 0 {
                    if let Ok(s) = core::str::from_utf8(&bytes[start..i]) {
                        if !s.is_empty() {
                            result.push(String::from(s));
                        }
                    }
                    in_str = false;
                }
            }
            _ => {}
        }
        i += 1;
    }
    result
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

    #[aidoku_test]
    fn test_manga_detail() {
        let manga = Manga {
            key: String::from("/Comic/Batman-2016"),
            title: String::from("Batman"),
            ..Default::default()
        };
        let source = RcoSource::new();
        let result = source.get_manga_update(manga, true, true);
        assert!(result.is_ok(), "get_manga_update failed: {:?}", result);
        let m = result.unwrap();
        assert!(!m.title.is_empty(), "Empty title");
        assert!(m.chapters.is_some(), "No chapters");
        let chapters = m.chapters.unwrap();
        assert!(!chapters.is_empty(), "Chapter list is empty");
        println!("Title: {}", m.title);
        println!("Chapters: {}", chapters.len());
        println!("First chapter key: {}", chapters[0].key);
    }

    #[aidoku_test]
    fn test_page_list() {
        let manga = Manga {
            key: String::from("/Comic/Batman-2016"),
            title: String::from("Batman"),
            ..Default::default()
        };
        let source = RcoSource::new();
        let with_chapters = source.get_manga_update(manga.clone(), false, true).unwrap();
        let chapters = with_chapters.chapters.expect("No chapters found");
        assert!(!chapters.is_empty());

        let chapter = chapters.last().unwrap().clone();
        println!("Testing page list for chapter: {}", chapter.key);

        let result = source.get_page_list(manga, chapter);
        assert!(result.is_ok(), "get_page_list failed: {:?}", result);
        let pages = result.unwrap();
        assert!(!pages.is_empty(), "No pages returned");
        println!("Pages: {}", pages.len());
    }
}
