use std::time::Duration;

pub enum Schedule {
    _OneMonth,
    _SevenDays,
    _SixDays,
    OneDay,
    _TwelveHours,
    _OneHour,
    _ThirtyMinutes,
    _OneMinute,
}

impl Schedule {
    pub fn to_duration(&self) -> Duration {
        match self {
            Schedule::_OneMonth => Duration::from_secs(30 * 24 * 60 * 60), // 1 mes
            Schedule::_SevenDays => Duration::from_secs(7 * 24 * 60 * 60), // 7 dias
            Schedule::_SixDays => Duration::from_secs(6 * 24 * 60 * 60),   // 6 dias
            Schedule::OneDay => Duration::from_secs(24 * 60 * 60),         // 1 dia
            Schedule::_TwelveHours => Duration::from_secs(12 * 60 * 60),   // 12 horas
            Schedule::_OneHour => Duration::from_secs(60 * 60),            // 1 hora
            Schedule::_ThirtyMinutes => Duration::from_secs(30 * 60),      // 30 min
            Schedule::_OneMinute => Duration::from_secs(60),               // 1 min
        }
    }
}
