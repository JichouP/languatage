use clap::Parser;
use languatage::{get_stat, LanguageStat};
use num_format::{Locale, ToFormattedString};
use prettytable::{row, Table};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    path: String,
}

fn main() {
    let arg = Args::parse();
    let path = arg.path;

    let stat = get_stat(path).unwrap();

    let mut table = Table::init(vec![row![b->"Language", b->"Percentage", b->"Size"]]);

    stat.iter().filter(|stat| stat.size != 0).for_each(|stat| {
        let LanguageStat {
            lang,
            percentage,
            size,
            ..
        } = stat;
        table.add_row(row![
            lang,
            r->format!("{: >5}%", (percentage * 100.0).round() / 100.0),
            r->size.to_formatted_string(&Locale::en)
        ]);
    });

    table.printstd();
}
