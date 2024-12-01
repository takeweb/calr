use ansi_term::Style;
use anyhow::{bail, Result};
use chrono::{Datelike, Months, NaiveDate};

const LINE_WIDTH: usize = 22;
pub const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// 対象年月の最終日を取得
///
/// * `year`  - 対象年
/// * `month` - 対象月
fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
}

/// 対象月をカレンダー形式フォーマットする
///
/// * `year`  - 対象年
/// * `month` - 対象月
/// * `add_year` - 年ヘッダを追加するか否か
/// * `today` - 当日日付
pub fn format_month(year: i32, month: u32, add_year: bool, today: NaiveDate) -> Vec<String> {
    let is_today = |day: u32| year == today.year() && month == today.month() && day == today.day();

    // 月ヘッダを行に追加
    let month_name = MONTH_NAMES[month as usize - 1];
    let mut lines = Vec::with_capacity(8);
    lines.push(format!(
        "{:^20}  ", // two trailing spaces
        if add_year {
            format!("{month_name} {year}")
        } else {
            month_name.to_string()
        }
    ));

    // 曜日ヘッダを行に追加
    lines.push("Su Mo Tu We Th Fr Sa  ".to_string()); // two trailing spaces

    // 対象期間のカレンダーを生成
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let last = last_day_in_month(year, month);
    let mut days: Vec<String> = (1..first.weekday().number_from_sunday())
        .map(|_| "  ".to_string()) // two spaces
        .collect();

    days.extend((first.day()..=last.day()).map(|num| {
        let fmt = format!("{num:>2}");
        if is_today(num) {
            Style::new().reverse().paint(fmt).to_string()
        } else {
            fmt
        }
    }));

    // 対象カレンダーを週単位(7日毎)に分割
    for week in days.chunks(7) {
        lines.push(format!(
            "{:width$}  ", // two trailing spaces
            week.join(" "),
            width = LINE_WIDTH - 2
        ));
    }

    // 空行補完
    while lines.len() < 8 {
        lines.push(" ".repeat(LINE_WIDTH));
    }

    lines
}

/// 対象年月のnヶ月前の日付取得
/// 対象年月のnヶ月前を算出して、その年月の1日の日付を返す。
///
/// * `n`     - nヶ月前
/// * `year`  - 対象年
/// * `month` - 対象月
pub fn get_before_month(n: u32, year: i32, month: u32) -> NaiveDate {
    let target_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    target_date - Months::new(n)
}

/// 対象年月のnヶ月後の日付取得
/// 対象年月のnヶ月後を算出して、その年月の最終日の日付を返す。
///
/// * `n`     - nヶ月後
/// * `year`  - 対象年
/// * `month` - 対象月
pub fn get_after_month(n: u32, year: i32, month: u32) -> NaiveDate {
    let target_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let after_month_first = target_date + Months::new(n);

    NaiveDate::from_ymd_opt(
        after_month_first.year(),
        after_month_first.month(),
        get_days_from_ym(after_month_first.year(), after_month_first.month()),
    )
    .unwrap()
}

