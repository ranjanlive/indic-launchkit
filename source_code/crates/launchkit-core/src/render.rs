use crate::{asset, Project, Result, MAX_FILE};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{DynamicImage, GenericImageView, ImageReader};
use std::{
    fs::File,
    io::{BufReader, Cursor},
    path::Path,
};

const LATIN: &[u8] = include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf");
const DEVANAGARI: &[u8] = include_bytes!("../../../../assets/fonts/NotoSansDevanagari-Regular.ttf");
pub fn validate_text(text: &str) -> Result<()> {
    let fonts = [
        ttf_parser::Face::parse(LATIN, 0).map_err(|e| e.to_string())?,
        ttf_parser::Face::parse(DEVANAGARI, 0).map_err(|e| e.to_string())?,
    ];
    if let Some(c) = text
        .chars()
        .find(|c| !c.is_whitespace() && !fonts.iter().any(|f| f.glyph_index(*c).is_some()))
    {
        return Err(format!(
            "Bundled fonts do not contain character U+{:04X}; choose supported text",
            c as u32
        ));
    }
    Ok(())
}
pub fn decode(path: &Path) -> Result<DynamicImage> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > MAX_FILE {
        return Err("Image exceeds 64 MiB".into());
    }
    let mut reader = ImageReader::new(BufReader::new(File::open(path).map_err(|e| e.to_string())?))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| format!("Cannot decode image: {e}"))?;
    let (w, h) = image.dimensions();
    if w == 0 || h == 0 || w as u64 * h as u64 > 24_000_000 {
        return Err("Image must have 1–24 million pixels and dimensions at most 8192px".into());
    }
    Ok(image)
}
fn image_uri(path: &Path) -> Result<String> {
    let image = decode(path)?.thumbnail(1440, 1920);
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(output.into_inner())
    ))
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn text_lines(text: &str, max: usize) -> Vec<String> {
    let latin = ttf_parser::Face::parse(LATIN, 0).unwrap();
    let devanagari = ttf_parser::Face::parse(DEVANAGARI, 0).unwrap();
    let measure = |s: &str| {
        s.chars()
            .map(|c| {
                let (f, g) = if let Some(g) = latin.glyph_index(c) {
                    (&latin, g)
                } else {
                    (
                        &devanagari,
                        devanagari.glyph_index(c).unwrap_or(ttf_parser::GlyphId(0)),
                    )
                };
                f.glyph_hor_advance(g).unwrap_or(600) as f32 / f.units_per_em() as f32
            })
            .sum::<f32>()
    };
    let limit = max as f32 * 0.55;
    let mut lines = vec![];
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if measure(&candidate) <= limit {
            line = candidate;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        for c in word.chars() {
            let candidate = format!("{line}{c}");
            if measure(&candidate) > limit && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            line.push(c);
        }
    }
    if !line.is_empty() {
        lines.push(line)
    }
    lines
}
pub fn check_layout(p: &Project) -> Result<()> {
    if text_lines(&p.title, 21).len() > 2 {
        return Err("Title is too long for the feature graphic; shorten it".into());
    }
    if text_lines(&p.tagline, 37).len() > 3 {
        return Err("Tagline is too long for the feature graphic; shorten it".into());
    }
    for shot in &p.screenshots {
        let caption = if shot.caption.trim().is_empty() {
            &p.title
        } else {
            &shot.caption
        };
        let lines = text_lines(caption, 24).len();
        if lines > 3 || 204 + lines * 80 + text_lines(&p.tagline, 55).len() * 41 > 500 {
            return Err(
                "Caption or tagline is too long for the screenshot layout; shorten it".into(),
            );
        }
    }
    Ok(())
}
fn text(text: &str, x: u32, y: u32, size: u32, color: &str, max: usize) -> String {
    text_lines(text,max).iter().enumerate().map(|(i,line)|format!("<text x='{x}' y='{}' font-family='Noto Sans' font-size='{size}' fill='{color}'>{}</text>",y+i as u32*(size+14),escape(line))).collect()
}
pub fn svg(root: &Path, p: &Project, kind: &str, index: usize) -> Result<String> {
    check_layout(p)?;
    let (bg, accent, ink, muted) = match p.template.as_str() {
        "paper" => ("#FFF5E3", "#AC612C", "#30251D", "#886E58"),
        "midnight" => ("#111B30", "#8CD6DB", "#FAFCFF", "#A6B4D0"),
        _ => ("#EEEAFB", "#6451D2", "#232443", "#77718E"),
    };
    let (width, height) = match kind {
        "feature" => (1024, 500),
        "social" => (1200, 630),
        "icon" => (512, 512),
        "screenshot" => (1080, 1920),
        _ => return Err("Unknown render kind".into()),
    };
    let mut content=format!("<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' width='{width}' height='{height}' viewBox='0 0 {width} {height}'><rect width='{width}' height='{height}' fill='{bg}'/>");
    if kind == "icon" {
        let icon = p.icon.as_ref().ok_or("Add an icon")?;
        let uri = image_uri(&asset(root, icon)?)?;
        content.push_str(&format!("<image x='0' y='0' width='512' height='512' preserveAspectRatio='xMidYMid meet' xlink:href='{uri}'/>"));
    } else if kind == "screenshot" {
        let shot = p
            .screenshots
            .get(index)
            .ok_or("Screenshot index out of range")?;
        content.push_str(&format!("<circle cx='920' cy='1280' r='640' fill='{accent}' opacity='.07'/><path d='M74 100h92' stroke='{accent}' stroke-width='8' stroke-linecap='round'/>"));
        let caption = if shot.caption.trim().is_empty() {
            &p.title
        } else {
            &shot.caption
        };
        let lines = text_lines(caption, 24);
        if lines.len() > 3 {
            return Err("Caption is too long for the screenshot template; shorten it".into());
        }
        content.push_str(&text(caption, 74, 204, 66, ink, 24));
        content.push_str(&text(
            &p.tagline,
            78,
            204 + lines.len() as u32 * 80,
            27,
            muted,
            55,
        ));
        let uri = image_uri(&asset(root, &shot.file)?)?;
        content.push_str(&format!("<rect x='180' y='520' width='720' height='1288' rx='62' fill='#12192A' opacity='.13'/><rect x='180' y='506' width='720' height='1288' rx='62' fill='#1D2335'/><clipPath id='screen'><rect x='193' y='519' width='694' height='1262' rx='49'/></clipPath><rect x='193' y='519' width='694' height='1262' rx='49' fill='{bg}'/><image x='193' y='519' width='694' height='1262' preserveAspectRatio='xMidYMid meet' clip-path='url(#screen)' xlink:href='{uri}'/>"));
        content.push_str(&text(&p.title, 74, 1860, 24, muted, 64));
    } else {
        let uri = image_uri(&asset(root, p.icon.as_ref().ok_or("Add an icon")?)?)?;
        content.push_str(&format!("<circle cx='{width}' cy='{height}' r='440' fill='{accent}' opacity='.1'/><rect x='58' y='58' width='112' height='112' rx='24' fill='#FFFFFF' opacity='.6'/><image x='64' y='64' width='100' height='100' preserveAspectRatio='xMidYMid meet' xlink:href='{uri}'/>"));
        if text_lines(&p.title, 21).len() > 2 {
            return Err("Title is too long for the feature graphic; shorten it".into());
        }
        content.push_str(&text(&p.title, 58, 246, 55, ink, 21));
        content.push_str(&text(&p.tagline, 61, 366, 24, muted, 37));
        if let Some(shot) = p.screenshots.first() {
            let uri = image_uri(&asset(root, &shot.file)?)?;
            let x = width - 300;
            content.push_str(&format!("<rect x='{x}' y='50' width='236' height='460' rx='24' fill='#20283C'/><clipPath id='screen'><rect x='{}' y='57' width='222' height='446' rx='18'/></clipPath><image x='{}' y='57' width='222' height='446' preserveAspectRatio='xMidYMid meet' clip-path='url(#screen)' xlink:href='{uri}'/>",x+7,x+7));
        }
    }
    content.push_str("</svg>");
    Ok(content)
}
pub fn png(root: &Path, p: &Project, kind: &str, index: usize, preview: bool) -> Result<Vec<u8>> {
    let svg = svg(root, p, kind, index)?;
    let mut options = resvg::usvg::Options::default();
    options.font_family = "Noto Sans".into();
    options.fontdb_mut().load_font_data(LATIN.to_vec());
    options.fontdb_mut().load_font_data(DEVANAGARI.to_vec());
    // Templates embed all images. No external files or network resources resolve.
    options.image_href_resolver.resolve_string = Box::new(|_, _| None);
    let tree = resvg::usvg::Tree::from_str(&svg, &options).map_err(|e| e.to_string())?;
    let scale = if preview {
        480.0 / tree.size().width()
    } else {
        1.0
    };
    let mut image = resvg::tiny_skia::Pixmap::new(
        (tree.size().width() * scale).round() as u32,
        (tree.size().height() * scale).round() as u32,
    )
    .ok_or("Output allocation failed")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut image.as_mut(),
    );
    image.encode_png().map_err(|e| e.to_string())
}
