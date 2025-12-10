use std::iter;

pub fn format_table(table: &Vec<Vec<String>>) -> Vec<String> {
    let longest_row = table.iter().map(|row| row.len()).max().unwrap_or(0);

    let max_by_cols = table.iter().fold(
        Vec::from_iter(iter::repeat_n(0, longest_row)),
        |maxes, row| {
            row.iter()
                .chain(iter::repeat_n(&String::new(), longest_row - row.len()))
                .zip(maxes)
                .map(|(el, max)| el.len().max(max))
                .collect()
        },
    );

    table
        .iter()
        .map(|row| {
            row.iter()
                .zip(&max_by_cols)
                .map(|(el, max)| format!("{}{}", el, " ".repeat(max - el.len() + 1)))
                .collect()
        })
        .collect()
}
