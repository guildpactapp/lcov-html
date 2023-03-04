use crate::template::IndexFile;
use crate::template::IndexTemplate;
use crate::template::ReportTemplate;
use askama::Template;
use clap::{load_yaml, App};
use lcov::{LcovParser, LcovReport};
use std::fs;
use std::path::Path;

mod lcov;
mod template;

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    let files = matches.values_of("file").expect("No input file provided");

    let source = matches.value_of("source").unwrap_or(".");
    let source = fs::canonicalize(source).expect("Failed to canonicalize path");
    let source = source.as_path();

    let output = matches.value_of("output").unwrap_or("./coverage/");
    let output = fs::canonicalize(output).expect("Failed to canonicalize path");
    let output_path = output.as_path();

    for file in files {
        let canonical_file = fs::canonicalize(file).expect("Failed to canonicalize path");
        let file_path = Path::new(&canonical_file);
        if !file_path.exists() {
            panic!("Unable to locate file: '{}'", file);
        }

        match fs::read_to_string(file_path) {
            Ok(content) => {
                let lcov_file = LcovParser::parse(&content).expect("Failed to parse file");
                let mut index_files = vec![];
                for report in lcov_file.reports {
                    let template = generate_report(&report, source);
                    let out = output_path.join(report.path).with_extension("html");
                    fs::create_dir_all(out.parent().unwrap())
                        .expect("Failed to make intermediate dirs");
                    fs::write(&out, template.render().unwrap()).expect(&format!(
                        "Unable to write to file '{}'",
                        out.to_str().unwrap()
                    ));

                    let index_file = IndexFile {
                        source_path: report.path.to_string(),
                        report_path: out.to_str().unwrap().to_string(),
                        lines_covered_per: template.lines_covered_per,
                        lines_uncovered_per: template.lines_uncovered_per,
                        fn_covered_per: template.fn_covered_per,
                        fn_uncovered_per: template.fn_uncovered_per,
                    };

                    index_files.push(index_file);
                }

                index_files.sort_by(|a, b| a.source_path.cmp(&b.source_path));
                let index_template = IndexTemplate { files: index_files };

                let index_path = output_path.join("index").with_extension("html");
                fs::write(index_path, index_template.render().unwrap()).unwrap();
            }
            Err(_) => panic!("Unable to read file: '{}'", file),
        }
    }
}

fn generate_report<'a>(report: &'a LcovReport, source_dir: &Path) -> ReportTemplate<'a> {
    let source_file = report.path;
    let source_path = source_dir.join(source_file);
    if !source_path.exists() {
        panic!("Unable to locate source file: '{}'", source_file);
    }

    match fs::read_to_string(source_path) {
        Ok(source) => {
            let line_coverage_percent = (report.ln_hit as f32 / report.ln_found as f32) * 100.0;
            let line_uconvered_percent = 100.0 - line_coverage_percent;

            let fn_coverage_percent = (report.fn_hit as f32 / report.fn_found as f32) * 100.0;
            let fn_uconvered_percent = 100.0 - fn_coverage_percent;

            let br_coverage_percent = (report.br_hit as f32 / report.br_found as f32) * 100.0;
            let br_uconvered_percent = 100.0 - br_coverage_percent;

            ReportTemplate {
                source: source,
                mark_uncovered: report
                    .ln_data
                    .iter()
                    .filter(|(_, v)| **v == 0)
                    .map(|(k, _)| *k)
                    .collect(),
                mark_covered: report
                    .ln_data
                    .iter()
                    .filter(|(_, v)| **v > 0)
                    .map(|(k, _)| *k)
                    .collect(),
                lines_hit: report.ln_hit,
                lines_total: report.ln_found,
                lines_covered_per: line_coverage_percent,
                lines_uncovered_per: line_uconvered_percent,
                fn_hit: report.fn_hit,
                fn_total: report.fn_found,
                fn_covered_per: fn_coverage_percent,
                fn_uncovered_per: fn_uconvered_percent,
                br_hit: report.br_hit,
                br_total: report.br_found,
                br_covered_per: br_coverage_percent,
                br_uncovered_per: br_uconvered_percent,
            }
        }
        Err(_) => panic!("Failed to read source file: '{}'", source_file),
    }
}
