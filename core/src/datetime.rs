use std::error::Error;

use chrono::{
    Datelike, Days, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Timelike,
};
use regex::Regex;

const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
const DATE_FORMAT: &str = "%Y-%m-%d";
const TIME_FORMAT: &str = "%H:%M";

/// Time format
pub trait TimeDisplay {
    /// Convert datetime/time to `String`, its format is `HH:MM`.
    fn to_string_hm(&self) -> String;
}

/// Represents time.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct TaskTime(NaiveDateTime);

impl From<NaiveDateTime> for TaskTime {
    fn from(value: NaiveDateTime) -> Self {
        let dt = value.with_second(0).unwrap().with_nanosecond(0).unwrap();
        TaskTime(dt)
    }
}

impl From<TaskTime> for NaiveDateTime {
    fn from(value: TaskTime) -> Self {
        value.0
    }
}

impl std::fmt::Display for TaskTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.0.format(DATETIME_FORMAT);
        write!(f, "{}", v)
    }
}

impl std::ops::Sub for TaskTime {
    type Output = TimeDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl std::ops::Sub for &TaskTime {
    type Output = TimeDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl TimeDisplay for TaskTime {
    fn to_string_hm(&self) -> String {
        self.0.format(TIME_FORMAT).to_string()
    }
}

impl TaskTime {
    /// Tries to parse given string to `TaskTime`. The expected format is `YYYY-MM-DDTHH:MM:SS`.
    pub fn parse(s: &str) -> Result<Self, Box<dyn Error>> {
        let datetime = NaiveDateTime::parse_from_str(s, DATETIME_FORMAT)?;
        Ok(datetime.into())
    }

    /// Tries to parse gigen time string to `TaskTime` using the current date.
    /// The expected format is `HH:MM` or `HHMM`.
    pub fn parse_hm(s: &str) -> Result<Self, Box<dyn Error>> {
        let (h, m) = parse_time_hm(s)?;
        let today = Local::now().date_naive();
        let time = today.and_hms_opt(h, m, 0).unwrap();
        Ok(time.into())
    }

    /// Tries to build a `TaskTime` from a `WorkingDate` and `HH:MM`/`HHMM` string.
    pub fn parse_with_date(date: &WorkingDate, time: &str) -> Result<Self, Box<dyn Error>> {
        let centinel = NaiveTime::from_hms_opt(5, 0, 0).unwrap();

        let (h, m) = parse_time_hm(time)?;
        let time = NaiveTime::from_hms_opt(h, m, 0).unwrap();
        if time < centinel {
            let date = date.0.checked_add_days(Days::new(1)).unwrap();
            return Ok(date.and_time(time).into());
        }
        Ok(date.0.and_hms_opt(h, m, 0).unwrap().into())
    }

    /// Current time.
    pub fn now() -> Self {
        let now = Local::now().naive_local();
        now.into()
    }
}

impl TimeDisplay for TimeDelta {
    fn to_string_hm(&self) -> String {
        let minutes = self.num_minutes();
        let quo = (minutes / 60).abs();
        let rem = (minutes % 60).abs();
        let sign = if minutes < 0 { "-" } else { "" };
        format!("{}{:>02}:{:>02}", sign, quo, rem)
    }
}

/// Represents a date.
///
/// In `WorkingDate`, after 5:00 am is considered as the next date.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct WorkingDate(NaiveDate);

impl From<NaiveDate> for WorkingDate {
    fn from(value: NaiveDate) -> Self {
        WorkingDate(value)
    }
}

impl From<TaskTime> for WorkingDate {
    fn from(value: TaskTime) -> Self {
        let date = value.0.date();
        let start = &date.and_hms_opt(5, 0, 0).unwrap();

        if &value.0 >= start {
            WorkingDate(date)
        } else {
            WorkingDate(date.pred_opt().unwrap())
        }
    }
}

impl From<&WorkingDate> for NaiveDate {
    fn from(value: &WorkingDate) -> Self {
        value.0
    }
}

impl std::fmt::Display for WorkingDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.0.format(DATE_FORMAT);
        write!(f, "{}", v)
    }
}