/// 対象年月の日数取得
/// 対象年月の日数を算出して返す。
///
/// * `year`  - 対象年
/// * `month` - 対象月
fn get_days_from_ym(year: i32, month: u32) -> u32 {
    let days = NaiveDate::from_ymd_opt(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .unwrap()
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days();
    TryFrom::try_from(days).unwrap()
}

/// 開始日から終了日が含まれる年月を取得
/// カレンダーに含まれる月初めの日付だけをフィルタリング後、年月だけに加工したVecを返す。
///
/// * `start_date`  - 開始日
/// * `end_date`    - 終了日
pub fn get_year_month(start_date: NaiveDate, end_date: NaiveDate) -> Vec<(i32, u32)> {
    let calendar: Vec<NaiveDate> = (0..)
        .map(|i| start_date + chrono::Duration::days(i))
        .take_while(|date| *date <= end_date)
        .collect();

    // カレンダーに含まれる月初めの日付だけをフィルタリング後、年月だけに加工したVecを返す
    let year_month: Vec<(i32, u32)> = calendar
        .into_iter()
        .filter(|date| date.day() == 1) // 月初の日付のみ
        .map(|date| (date.year(), date.month()))
        .collect();

    year_month
}

/// 全ての年が同じか確認
/// 対象年月から年だけを取り出して、最初の年と同じかを比較する。
///
/// * `year_months`    - 対象年月タプルのVec
pub fn is_all_same_year(year_months: Vec<(i32, u32)>) -> bool {
    let all_same_year = year_months
        .iter()
        .map(|(year, _)| year) // 年だけを取り出す
        .all(|&year| year == year_months[0].0); // 最初の年と比較

    all_same_year
}

/// カレンダー生成
/// 対象年月からカレンダーを生成する。
///
/// * `year_months`    - 対象年月タプルのVec
/// * `all_same_year`  - 全ての年月が同じ年か否か
/// * `today`          - 当日日付
pub fn get_calendar(
    year_months: Vec<(i32, u32)>,
    all_same_year: bool,
    today: NaiveDate,
) -> Vec<Vec<String>> {
    // カレンダー生成
    let calendar: Vec<_> = year_months
        .clone()
        .into_iter()
        .map(|(year, month)| format_month(year, month, all_same_year, today))
        .collect();
    calendar
}

/// 引数・月の解析
/// 受け取った月(数値 or 文字列)を解析してu32型に変換して返す。
///
/// * `year_months`    - 対象年月タプルのVec
/// * `all_same_year`  - 全ての年月が同じ年か否か
/// * `today`          - 当日日付
pub fn parse_month(month: String) -> Result<u32> {
    match month.parse() {
        Ok(num) => {
            if (1..=12).contains(&num) {
                Ok(num)
            } else {
                bail!(r#"month "{month}" not in the range 1 through 12"#)
            }
        }
        _ => {
            let lower = &month.to_lowercase();
            let matches: Vec<_> = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    if name.to_lowercase().starts_with(lower) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .collect();

            if matches.len() == 1 {
                Ok(matches[0] as u32)
            } else {
                bail!(r#"Invalid month "{month}""#)
            }
        }
    }
}

// ---------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{
        format_month, get_after_month, get_before_month, get_year_month, is_all_same_year,
        last_day_in_month, parse_month,
    };
    use chrono::NaiveDate;

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
    }

    #[test]
    fn test_get_before_month() {
        assert_eq!(
            get_before_month(0, 2022, 1),
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()
        );
        assert_eq!(
            get_before_month(1, 2022, 1),
            NaiveDate::from_ymd_opt(2021, 12, 1).unwrap()
        );
        assert_eq!(
            get_before_month(3, 2022, 1),
            NaiveDate::from_ymd_opt(2021, 10, 1).unwrap()
        );
        assert_eq!(
            get_before_month(6, 2022, 12),
            NaiveDate::from_ymd_opt(2022, 6, 1).unwrap()
        );
        assert_eq!(
            get_before_month(12, 2022, 2,),
            NaiveDate::from_ymd_opt(2021, 2, 1).unwrap()
        );
    }

    #[test]
    fn test_get_after_month() {
        assert_eq!(
            get_after_month(0, 2022, 6),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap()
        );
        assert_eq!(
            get_after_month(1, 2022, 6),
            NaiveDate::from_ymd_opt(2022, 7, 31).unwrap()
        );
        assert_eq!(
            get_after_month(3, 2022, 1),
            NaiveDate::from_ymd_opt(2022, 4, 30).unwrap()
        );
        assert_eq!(
            get_after_month(6, 2022, 12),
            NaiveDate::from_ymd_opt(2023, 6, 30).unwrap()
        );
        assert_eq!(
            get_after_month(12, 2022, 2),
            NaiveDate::from_ymd_opt(2023, 2, 28).unwrap()
        );
    }

    #[test]
    fn test_get_year_month() {
        let start_date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 8, 31).unwrap();
        assert_eq!(
            get_year_month(start_date, end_date),
            vec![(2024, 6), (2024, 7), (2024, 8)]
        );
        let start_date = NaiveDate::from_ymd_opt(2024, 11, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2025, 1, 31).unwrap();
        assert_eq!(
            get_year_month(start_date, end_date),
            vec![(2024, 11), (2024, 12), (2025, 1)]
        );
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert_eq!(
            get_year_month(start_date, end_date),
            vec![
                (2024, 1),
                (2024, 2),
                (2024, 3),
                (2024, 4),
                (2024, 5),
                (2024, 6),
                (2024, 7),
                (2024, 8),
                (2024, 9),
                (2024, 10),
                (2024, 11),
                (2024, 12)
            ]
        );
    }

    #[test]
    fn test_is_all_same_year() {
        assert!(is_all_same_year(vec![(2024, 6), (2024, 7), (2024, 8)]));
        assert!(!is_all_same_year(vec![(2024, 11), (2024, 12), (2025, 1)]),);
        assert!(is_all_same_year(vec![
            (2024, 1),
            (2024, 2),
            (2024, 3),
            (2024, 4),
            (2024, 5),
            (2024, 6),
            (2024, 7),
            (2024, 8),
            (2024, 9),
            (2024, 10),
            (2024, 11),
            (2024, 12)
        ]),);
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"month "0" not in the range 1 through 12"#
        );

        let res = parse_month("13".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"month "13" not in the range 1 through 12"#
        );

        let res = parse_month("foo".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid month "foo""#);
    }
}
