use alloc::format;
use alloc::string::String;
use custom_types::spin_lock::SpinLock;

#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub day: u8,
    pub month: u8,
    pub year: u16,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

pub static CURRENT_TIME: SpinLock<DateTime> = SpinLock::new(DateTime {
    day: 1,
    month: 1,
    year: 2025,
    hours: 12,
    minutes: 0,
    seconds: 0,
});

impl DateTime {
    const MAX_MONTHS: u8 = 12;
    const MAX_HOURS: u8 = 24;
    const MAX_MINUTES: u8 = 60;
    const MAX_SECONDS: u8 = 60;

    pub fn now() -> String {
        let current_datetime = CURRENT_TIME.lock();
        format!(
            "{:02}.{:02}.{:04} {:02}:{:02}:{:02}",
            current_datetime.day,
            current_datetime.month,
            current_datetime.year,
            current_datetime.hours,
            current_datetime.minutes,
            current_datetime.seconds
        )
    }

    pub fn update(&mut self) {
        self.seconds += 1;
        if self.seconds < Self::MAX_SECONDS {
            return;
        }

        self.seconds = 0;
        self.minutes += 1;
        if self.minutes < Self::MAX_MINUTES {
            return;
        }

        self.minutes = 0;
        self.hours += 1;
        if self.hours < Self::MAX_HOURS {
            return;
        }

        self.hours = 0;
        self.day += 1;

        if self.day <= Self::days_in_month(self.month, self.year) {
            return;
        }

        self.day = 1;
        self.month += 1;

        if self.month <= Self::MAX_MONTHS {
            return;
        }

        self.month = 1;
        self.year += 1;
    }

    pub fn set_time(&mut self, hours: u8, minutes: u8, seconds: u8) -> Result<(), &'static str> {
        if hours > 23 || minutes > 59 || seconds > 59 {
            return Err("Invalid time");
        }
        self.hours = hours;
        self.minutes = minutes;
        self.seconds = seconds;
        Ok(())
    }

    pub fn set_date(&mut self, day: u8, month: u8, year: u16) -> Result<(), &'static str> {
        if month == 0 || month > 12 {
            return Err("Invalid month");
        }
        let max_days = Self::days_in_month(month, year);
        if day == 0 || day > max_days {
            return Err("Invalid day for given month/year");
        }
        self.day = day;
        self.month = month;
        self.year = year;
        Ok(())
    }

    pub fn time_string(&self) -> String {
        format!("{:02}:{:02}", self.hours, self.minutes)
    }

    pub fn date_string(&self) -> String {
        format!("{:02}.{:02}.{:04}", self.day, self.month, self.year)
    }

    fn days_in_month(month: u8, year: u16) -> u8 {
        match month {
            1 => 31,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            3 => 31,
            4 => 30,
            5 => 31,
            6 => 30,
            7 => 31,
            8 => 31,
            9 => 30,
            10 => 31,
            11 => 30,
            12 => 31,
            _ => 30,
        }
    }

    fn is_leap_year(year: u16) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    pub fn get_time() -> (u8, u8, u8) {
        let time = CURRENT_TIME.lock();
        (time.hours, time.minutes, time.seconds)
    }

    pub fn get_date() -> (u8, u8, u16) {
        let time = CURRENT_TIME.lock();
        (time.day, time.month, time.year)
    }
}
