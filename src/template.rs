use askama::Template; // bring trait in scope

#[derive(Template)]
#[template(path = "../templates/report.html")]
pub struct ReportTemplate<'a> {
    pub source: String, // TODO figure out how to make this a ref
    pub mark_uncovered: Vec<&'a str>,
    pub mark_covered: Vec<&'a str>,
    pub lines_hit: u32,
    pub lines_total: u32,
    pub lines_covered_per: f32,
    pub lines_uncovered_per: f32,
    pub fn_hit: u32,
    pub fn_total: u32,
    pub fn_covered_per: f32,
    pub fn_uncovered_per: f32,
    pub br_hit: u32,
    pub br_total: u32,
    pub br_covered_per: f32,
    pub br_uncovered_per: f32,
}

#[derive(Template)]
#[template(path = "../templates/index.html")]
pub struct IndexTemplate {
    pub files: Vec<IndexFile>,
}

pub struct IndexFile {
    pub source_path: String,
    pub report_path: String,
    pub lines_covered_per: f32,
    pub lines_uncovered_per: f32,
    pub fn_covered_per: f32,
    pub fn_uncovered_per: f32,
}