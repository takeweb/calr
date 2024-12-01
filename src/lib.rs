mod utils;

use anyhow::{Error, Result};
use chrono::{Datelike, Local, NaiveDate};
use clap::Parser;
use itertools::izip;
use utils::date_util::{
    format_month, get_after_month, get_before_month, get_calendar, get_year_month,
    is_all_same_year, parse_month,
};

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of `cal`
struct Args {
    /// Year (1-9999)
    #[arg(value_parser(clap::value_parser!(i32).range(1..=9999)))]
    year: Option<i32>,

    /// Month name or number (1-12)
    #[arg(short)]
    month: Option<String>,

    /// Show the whole current year
    #[arg(short('y'), long("year"), default_value_t = false, conflicts_with_all(["month", "year"]))]
    show_current_year: bool,

    /// Show near 3 month
    #[arg(short('3'), long, value_parser, default_value_t = false)]
    three: bool,
}

pub fn run() -> Result<String> {
    // コマンドライン引数解析
    let (year, month, today, three_flg) = parse_args()?;
    let year = year.unwrap_or(today.year());

    // オプション「-3」の処理
    if three_flg {
        let month = month.unwrap_or(today.month());

        // 対象とする期間を決定(前後１ヶ月)
        let start_date = get_before_month(1, year, month);
        let end_date = get_after_month(1, year, month);
        let year_months = get_year_month(start_date, end_date);

        // すべての年が同じか確認
        let all_same_year = is_all_same_year(year_months.clone());
        if all_same_year {
            println!("{year:>32}");
        }

        // カレンダー生成
        let calendar: Vec<_> = get_calendar(year_months, !all_same_year, today);

        // カレンダーを３ヶ月毎にコンソール出力
        print_chunk_tree_month(calendar);
    } else {
        match month {
            Some(month) => {
                let lines = format_month(year, month, true, today);
                println!("{}", lines.join("\n"));
            }
            None => {
                println!("{year:>32}");

                // 対象とする期間を決定(対象年1年間)
                let start_date = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
                let end_date = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
                let year_months = get_year_month(start_date, end_date);

                // カレンダー生成
                let calendar: Vec<_> = get_calendar(year_months, false, today);

                // カレンダーを３ヶ月毎にコンソール出力
                print_chunk_tree_month(calendar);
            }
        }
    }

    Ok(String::from("Success"))
}

/// コマンドライン引数を解析
fn parse_args() -> Result<(Option<i32>, Option<u32>, NaiveDate, bool), Error> {
    let args = Args::parse();
    let today = Local::now().date_naive();
    let mut year = args.year;
    let mut month = args.month.map(parse_month).transpose()?;

    if args.show_current_year {
        year = Some(today.year());
        month = None;
    } else if month.is_none() && year.is_none() {
        year = Some(today.year());
        month = Some(today.month());
    }
    Ok((year, month, today, args.three))
}

/// カレンダーを３ヶ月毎にコンソール出力
fn print_chunk_tree_month(calendar: Vec<Vec<String>>) {
    for (i, chunk) in calendar.chunks(3).enumerate() {
        if let [m1, m2, m3] = chunk {
            for lines in izip!(m1, m2, m3) {
                println!("{}{}{}", lines.0, lines.1, lines.2);
            }
            if i < 3 {
                println!();
            }
        }
    }
}
