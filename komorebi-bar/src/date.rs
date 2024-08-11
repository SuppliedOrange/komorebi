use crate::widget::BarWidget;

#[derive(Copy, Clone, Debug)]
pub enum DateFormat {
    MonthDateYear,
    YearMonthDate,
    DateMonthYear,
    DayDateMonthYear,
}

impl DateFormat {
    pub fn next(&mut self) {
        match self {
            DateFormat::MonthDateYear => *self = Self::YearMonthDate,
            DateFormat::YearMonthDate => *self = Self::DateMonthYear,
            DateFormat::DateMonthYear => *self = Self::DayDateMonthYear,
            DateFormat::DayDateMonthYear => *self = Self::MonthDateYear,
        };
    }

    fn fmt_string(&self) -> String {
        match self {
            DateFormat::MonthDateYear => String::from("%D"),
            DateFormat::YearMonthDate => String::from("%F"),
            DateFormat::DateMonthYear => String::from("%v"),
            DateFormat::DayDateMonthYear => String::from("%A %e %B %Y"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Date {
    pub enable: bool,
    pub format: DateFormat,
}

impl Date {
    pub fn new(enable: bool, format: DateFormat) -> Self {
        Self { enable, format }
    }
}

impl BarWidget for Date {
    fn output(&mut self) -> Vec<String> {
        vec![chrono::Local::now()
            .format(&self.format.fmt_string())
            .to_string()]
    }
}
