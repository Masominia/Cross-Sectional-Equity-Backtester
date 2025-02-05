//! Reader for the panel file: a header line, then one row per day; the first column is a day counter.

use std::fs;

use super::panel::Panel;

pub fn parse(text: &str) -> Result<Panel, String> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().ok_or("empty file")?;
    let names: Vec<String> = header.trim_end_matches('\r').split(',').skip(1).map(|s| s.trim().to_string()).collect();
    if names.is_empty() {
        return Err("no asset columns".into());
    }
    let mut prices = Vec::new();
    for (row, line) in lines.enumerate() {
        let cells: Vec<&str> = line.trim_end_matches('\r').split(',').collect();
        if cells.len() != names.len() + 1 {
            return Err(format!("row {} has {} columns, expected {}", row + 1, cells.len(), names.len() + 1));
        }
        for cell in &cells[1..] {
            prices.push(cell.trim().parse::<f64>().map_err(|_| format!("row {}: '{cell}' is not a number", row + 1))?);
        }
    }
    Panel::new(names, prices)
}

pub fn read(path: &str) -> Result<Panel, String> {
    parse(&fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?)
}
