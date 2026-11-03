use std::time::Instant;

use spike_epub::open_book;
use spike_pdf::sample;
use spike_report::{PdfEngine, PixelScale, Summary, require_release, stat};

fn main() {
    let started = Instant::now();
    if let Err(err) = require_release() {
        eprintln!("{err}");
        std::process::exit(2);
    }
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("pdf") => {
            let engine = match args.next().as_deref() {
                Some("pdfium") => PdfEngine::Pdfium,
                Some("hayro") => PdfEngine::Hayro,
                other => {
                    eprintln!("pdf engine {other:?}");
                    std::process::exit(2);
                }
            };
            let scale = match args.next().as_deref() {
                Some("zoom-2x") => PixelScale::Zoom2x,
                _ => PixelScale::Retina1x,
            };
            match sample(engine, &spike_report::gpt3_path(), scale, started) {
                Ok(body) => println!("{}", serde_json::to_string(&body).unwrap()),
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            }
        }
        Some("epub") => match epub_sample() {
            Ok(body) => println!("{}", serde_json::to_string(&body).unwrap()),
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        },
        _ => campaign(),
    }
}

fn campaign() {
    std::fs::create_dir_all("/tmp/superreader-spike").ok();
    let exe = std::env::current_exe().expect("exe");
    let mut runs = Vec::new();
    for scale in PixelScale::ALL {
        for _ in 0..5 {
            for engine in [PdfEngine::Pdfium, PdfEngine::Hayro] {
                let output = std::process::Command::new(&exe)
                    .args(["pdf", engine.name(), scale.name()])
                    .output()
                    .expect("child");
                if !output.status.success() {
                    eprintln!(
                        "{} {} failed\n{}",
                        engine.name(),
                        scale.name(),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    std::process::exit(1);
                }
                let sample: spike_report::PdfSample =
                    serde_json::from_slice(&output.stdout).expect("child json");
                if sample.errors > 0 || !sample.rgba_ok {
                    eprintln!(
                        "{} {} errors {} rgba_ok {}",
                        sample.engine, sample.scale, sample.errors, sample.rgba_ok
                    );
                    std::process::exit(1);
                }
                runs.push(sample);
            }
        }
    }
    let mut summaries = Vec::new();
    for scale in PixelScale::ALL {
        for engine in [PdfEngine::Pdfium, PdfEngine::Hayro] {
            let group: Vec<_> = runs
                .iter()
                .filter(|run| run.engine == engine.name() && run.scale == scale.name())
                .collect();
            let later: Vec<f64> = group.iter().flat_map(|run| run.later_page_ms.clone()).collect();
            summaries.push(Summary {
                engine: engine.name().into(),
                scale: scale.name().into(),
                runs: group.len(),
                open_to_first_ms: stat(&group.iter().map(|r| r.open_to_first_ms).collect::<Vec<_>>()),
                first_raster_ms: stat(&group.iter().map(|r| r.first_raster_ms).collect::<Vec<_>>()),
                later_page_ms: stat(&later),
                rss_after_last_kb: stat(
                    &group
                        .iter()
                        .map(|r| r.rss_after_last_kb as f64)
                        .collect::<Vec<_>>(),
                ),
                text_hits_min: group.iter().map(|r| r.text_hits).min().unwrap_or(0),
                errors: group.iter().map(|r| r.errors).sum(),
            });
        }
    }
    let path = "/tmp/superreader-spike/pdf-summary.json";
    std::fs::write(path, serde_json::to_string_pretty(&summaries).unwrap()).unwrap();
    println!("{}", serde_json::to_string_pretty(&summaries).unwrap());
    eprintln!("wrote {path}");
}

fn epub_sample() -> Result<serde_json::Value, String> {
    let started = Instant::now();
    let pride = open_book(&spike_report::pride_path())?;
    let pride_open_ms = started.elapsed().as_secs_f64() * 1000.0;
    let chapter = &pride.chapters[0];
    let blitz = spike_epub::paginate_blitz(&chapter.flow_html)?;
    let zhaohua = open_book(&spike_report::zhaohua_path())?;
    let zchapter = &zhaohua.chapters[0];
    let zblitz = spike_epub::paginate_blitz(&zchapter.flow_html)?;
    Ok(serde_json::json!({
        "pride_title": pride.title,
        "pride_open_ms": pride_open_ms,
        "pride_chapters": pride.chapters.len(),
        "pride_href": chapter.href,
        "pride_plain_chars": chapter.plain.chars().count(),
        "pride_has_truth": chapter.plain.contains("truth universally acknowledged"),
        "blitz_columns": blitz.columns,
        "blitz_pages": blitz.count,
        "blitz_raster_ms": blitz.raster_ms,
        "blitz_rgba": blitz.first.len(),
        "zhaohua_title": zhaohua.title,
        "zhaohua_href": zchapter.href,
        "zhaohua_plain_chars": zchapter.plain.chars().count(),
        "zhaohua_columns": zblitz.columns,
        "zhaohua_pages": zblitz.count,
        "zhaohua_raster_ms": zblitz.raster_ms,
    }))
}
