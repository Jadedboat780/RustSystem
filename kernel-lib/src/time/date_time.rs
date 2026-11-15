use core::fmt;

const SECONDS_PER_DAY: u64 = 86_400;
const MIN_YEAR: u16 = 1970;
const MAX_YEAR: u16 = 9999;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeError {
    InvalidTime,
    InvalidMonth,
    InvalidDay,
    YearOutOfRange,
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TimeError::InvalidTime => "invalid time",
            TimeError::InvalidMonth => "invalid month",
            TimeError::InvalidDay => "invalid day for the given month and year",
            TimeError::YearOutOfRange => "year must be between 1970 and 9999",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    const ALL: [Weekday; 7] = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Weekday::Monday => "Monday",
            Weekday::Tuesday => "Tuesday",
            Weekday::Wednesday => "Wednesday",
            Weekday::Thursday => "Thursday",
            Weekday::Friday => "Friday",
            Weekday::Saturday => "Saturday",
            Weekday::Sunday => "Sunday",
        }
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
    year: u16,
    month: u8,
    day: u8,
    hours: u8,
    minutes: u8,
    seconds: u8,
}

impl DateTime {
    pub const UNIX_EPOCH: DateTime = DateTime {
        year: 1970,
        month: 1,
        day: 1,
        hours: 0,
        minutes: 0,
        seconds: 0,
    };

    pub const MAX: DateTime = DateTime {
        year: MAX_YEAR,
        month: 12,
        day: 31,
        hours: 23,
        minutes: 59,
        seconds: 59,
    };

    pub const fn new(year: u16, month: u8, day: u8, hours: u8, minutes: u8, seconds: u8) -> Result<Self, TimeError> {
        if year < MIN_YEAR || year > MAX_YEAR {
            return Err(TimeError::YearOutOfRange);
        }
        if month == 0 || month > 12 {
            return Err(TimeError::InvalidMonth);
        }
        if day == 0 || day > days_in_month(month, year) {
            return Err(TimeError::InvalidDay);
        }
        if hours > 23 || minutes > 59 || seconds > 59 {
            return Err(TimeError::InvalidTime);
        }
        Ok(Self {
            year,
            month,
            day,
            hours,
            minutes,
            seconds,
        })
    }

    pub const fn year(&self) -> u16 {
        self.year
    }

    pub const fn month(&self) -> u8 {
        self.month
    }

    pub const fn day(&self) -> u8 {
        self.day
    }

    pub const fn hours(&self) -> u8 {
        self.hours
    }

    pub const fn minutes(&self) -> u8 {
        self.minutes
    }

    pub const fn seconds(&self) -> u8 {
        self.seconds
    }

    pub const fn with_time(self, hours: u8, minutes: u8, seconds: u8) -> Result<Self, TimeError> {
        Self::new(self.year, self.month, self.day, hours, minutes, seconds)
    }

    pub const fn with_date(self, year: u16, month: u8, day: u8) -> Result<Self, TimeError> {
        Self::new(year, month, day, self.hours, self.minutes, self.seconds)
    }

    pub fn weekday(&self) -> Weekday {
        let days = days_from_civil(self.year, self.month, self.day);
        Weekday::ALL[((days + 3) % 7) as usize]
    }

    pub fn to_unix(&self) -> u64 {
        let days = days_from_civil(self.year, self.month, self.day);
        days * SECONDS_PER_DAY + self.hours as u64 * 3600 + self.minutes as u64 * 60 + self.seconds as u64
    }

    pub fn from_unix(timestamp: u64) -> Self {
        if timestamp > Self::MAX.to_unix() {
            return Self::MAX;
        }
        let (year, month, day) = civil_from_days(timestamp / SECONDS_PER_DAY);
        let secs = timestamp % SECONDS_PER_DAY;
        Self {
            year,
            month,
            day,
            hours: (secs / 3600) as u8,
            minutes: (secs / 60 % 60) as u8,
            seconds: (secs % 60) as u8,
        }
    }

    pub const fn time(&self) -> TimeDisplay {
        TimeDisplay(*self)
    }

    pub const fn date(&self) -> DateDisplay {
        DateDisplay(*self)
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:02}:{:02}:{:02}", self.date(), self.hours, self.minutes, self.seconds)
    }
}

pub struct TimeDisplay(DateTime);

impl fmt::Display for TimeDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.0.hours, self.0.minutes)
    }
}

pub struct DateDisplay(DateTime);

impl fmt::Display for DateDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}.{:02}.{:04}", self.0.day, self.0.month, self.0.year)
    }
}

const fn is_leap_year(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

const fn days_in_month(month: u8, year: u16) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn days_from_civil(year: u16, month: u8, day: u8) -> u64 {
    let year = year as u64 - (month <= 2) as u64;
    let era = year / 400;
    let year_of_era = year % 400;
    let month = month as u64;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day as u64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: u64) -> (u16, u8, u8) {
    let days = days + 719_468;
    let era = days / 146_097;
    let day_of_era = days % 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + (month <= 2) as u64;
    (year as u16, month as u8, day as u8)
}