impl WorkingDate {
    /// Tries to parse given string to `WorkingDate`. The expected format is `YYYY-MM-DD`.
    pub fn parse(s: &str) -> Result<Self, Box<dyn Error>> {
        let (y, m, d) = parse_date(s)?;
        let date = NaiveDate::from_ymd_opt(y, m, d).ok_or("invalid date")?;
        Ok(date.into())
    }

    /// Tries to parse given year and month string (`YYYY-MM` or `YYYYMM`) to (start, end) tuple.
    ///
    /// Start is the first day of the month, and end is the last day of the month.
    pub fn parse_ym(s: &str) -> Result<(Self, Self), Box<dyn Error>> {
        let (y, m) = parse_yearmonth(s)?;
        let date_first = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
        let date_last = NaiveDate::from_ymd_opt(y, m, 1)
            .and_then(|d| d.checked_add_months(Months::new(1)))
            .and_then(|d| d.pred_opt())
            .unwrap();

        Ok((date_first.into(), date_last.into()))
    }

    /// Build `TaskTime` with hour and minutes.
    pub fn and_hm_opt(&self, hour: u32, min: u32) -> Option<TaskTime> {
        let centinel = NaiveTime::from_hms_opt(5, 0, 0).unwrap();

        if let Some(time) = NaiveTime::from_hms_opt(hour, min, 0) {
            if time < centinel {
                let date = self.0.checked_add_days(Days::new(1)).unwrap();
                return Some(date.and_time(time).into());
            }
            Some(self.0.and_hms_opt(hour, min, 0).unwrap().into())
        } else {
            None
        }
    }

    /// Current date
    pub fn today() -> Self {
        TaskTime::now().into()
    }
}

/// Parse time string (`HH:MM`, `H:MM`, `HHMM`, or `HMM`) to (hour, minutes) tuple.
fn parse_time_hm(s: &str) -> Result<(u32, u32), Box<dyn Error>> {
    let time_re = Regex::new(r"^([0-9]|[01][0-9]|2[0-3]):?([0-5][0-9])$").unwrap();
    let captures = time_re.captures(s).ok_or("invalid format")?;

    let h = captures.get(1).unwrap().as_str().parse()?;
    let m = captures.get(2).unwrap().as_str().parse()?;

    Ok((h, m))
}

/// Parse date string (`YYYY-MM-DD` or `YYYYMMDD`) to (year, month, day) tuple.
fn parse_date(s: &str) -> Result<(i32, u32, u32), Box<dyn Error>> {
    let date_re =
        Regex::new(r"^(([0-9]{4})-?)?(0[1-9]|1[0-2])-?(0[1-9]|[12][0-9]|3[01])$").unwrap();
    let captures = date_re.captures(s).ok_or("invalid format")?;

    let y = if let Some(matched) = captures.get(2) {
        matched.as_str().parse()?
    } else {
        Local::now().year()
    };
    let m = captures.get(3).unwrap().as_str().parse()?;
    let d = captures.get(4).unwrap().as_str().parse()?;

    Ok((y, m, d))
}

/// Parse year-month string (`YYYY-MM` or `YYYYMM`) to (year, month) tuple.
fn parse_yearmonth(s: &str) -> Result<(i32, u32), Box<dyn Error>> {
    let ym_re = Regex::new(r"^([0-9]{4})-?(0[1-9]|1[0-2])$").unwrap();
    let captures = ym_re.captures(s).ok_or("invalid format")?;

    let y = captures.get(1).unwrap().as_str().parse()?;
    let m = captures.get(2).unwrap().as_str().parse()?;

    Ok((y, m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tasktime_parse() {
        let result = TaskTime::parse("2022-06-30T11:30:25").unwrap();
        let expected = TaskTime(
            NaiveDateTime::parse_from_str("2022-06-30T11:30:00", DATETIME_FORMAT).unwrap(),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tasktime_parse_with_date() {
        let date = WorkingDate::parse("2021-01-01").unwrap();
        let result = TaskTime::parse_with_date(&date, "500").unwrap();
        let expected = TaskTime::parse("2021-01-01T05:00:00").unwrap();
        assert_eq!(result, expected);

        let date = WorkingDate::parse("2021-01-01").unwrap();
        let result = TaskTime::parse_with_date(&date, "1000").unwrap();
        let expected = TaskTime::parse("2021-01-01T10:00:00").unwrap();
        assert_eq!(result, expected);

        let date = WorkingDate::parse("2021-01-01").unwrap();
        let result = TaskTime::parse_with_date(&date, "2359").unwrap();
        let expected = TaskTime::parse("2021-01-01T23:59:00").unwrap();
        assert_eq!(result, expected);

        let date = WorkingDate::parse("2021-01-01").unwrap();
        let result = TaskTime::parse_with_date(&date, "0000").unwrap();
        let expected = TaskTime::parse("2021-01-02T00:00:00").unwrap();
        assert_eq!(result, expected);

        let date = WorkingDate::parse("2021-01-01").unwrap();
        let result = TaskTime::parse_with_date(&date, "459").unwrap();
        let expected = TaskTime::parse("2021-01-02T04:59:00").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tasktime_to_string() {
        let t_str = "2022-06-30T11:30:25";
        let t = NaiveDateTime::parse_from_str(t_str, DATETIME_FORMAT).unwrap();
        assert_eq!(TaskTime::from(t).to_string(), "2022-06-30T11:30:00");
        assert_eq!(TaskTime::from(t).to_string_hm(), "11:30");
    }

    #[test]
    fn test_duration() {
        let t1 = NaiveDateTime::parse_from_str("2015-09-18T23:56:00", DATETIME_FORMAT).unwrap();
        let t2 = NaiveDateTime::parse_from_str("2015-09-19T01:10:00", DATETIME_FORMAT).unwrap();

        let dur = &TaskTime::from(t2) - &TaskTime::from(t1);
        assert_eq!(dur, TimeDelta::minutes(74));
        assert_eq!(dur.to_string_hm(), "01:14");

        let dur = &TaskTime::from(t1) - &TaskTime::from(t2);
        assert_eq!(dur, TimeDelta::minutes(-74));
        assert_eq!(dur.to_string_hm(), "-01:14");
    }

    #[test]
    fn test_workingdate_parse_ymd() {
        let result = WorkingDate::parse("2021-01-01").unwrap();
        let expected = WorkingDate(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        assert_eq!(result, expected);

        let result = WorkingDate::parse("20210101").unwrap();
        let expected = WorkingDate(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_workingdate_parse_md() {
        let this_year = Local::now().year();
        let result = WorkingDate::parse("01-01").unwrap();
        let expected = WorkingDate(NaiveDate::from_ymd_opt(this_year, 1, 1).unwrap());
        assert_eq!(result, expected);

        let result = WorkingDate::parse("0101").unwrap();
        let expected = WorkingDate(NaiveDate::from_ymd_opt(this_year, 1, 1).unwrap());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_workingdate_parse_ym() {
        let st_expected = WorkingDate::parse("2021-04-01").unwrap();
        let en_expected = WorkingDate::parse("2021-04-30").unwrap();

        let (st, en) = WorkingDate::parse_ym("2021-04").unwrap();
        assert_eq!(st, st_expected);
        assert_eq!(en, en_expected);

        let (st, en) = WorkingDate::parse_ym("202104").unwrap();
        assert_eq!(st, st_expected);
        assert_eq!(en, en_expected);
    }

    #[test]
    fn test_workingdate_and_hm_opt() {
        let date = WorkingDate::parse("2021-01-01").unwrap();

        assert_eq!(
            date.and_hm_opt(5, 0).unwrap(),
            TaskTime::parse("2021-01-01T05:00:00").unwrap()
        );
        assert_eq!(
            date.and_hm_opt(10, 30).unwrap(),
            TaskTime::parse("2021-01-01T10:30:00").unwrap()
        );
        assert_eq!(
            date.and_hm_opt(23, 59).unwrap(),
            TaskTime::parse("2021-01-01T23:59:00").unwrap()
        );
        assert_eq!(
            date.and_hm_opt(0, 0).unwrap(),
            TaskTime::parse("2021-01-02T00:00:00").unwrap()
        );
        assert_eq!(
            date.and_hm_opt(4, 59).unwrap(),
            TaskTime::parse("2021-01-02T04:59:00").unwrap()
        );
    }

    #[test]
    fn test_workingdate_to_string() {
        let d = WorkingDate(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        assert_eq!(d.to_string(), "2021-01-01");
    }

    #[test]
    fn test_workingdate_creation() {
        let t = NaiveDateTime::parse_from_str("2021-01-01T05:00:00", DATETIME_FORMAT).unwrap();
        let expected = WorkingDate(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        assert_eq!(WorkingDate::from(TaskTime(t)), expected);

        let t = NaiveDateTime::parse_from_str("2021-01-01T23:59:00", DATETIME_FORMAT).unwrap();
        let expected = WorkingDate(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        assert_eq!(WorkingDate::from(TaskTime(t)), expected);

        let t = NaiveDateTime::parse_from_str("2021-01-02T00:00:00", DATETIME_FORMAT).unwrap();
        let expected = WorkingDate(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        assert_eq!(WorkingDate::from(TaskTime(t)), expected);

        let t = NaiveDateTime::parse_from_str("2021-01-02T04:59:00", DATETIME_FORMAT).unwrap();
        let expected = WorkingDate(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap());
        assert_eq!(WorkingDate::from(TaskTime(t)), expected);

        let t = NaiveDateTime::parse_from_str("2021-01-02T05:00:00", DATETIME_FORMAT).unwrap();
        let expected = WorkingDate(NaiveDate::from_ymd_opt(2021, 1, 2).unwrap());
        assert_eq!(WorkingDate::from(TaskTime(t)), expected);
    }

    #[test]
    fn test_parse_time_hm() {
        assert_eq!(parse_time_hm("2310").unwrap(), (23, 10));
        assert_eq!(parse_time_hm("0559").unwrap(), (5, 59));
        assert_eq!(parse_time_hm("559").unwrap(), (5, 59));
        assert_eq!(parse_time_hm("0605").unwrap(), (6, 5));
        assert_eq!(parse_time_hm("605").unwrap(), (6, 5));
        assert_eq!(parse_time_hm("23:10").unwrap(), (23, 10));
        assert_eq!(parse_time_hm("05:59").unwrap(), (5, 59));
        assert_eq!(parse_time_hm("6:05").unwrap(), (6, 5));

        assert!(parse_time_hm("aaa").is_err());
        assert!(parse_time_hm("2410").is_err());
        assert!(parse_time_hm("0560").is_err());
        assert!(parse_time_hm("24:10").is_err());
        assert!(parse_time_hm("05:60").is_err());
        assert!(parse_time_hm("5:60").is_err());
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(parse_date("2021-01-01").unwrap(), (2021, 1, 1));
        assert_eq!(parse_date("2021-12-31").unwrap(), (2021, 12, 31));
        assert_eq!(parse_date("20210101").unwrap(), (2021, 1, 1));
        assert_eq!(parse_date("20211231").unwrap(), (2021, 12, 31));

        let this_year = Local::now().year();
        assert_eq!(parse_date("01-01").unwrap(), (this_year, 1, 1));
        assert_eq!(parse_date("12-31").unwrap(), (this_year, 12, 31));
        assert_eq!(parse_date("0101").unwrap(), (this_year, 1, 1));
        assert_eq!(parse_date("1231").unwrap(), (this_year, 12, 31));

        assert!(parse_date("2021-00-01").is_err());
        assert!(parse_date("2021-13-31").is_err());
        assert!(parse_date("20210100").is_err());
        assert!(parse_date("20211232").is_err());
    }
}
